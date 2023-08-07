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

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
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
            //     │               ├─(Dry Signal)
            //   Filter ───────┐   │
            //     │           │   │
            //  Pre-Gain       ├─(Excess Signal)
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
                let (ft, fp, fm) = if match (should_copy.is_on(), should_copy.is_positive(), *sample >= 0.0) {
                    // If copy is on and positive is selected
                    (true,  true,  _   ) => true,
                    // If copy is on and negative is selected
                    (true,  false, _   ) => false,
                    // If copy is off and phase is positive
                    (false, _,    true ) => true,
                    // If copy is off and phase is negative
                    (false, _,    false) => false,
                } {
                    (self.params.pos_function_type.value(),
                    self.params.pos_function_param.smoothed.next(),
                    self.params.pos_function_mix.smoothed.next())
                } else {
                    (self.params.neg_function_type.value(),
                    self.params.neg_function_param.smoothed.next(),
                    self.params.neg_function_mix.smoothed.next())
                };
                let wss = mix_between(
                    *sample,
                    ft.apply(*sample, fp),
                    fm,
                );
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
    /// Update waveshapers data to be sent to the UI
    fn update_waveshapers_data(&mut self) {
        let waveshapers_data = self.waveshapers_data.lock().unwrap();
        waveshapers_data.set_input_gain(self.params.input_gain.smoothed.next());
        waveshapers_data.set_output_gain(self.params.output_gain.smoothed.next());
        waveshapers_data.set_function_types(
            self.params.pos_function_type.value(),
            self.params.neg_function_type.value()
        );
        waveshapers_data.set_function_params(
            self.params.pos_function_param.smoothed.next(),
            self.params.neg_function_param.smoothed.next()
        );
        waveshapers_data.set_function_mixs(
            self.params.pos_function_mix.smoothed.next(),
            self.params.neg_function_mix.smoothed.next()
        );
        waveshapers_data.set_clip(self.params.output_clip.value());
        waveshapers_data.set_clip_threshold(self.params.output_clip_threshold.smoothed.next());
        waveshapers_data.set_copy(self.params.copy_function.value());
        waveshapers_data.set_flip(self.params.flip.value());
    }

    /// Update filters (when parameters change)
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

    /// Process a sample through the first filter, returning the filtered sample and the excess
    fn f1_process(&mut self, channel_index: usize, sample: f32) -> (f32, f32) {
        self.f1[channel_index].process(sample)
    }

    /// Process a sample through the second filter, returning the filtered sample and the excess
    fn f2_process(&mut self, channel_index: usize, sample: f32) -> (f32, f32) {
        self.f2[channel_index].process(sample)
    }

    /// Update first filter with current parameters
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

    /// Update second filter with current parameters
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
