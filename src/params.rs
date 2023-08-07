use crate::{
    fxs::{waveshaper, filter},
    editor,
};
use std::sync::Arc;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

const MAX_FREQ: f32 = 22000.0;
const MIN_FREQ: f32 = 3.0;

/// A tri-state enum type
#[derive(Enum, PartialEq)]
pub enum TriState { Off, Pos, Neg }

impl TriState {
    pub fn is_off(&self)      -> bool { matches!(self, TriState::Off) }
    pub fn is_on(&self)       -> bool { !self.is_off() }
    pub fn is_positive(&self) -> bool { matches!(self, TriState::Pos) }
}

impl From<usize> for TriState {
    fn from(id: usize) -> Self {
        Self::from_index(id)
    }
}

impl From<TriState> for usize {
    fn from(tri: TriState) -> Self {
        tri as usize
    }
}

impl std::fmt::Display for TriState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriState::Off => write!(f, "Off"),
            TriState::Pos => write!(f, "Positive"),
            TriState::Neg => write!(f, "Negative"),
        }
    }
}

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
    /// Input gain before effects
    #[id = "input-gain"]
    pub input_gain: FloatParam,
    /// Output gain after effects
    #[id = "output-gain"]
    pub output_gain: FloatParam,

    // ──────────────────────────────
    // Waveshaper
    // ──────────────────────────────

    /// Mix between dry and wet signal (excluding gain)
    #[id = "function-mix"]
    pub function_mix: FloatParam,
    /// Function type to apply to positive shape
    #[id = "pos-function-type"]
    pub pos_function_type: EnumParam<waveshaper::FunctionType>,
    /// Function parameter to use in positive shape function
    #[id = "pos-function-param"]
    pub pos_function_param: FloatParam,
    /// Mix
    #[id = "pos-function-mix"]
    pub pos_function_mix: FloatParam,
    /// Function type to apply to negative shape
    #[id = "neg-function-type"]
    pub neg_function_type: EnumParam<waveshaper::FunctionType>,
    /// Function parameter to use in negative shape function
    #[id = "neg-function-param"]
    pub neg_function_param: FloatParam,
    /// Mix
    #[id = "neg-function-mix"]
    pub neg_function_mix: FloatParam,
    /// Use function for the positive/negative shape too
    #[id = "copy-function"]
    pub copy_function: EnumParam<TriState>,
    /// Flip the waveshaped signal
    #[id = "flip"]
    pub flip: BoolParam,

    // ──────────────────────────────
    // Filter
    // ──────────────────────────────

    /// Mix excess signal back into the input
    #[id = "excess-mix"]
    pub excess_mix: FloatParam,
    /// Filter 1 type
    #[id = "f1-type"]
    pub f1_type: EnumParam<filter::FilterType>,
    /// Filter 1 frequency
    #[id = "f1-freq"]
    pub f1_freq: FloatParam,
    /// Filter 1 Q
    #[id = "f1-q"]
    pub f1_q: FloatParam,
    /// Filter 2 type
    #[id = "f2-type"]
    pub f2_type: EnumParam<filter::FilterType>,
    /// Filter 2 frequency
    #[id = "f2-freq"]
    pub f2_freq: FloatParam,
    /// Filter 2 Q
    #[id = "f2-q"]
    pub f2_q: FloatParam,
    /// Excess signal bypass
    #[id = "excess-bypass"]
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
            input_gain:            db!("Pre Gain", 30.0),
            output_gain:           db!("Output Gain", 30.0),

            function_mix:       percentage!("Function Mix", 1.0),
            pos_function_type:  EnumParam::new("Positive Function Type", waveshaper::FunctionType::HardClip),
            pos_function_param: db!("Positive Function Parameter", 30.0),
            pos_function_mix:   percentage!("Positive Function Mix", 1.0),
            neg_function_type:  EnumParam::new("Negative Function Type", waveshaper::FunctionType::HardClip),
            neg_function_param: db!("Negative Function Parameter", 30.0),
            neg_function_mix:   percentage!("Negative Function Mix", 1.0),
            copy_function:      EnumParam::new("Copy Function", TriState::Off),
            flip:               BoolParam::new("Flip", false),

            excess_mix:    percentage!("Excess Mix", 0.0),
            f1_type:       EnumParam::new("Filter 1 Type", filter::FilterType::Lowpass),
            f1_freq:       hz!("Filter 1 Freq", MAX_FREQ),
            f1_q:          q!("Filter 1 Q", 2.0f32.sqrt() / 2.0),
            f2_type:       EnumParam::new("Filter 2 Type", filter::FilterType::Highpass),
            f2_freq:       hz!("Filter 2 Freq", MIN_FREQ),
            f2_q:          q!("Filter 2 Q", 2.0f32.sqrt() / 2.0),
            excess_bypass: BoolParam::new("Excess Bypass", false),
        }
    }
}