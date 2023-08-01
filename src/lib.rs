use std::sync::Arc;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

mod clip;
mod filter;
mod editor;

const MAX_FREQ: f32 = 22000.0;
const MIN_FREQ: f32 = 5.0;

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

#[derive(Params)]
struct PenareParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    // The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    // these IDs remain constant, you can rename and reorder these fields as you wish. The
    // parameters are exposed to the host in the same order they were defined.

    /// Mix between dry and wet signal
    #[id = "mix"]
    pub mix: FloatParam,
    /// Clipping types
    #[id = "clip-type"]
    pub clip_type: EnumParam<clip::ClipType>,

    /// Pre gain before clip parameter in decibels
    #[id = "pre-gain"]
    pub pre_gain: FloatParam,
    /// Clip Threshold in decibels
    #[id = "clip"]
    pub threshold: FloatParam,
    /// Post gain after clip parameter in decibels
    #[id = "post-gain"]
    pub post_gain: FloatParam,

    /// Mix excess signal back into the input
    #[id = "excess-mix"]
    pub excess_mix: FloatParam,
    /// Low pass on the clipper (where the clipper should start clipping)
    #[id = "low-pass"]
    pub low_pass: FloatParam,
    #[id = "low-pass-q"]
    pub low_pass_q: FloatParam,
    /// High pass on the clipper (where the clipper should stop clipping)
    #[id = "high-pass"]
    pub high_pass: FloatParam,
    #[id = "high-pass-q"]
    pub high_pass_q: FloatParam,
    /// Excess signal bypass
    #[id = "unfiltered"]
    pub excess_bypass: BoolParam,
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

impl Default for PenareParams {
    fn default() -> Self {
        macro_rules! db {
            ($name:expr, $range:expr) => {
                FloatParam::new(
                    $name,
                    util::db_to_gain(0.0),
                    FloatRange::Skewed {
                        min: util::db_to_gain(-$range),
                        max: util::db_to_gain($range),
                        factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                    },
                )
                .with_smoother(SmoothingStyle::Logarithmic(50.0))
                .with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
            }
        }
        macro_rules! hz {
            ($name:expr, $default:expr) => {
                FloatParam::new(
                    $name,
                    $default,
                    FloatRange::Skewed {
                        min: MIN_FREQ,
                        max: MAX_FREQ,
                        factor: FloatRange::skew_factor(-1.0),
                    },
                )
                .with_smoother(SmoothingStyle::Logarithmic(100.0))
                .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
                .with_string_to_value(formatters::s2v_f32_hz_then_khz())
            }
        }
        macro_rules! q {
            ($name:expr, $default:expr) => {
                FloatParam::new(
                    $name,
                    $default,
                    FloatRange::Skewed {
                        min: 2.0f32.sqrt() / 2.0,
                        max: 10.0,
                        factor: FloatRange::skew_factor(-1.0),
                    },
                )
                .with_smoother(SmoothingStyle::Logarithmic(100.0))
                .with_value_to_string(formatters::v2s_f32_rounded(2))
            }
        }
        macro_rules! percentage {
            ($name:expr, $default:expr) => {
                FloatParam::new(
                    $name,
                    $default,
                    FloatRange::Linear {
                        min: 0.0,
                        max: 1.0,
                    },
                )
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage())
            }
        }

        Self {
            editor_state: editor::default_state(),

            mix: percentage!("Mix", 1.0),
            clip_type: EnumParam::new("Clip Type", clip::ClipType::Hard),
            pre_gain: db!("Pre Gain", 30.0),
            threshold: db!("Threshold", 30.0),
            post_gain: db!("Post Gain", 30.0),

            excess_mix: percentage!("Excess Mix", 0.0),
            low_pass: hz!("Low Pass", 1000.0),
            low_pass_q: q!("Low Pass Q", 2.0f32.sqrt() / 2.0),
            high_pass: hz!("High Pass", 400.0),
            high_pass_q: q!("High Pass Q", 2.0f32.sqrt() / 2.0),
            excess_bypass: BoolParam::new("Excess Bypass", false),
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
            let clip_type = self.params.clip_type.value();
            let pre_gain = self.params.pre_gain.smoothed.next();
            let threshold = self.params.threshold.smoothed.next();
            let post_gain = self.params.post_gain.smoothed.next();

            let excess_mix = self.params.excess_mix.smoothed.next();
            let excess_bypass = self.params.excess_bypass.value();
            self.filter_update();

            for (channel_idx, sample) in channel_samples.into_iter().enumerate() {
                // Apply low-pass filter
                let (s, lp_ex) = self.low_pass(channel_idx, *sample);
                // Apply high-pass filter
                let (s, hp_ex) = self.high_pass(channel_idx, s);
                *sample = s;

                let dry = *sample;
                // Apply pre-gain
                *sample *= pre_gain;
                // Clip
                *sample = clip_type.apply(*sample, threshold);
                // Apply post-gain
                *sample *= post_gain;
                // Mix between dry and wet
                *sample =
                    mix * *sample // Wet signal
                    + (1.0 - mix) * dry; // Dry signal
                // Calculate amplitude (for peak meter)
                amplitude += *sample;

                // Filter mix
                if !excess_bypass {
                    // Mix between filtered and unfiltered signal
                    // 100% = Processed only
                    // 0%   = Processed + Filter excesses
                    *sample =
                        *sample // Processed signal
                        // Excess signal
                        + excess_mix * lp_ex
                        + excess_mix * hp_ex;
                } else {
                    // Excess signal only
                    *sample = lp_ex + hp_ex;
                }
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
