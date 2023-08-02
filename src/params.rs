use crate::{waveshaper, rectify, editor};
use std::sync::Arc;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

const MAX_FREQ: f32 = 22000.0;
const MIN_FREQ: f32 = 5.0;

#[derive(Params)]
pub struct PenareParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,
    // The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    // these IDs remain constant, you can rename and reorder these fields as you wish. The
    // parameters are exposed to the host in the same order they were defined.

    // ──────────────────────────────
    // Mix
    // ──────────────────────────────

    /// Mix between dry and wet signal
    #[id = "mix"]
    pub mix: FloatParam,
    /// (Hard) clip the final output (after everything)
    /// Essentially turning some of the clipping types into distortion
    #[id = "output-clip"]
    pub output_clip: BoolParam,
    /// Final clip threshold
    #[id = "output-clip-threshold"]
    pub output_clip_threshold: FloatParam,

    // ──────────────────────────────
    // Waveshaper
    // ──────────────────────────────

    /// Pre gain before waveshaping in decibels
    #[id = "pre-gain"]
    pub pre_gain: FloatParam,
    /// Mix between dry and wet signal (excluding gain)
    #[id = "function-mix"]
    pub function_mix: FloatParam,
    /// Function type to use in waveshaper
    #[id = "clip-type"]
    pub function_type: EnumParam<waveshaper::FunctionType>,
    /// Wave shaper parameter
    #[id = "function-param"]
    pub function_param: FloatParam,
    /// Post gain after waveshaping
    #[id = "post-gain"]
    pub post_gain: FloatParam,

    // ──────────────────────────────
    // Rectify
    // ──────────────────────────────

    /// Rectify the signal
    #[id = "rectify"]
    pub rectify: BoolParam,
    /// Rectify mix
    #[id = "rectify-mix"]
    pub rectify_mix: FloatParam,
    /// Mix in rectified signal
    #[id = "rectify-mix-in"]
    pub rectify_mix_in: FloatParam,
    /// Rectify type
    #[id = "rectify-type"]
    pub rectify_type: EnumParam<rectify::RectifyType>,
    /// Filp rectified signal
    #[id = "rectify-flip"]
    pub rectify_flip: BoolParam,

    // ──────────────────────────────
    // Floorer
    // ──────────────────────────────

    /// Floor the signal
    #[id = "floor"]
    pub floor: BoolParam,
    /// Floor mix
    #[id = "floor-mix"]
    pub floor_mix: FloatParam,
    /// Floor mix in
    #[id = "floor-mix-in"]
    pub floor_mix_in: FloatParam,
    /// Floor step size
    #[id = "floor-step"]
    pub floor_step: FloatParam,

    // ──────────────────────────────
    // Filter
    // ──────────────────────────────

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

            mix:                   percentage!("Mix", 1.0),
            output_clip:           BoolParam::new("Output Clip", true),
            output_clip_threshold: db!("Output Clip Threshold", 30.0),

            pre_gain:       db!("Pre Gain", 30.0),
            function_mix:   percentage!("Function Mix", 1.0),
            function_type:  EnumParam::new("Function Type", waveshaper::FunctionType::HardClip),
            function_param: db!("Function Parameter", 30.0),
            post_gain:      db!("Post Gain", 30.0),

            rectify:        BoolParam::new("Rectify", false),
            rectify_mix:    percentage!("Rectify Mix", 0.0),
            rectify_mix_in: percentage!("Rectify Mix In", 1.0),
            rectify_type:   EnumParam::new("Rectify Type", rectify::RectifyType::HalfWave),
            rectify_flip:   BoolParam::new("Rectify Flip", false),

            floor:        BoolParam::new("Floor", false),
            floor_mix:    percentage!("Floor Mix", 1.0),
            floor_mix_in: percentage!("Floor Mix In", 0.0),
            floor_step:   FloatParam::new(
                "Floor Step",
                3.0,
                FloatRange::Linear { min: 0.0, max: 20.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            excess_mix:    percentage!("Excess Mix", 0.0),
            low_pass:      hz!("Low Pass", MAX_FREQ),
            low_pass_q:    q!("Low Pass Q", 2.0f32.sqrt() / 2.0),
            high_pass:     hz!("High Pass", MIN_FREQ),
            high_pass_q:   q!("High Pass Q", 2.0f32.sqrt() / 2.0),
            excess_bypass: BoolParam::new("Excess Bypass", false),
        }
    }
}