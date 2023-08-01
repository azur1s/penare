use std::sync::Arc;
use nih_plug::prelude::*;

mod params;
mod waveshaper;
mod rectify;
mod filter;
mod editor;
mod utils;

use params::PenareParams;
use utils::{mix_between, mix_in};

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started
struct Penare {
    params: Arc<PenareParams>,
    sample_rate: f32,
    /// Needed to normalize the peak meter's response based on the sample rate.
    peak_meter_decay_weight: f32,
    /// The current data for the peak meter. This is stored as an [`Arc`] so we can share it between
    /// the GUI and the audio processing parts. If you have more state to share, then it's a good
    /// idea to put all of that in a struct behind a single `Arc`.
    ///
    /// This is stored as voltage gain.
    peak_meter: Arc<AtomicF32>,
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
            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            lp: [filter::Biquad::default(); 2],
            lp_invert: [filter::Biquad::default(); 2],
            hp: [filter::Biquad::default(); 2],
            hp_invert: [filter::Biquad::default(); 2],
        }
    }
}

impl Plugin for Penare {
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
            self.peak_meter.clone(),
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
        let peak_meter_decay_ms = 150.0;
        // After `peak_meter_decay_ms` milliseconds of pure silence, the peak meter's value should
        // have dropped by 12 dB
        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * peak_meter_decay_ms / 1000.0).recip())
            as f32;

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
            let mut amplitude = 0.0;
            let num_samples = channel_samples.len();

            let mix = self.params.mix.smoothed.next();
            let clip_output = self.params.clip_output.value();
            let clip_output_value = self.params.clip_output_value.value();

            let pre_gain = self.params.pre_gain.smoothed.next();
            let function_mix = self.params.function_mix.smoothed.next();
            let function_type = self.params.function_type.value();
            let function_param = self.params.function_param.smoothed.next();
            let post_gain = self.params.post_gain.smoothed.next();

            let rectify = self.params.rectify.value();
            let rectify_mix = self.params.rectify_mix.smoothed.next();
            let rectify_mix_in = self.params.rectify_mix_in.smoothed.next();
            let rectify_type = self.params.rectify_type.value();

            let excess_mix = self.params.excess_mix.smoothed.next();
            let excess_bypass = self.params.excess_bypass.value();
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
            // Waveshape       │   │
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
                *sample *= pre_gain; // Pre-gain

                // --- Distortions ---
                // Rectify
                if rectify {
                    let rs = rectify_type.apply(*sample); // Rectified signal
                    *sample = mix_between(*sample, rs, rectify_mix);
                    *sample = mix_in(*sample, rs, rectify_mix_in);
                }

                // --- Waveshaper ---
                let wss = function_type.apply(*sample, function_param); // Wave shaped signal
                *sample = mix_between(*sample, wss, function_mix);

                // --- Post-Gain ---
                *sample *= post_gain; // Post-gain

                // Filter mix
                if !excess_bypass {
                    // Mix in excess signal
                    *sample = mix_in(
                        *sample,
                        excess_mix * lp_ex + excess_mix * hp_ex,
                        excess_mix,
                    );
                } else {
                    // Excess signal only
                    *sample = lp_ex + hp_ex;
                }

                // Mix between dry and wet
                *sample = mix_between(dry, *sample, mix);

                // Final clip
                if clip_output {
                    *sample = waveshaper::FunctionType::Hard.apply(*sample, clip_output_value);
                }

                // Calculate amplitude (for peak meter)
                amplitude += *sample;
            }

            // Only calculate the UI-related data if the editor is open.
            if self.params.editor_state.is_open() {
                // Peak meter
                amplitude = (amplitude / num_samples as f32).abs();
                let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude > current_peak_meter {
                    amplitude
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed);
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
    const VST3_CLASS_ID: [u8; 16] = *b"Penare....Azur1s";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Penare);
nih_export_vst3!(Penare);
