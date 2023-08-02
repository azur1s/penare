use std::sync::Arc;
use nih_plug::prelude::*;

mod params;
mod editor;
mod fxs;

use params::PenareParams;
use fxs::{
    filter,
    waveshaper,
    utils::{mix_between, mix_in, hard_clip},
};

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started
struct Penare {
    params: Arc<PenareParams>,
    sample_rate: f32,
    /// Filters
    lp: [filter::Biquad; 2],
    lp_invert: [filter::Biquad; 2],
    hp: [filter::Biquad; 2],
    hp_invert: [filter::Biquad; 2],
}

impl Default for Penare {
    fn default() -> Self {
        Self {
            params: Arc::new(PenareParams::default()),
            sample_rate: 1.0,
            lp: [filter::Biquad::default(); 2],
            lp_invert: [filter::Biquad::default(); 2],
            hp: [filter::Biquad::default(); 2],
            hp_invert: [filter::Biquad::default(); 2],
        }
    }
}

impl Plugin for Penare {
    #[cfg(debug_assertions)]
    const NAME: &'static str = "Penare Debug";
    #[cfg(not(debug_assertions))]
    const NAME: &'static str = "Penare";
    const VENDOR: &'static str = "Azur1s";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "natapat.samutpong@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        self.update_lp();
        self.update_hp();

        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        for filter in &mut self.hp {
            filter.reset();
        }
        for filter in &mut self.lp {
            filter.reset();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            self.filter_update();

            //   Input
            //     ├───────────────┐
            //     │               │ (Dry Signal)
            //   Filter ───────┐   │
            //     │           │   │
            //  Pre-Gain       │ (Excess Signal)
            //     │           │   │
            // Distortions     │   │
            //     │           │   │
            // Post-Gain       │   │
            //     │           │   │
            // Excess Mix ─────┘   │
            //     │               │
            //    Mix ─────────────┘
            //     │
            // Final-Clip
            //     │
            //   Output

            for (channel_idx, sample) in channel_samples.into_iter().enumerate() {
                let dry = *sample;
                // --- Filter ---
                // Apply low-pass filter
                let (s, lp_ex) = self.low_pass(channel_idx, *sample);
                // Apply high-pass filter
                let (s, hp_ex) = self.high_pass(channel_idx, s);
                *sample = s;

                // --- Pre-Gain ---
                *sample *= self.params.pre_gain.smoothed.next(); // Pre-gain

                // --- Distortions ---
                // - Rectify
                if self.params.rectify.value() {
                    let mut rs = self.params.rectify_type.value().apply(*sample); // Rectified signal
                    if self.params.rectify_flip.value() {
                        rs = -rs;
                    }
                    *sample = mix_between(*sample, rs, self.params.rectify_mix.smoothed.next());
                    *sample = mix_in(*sample, rs, self.params.rectify_mix_in.smoothed.next());
                }

                // - Waveshaper
                let should_copy = self.params.copy_function.value();
                let clip = self.params.clip_function.value();
                // Wave shaped signal
                let (mut wss, fp) = if !should_copy.is_false() {
                    // If "Copy Function" is on then the function type is used
                    // for the both positive and negative shape
                    let (ft, fp) = if should_copy.is_positive() {
                        (self.params.pos_function_type.value(),
                        self.params.pos_function_param.smoothed.next())
                    } else {
                        (self.params.neg_function_type.value(),
                        self.params.neg_function_param.smoothed.next())
                    };
                    let mut wss = ft.apply(*sample, fp);
                    // Mix between the original signal and the waveshaped signal
                    let fm = if *sample >= 0.0 {
                        self.params.pos_function_mix.smoothed.next()
                    } else {
                        self.params.neg_function_mix.smoothed.next()
                    };
                    wss = mix_between(*sample, wss, fm);
                    (wss, fp)
                } else {
                    // Otherwise, use the function type for the shape of the signal
                    let fp = if *sample >= 0.0 {
                        self.params.pos_function_param.smoothed.next()
                    } else {
                        self.params.neg_function_param.smoothed.next()
                    };
                    (if *sample >= 0.0 {
                        let mut pws = self.params.pos_function_type.value().apply(*sample, fp);
                        pws = mix_between(*sample, pws, self.params.pos_function_mix.smoothed.next());
                        pws
                    } else {
                        let mut nws = self.params.neg_function_type.value().apply(*sample, fp);
                        nws = mix_between(*sample, nws, self.params.neg_function_mix.smoothed.next());
                        nws
                    }, fp)
                };
                // Clip the waveshaped signal
                if clip {
                    wss = hard_clip(*sample, fp);
                }
                // Flip the phase of the signal
                let wss = if self.params.flip.value() { -wss } else { wss };
                *sample = mix_between(*sample, wss, self.params.function_mix.smoothed.next());

                // - Crusher
                if self.params.crush.value() {
                    // Crushed signal
                    let step = self.params.crush_step.smoothed.next();
                    let cs = self.params.crush_type.value().apply(*sample, step);
                    *sample = mix_between(*sample, cs, self.params.crush_mix.smoothed.next());
                    *sample = mix_in(*sample, cs, self.params.crush_mix_in.smoothed.next());
                }

                // --- Post-Gain ---
                *sample *= self.params.post_gain.smoothed.next(); // Post-gain

                // Filter mix
                if !self.params.excess_bypass.value() {
                    // Mix in excess signal
                    let excess_mix = self.params.excess_mix.smoothed.next();
                    *sample = mix_in(
                        *sample,
                        excess_mix * lp_ex
                        + excess_mix * hp_ex,
                        excess_mix,
                    );
                } else {
                    // Excess signal only
                    *sample = lp_ex + hp_ex;
                }

                // Mix between dry and wet
                *sample = mix_between(dry, *sample, self.params.mix.smoothed.next());

                // Final clip
                if self.params.output_clip.value() {
                    *sample = waveshaper::FunctionType::HardClip.apply(
                        *sample,
                        self.params.output_clip_threshold.smoothed.next(),
                    );
                }
            }

            // Only calculate the UI-related data if the editor is open.
            if self.params.editor_state.is_open() {
            }
        }

        ProcessStatus::Normal
    }
}

impl Penare {
    fn filter_update(&mut self) {
        if self.params.low_pass.smoothed.is_smoothing()
        || self.params.low_pass_q.smoothed.is_smoothing() {
            self.update_lp();
        }
        if self.params.high_pass.smoothed.is_smoothing()
        || self.params.high_pass_q.smoothed.is_smoothing() {
            self.update_hp();
        }
    }

    /// Returns (low-passed, excess)
    fn low_pass(&mut self, channel_index: usize, sample: f32) -> (f32, f32) {
        (
            self.lp[channel_index].process(sample),
            self.lp_invert[channel_index].process(sample),
        )
    }

    /// Returns (high-passed, excess)
    fn high_pass(&mut self, channel_index: usize, sample: f32) -> (f32, f32) {
        (
            self.hp[channel_index].process(sample),
            self.hp_invert[channel_index].process(sample),
        )
    }

    fn update_lp(&mut self) {
        let freq = self.params.low_pass.smoothed.next();
        let q = self.params.low_pass_q.smoothed.next();
        let coeff = filter::Coeff::lowpass(self.sample_rate, freq, q);
        let coeff_invert = filter::Coeff::highpass(self.sample_rate, freq, q);
        for filter in &mut self.lp {
            filter.coeff = coeff;
        }
        for filter in &mut self.lp_invert {
            filter.coeff = coeff_invert;
        }
    }

    fn update_hp(&mut self) {
        let freq = self.params.high_pass.smoothed.next();
        let q = self.params.high_pass_q.smoothed.next();
        let coeff = filter::Coeff::highpass(self.sample_rate, freq, q);
        let coeff_invert = filter::Coeff::lowpass(self.sample_rate, freq, q);
        for filter in &mut self.hp {
            filter.coeff = coeff;
        }
        for filter in &mut self.hp_invert {
            filter.coeff = coeff_invert;
        }
    }
}

impl ClapPlugin for Penare {
    const CLAP_ID: &'static str = "moe.azur.penare";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Penare {
    #[cfg(debug_assertions)]
    const VST3_CLASS_ID: [u8; 16] = *b"PenareDBG!Azur1s";
    #[cfg(not(debug_assertions))]
    const VST3_CLASS_ID: [u8; 16] = *b"Penare....Azur1s";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Penare);
nih_export_vst3!(Penare);
