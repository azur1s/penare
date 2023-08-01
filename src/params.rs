use crate::{clip, editor};
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

    /// Mix between dry and wet signal
    #[id = "mix"]
    pub mix: FloatParam,
    /// Clipping types
    #[id = "clip-type"]
    pub clip_type: EnumParam<clip::ClipType>,
    /// (Hard) clip the final output (after everything)
    /// Essentially turning some of the clipping types into distortion
    #[id = "clip-output"]
    pub clip_output: BoolParam,
    /// Use 1.0 as threshold for final clipping
    #[id = "clip-output-value"]
    pub clip_output_value: BoolParam, // true = 1.0, false = threshold

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
            clip_output: BoolParam::new("Clip Output", true),
            clip_output_value: BoolParam::new("Clip Output Value", true),

            excess_mix: percentage!("Excess Mix", 0.0),
            low_pass: hz!("Low Pass", 1000.0),
            low_pass_q: q!("Low Pass Q", 2.0f32.sqrt() / 2.0),
            high_pass: hz!("High Pass", 400.0),
            high_pass_q: q!("High Pass Q", 2.0f32.sqrt() / 2.0),
            excess_bypass: BoolParam::new("Excess Bypass", false),
        }
    }
}