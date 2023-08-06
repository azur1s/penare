use std::sync::{Arc, Mutex};
use nih_plug::prelude::*;

mod params;
mod data;
mod editor;
mod fxs;

use params::PenareParams;
use data::WaveshapersData;
use fxs::{
    filter,
    waveshaper,
    utils::{mix_between, mix_in},
};

struct Penare {
    params: Arc<PenareParams>,
    sample_rate: f32,
    // Waveshapers Data (for the UI)
    waveshapers_data: Arc<Mutex<WaveshapersData>>,
    // Filters
    f1: [filter::Biquad; 2],
    f2: [filter::Biquad; 2],
}

impl Default for Penare {
    fn default() -> Self {
        Self {
            params: Arc::new(PenareParams::default()),
            sample_rate: 1.0,
            waveshapers_data: Arc::new(Mutex::new(WaveshapersData::default())),
            f1: [filter::Biquad::default(); 2],
            f2: [filter::Biquad::default(); 2],
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
            self.waveshapers_data.clone(),
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

        self.update_waveshapers_data();

        for filter in &mut self.f1 {
            filter.sample_rate = self.sample_rate;
        }
        for filter in &mut self.f2 {
            filter.sample_rate = self.sample_rate;
        }
        self.update_f1();
        self.update_f2();

        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        for filter in &mut self.f1 {
            filter.reset();
        }
        for filter in &mut self.f2 {
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
            self.update_fs();

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
                let (s, f1_ex) = self.f1_process(channel_idx, *sample);
                // Apply high-pass filter
                let (s, f2_ex) = self.f2_process(channel_idx, s);
                *sample = s;

                // --- Pre-Gain ---
                *sample *= self.params.input_gain.smoothed.next();

                // --- Distortions ---

                // - Waveshaper
                let should_copy = self.params.copy_function.value();
                // Wave shaped signal
                let wss = if !should_copy.is_false() {
                    // If "Copy Function" is on then the function type is used
                    // for the both positive and negative shape
                    let (ft, fp) = if should_copy.is_positive() {
                        (self.params.pos_function_type.value(),
                        self.params.pos_function_param.smoothed.next())
                    } else {
                        (self.params.neg_function_type.value(),
                        self.params.neg_function_param.smoothed.next())
                    };
                    // Mix between the original signal and the waveshaped signal
                    mix_between(
                        *sample,
                        ft.apply(*sample, fp),
                        if *sample >= 0.0 {
                            self.params.pos_function_mix.smoothed.next()
                        } else {
                            self.params.neg_function_mix.smoothed.next()
                        },
                    )
                } else {
                    // Otherwise, use the function type for the shape of the signal
                    let fp = if *sample >= 0.0 {
                        self.params.pos_function_param.smoothed.next()
                    } else {
                        self.params.neg_function_param.smoothed.next()
                    };
                    mix_between(
                        *sample,
                        if *sample >= 0.0 {
                            self.params.pos_function_type.value()
                        } else {
                            self.params.neg_function_type.value()
                        }.apply(*sample, fp),
                        if *sample >= 0.0 {
                            self.params.pos_function_mix.smoothed.next()
                        } else {
                            self.params.neg_function_mix.smoothed.next()
                        },
                    )
                };
                // Flip the phase of the signal
                let wss = if self.params.flip.value() { -wss } else { wss };
                *sample = mix_between(*sample, wss, self.params.function_mix.smoothed.next());

                // --- Post-Gain ---
                *sample *= self.params.output_gain.smoothed.next(); // Post-gain

                // Filter mix
                if !self.params.excess_bypass.value() {
                    // Mix in excess signal
                    let excess_mix = self.params.excess_mix.smoothed.next();
                    *sample = mix_in(
                        *sample,
                        excess_mix * f1_ex
                        + excess_mix * f2_ex,
                        excess_mix,
                    );
                } else {
                    // Excess signal only
                    *sample = f1_ex + f2_ex;
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
                self.update_waveshapers_data();
            }
        }

        ProcessStatus::Normal
    }
}

impl Penare {
    fn update_waveshapers_data(&mut self) {
        let waveshapers_data = self.waveshapers_data.lock().unwrap();
        waveshapers_data.set_input_gain(self.params.input_gain.smoothed.next());
        waveshapers_data.set_output_gain(self.params.output_gain.smoothed.next());
        waveshapers_data.set_pos_function_type(self.params.pos_function_type.value());
        waveshapers_data.set_pos_function_param(self.params.pos_function_param.smoothed.next());
        waveshapers_data.set_neg_function_type(self.params.neg_function_type.value());
        waveshapers_data.set_neg_function_param(self.params.neg_function_param.smoothed.next());
        waveshapers_data.set_clip(self.params.output_clip.value());
        waveshapers_data.set_clip_threshold(self.params.output_clip_threshold.smoothed.next());
    }

    fn update_fs(&mut self) {
        if self.params.f1_freq.smoothed.is_smoothing()
        || self.params.f1_q.smoothed.is_smoothing()
        || self.f1[0].filter_type != self.params.f1_type.value() {
            self.update_f1();
        }
        if self.params.f2_freq.smoothed.is_smoothing()
        || self.params.f2_q.smoothed.is_smoothing()
        || self.f2[0].filter_type != self.params.f2_type.value() {
            self.update_f2();
        }
    }

    fn f1_process(&mut self, channel_index: usize, sample: f32) -> (f32, f32) {
        self.f1[channel_index].process(sample)
    }

    fn f2_process(&mut self, channel_index: usize, sample: f32) -> (f32, f32) {
        self.f2[channel_index].process(sample)
    }

    fn update_f1(&mut self) {
        let ty = self.params.f1_type.value();
        let freq = self.params.f1_freq.smoothed.next();
        let q = self.params.f1_q.smoothed.next();
        for filter in &mut self.f1 {
            filter.filter_type = ty;
            filter.freq = freq;
            filter.q = q;
            filter.calculate_coeff();
        }
    }

    fn update_f2(&mut self) {
        let ty = self.params.f2_type.value();
        let freq = self.params.f2_freq.smoothed.next();
        let q = self.params.f2_q.smoothed.next();
        for filter in &mut self.f2 {
            filter.filter_type = ty;
            filter.freq = freq;
            filter.q = q;
            filter.calculate_coeff();
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
