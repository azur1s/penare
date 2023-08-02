use nih_plug::prelude::*;

#[derive(Clone, Copy, Enum, PartialEq)]
/// Fig 4.36 in DAFX
pub enum RectifyType {
    HalfWave,
    FullWave,
    // TODO: Octave
}

impl RectifyType {
    pub fn apply(&self, x: f32) -> f32 {
        match self {
            RectifyType::HalfWave => x.max(0.0),
            RectifyType::FullWave => x.abs(),
        }
    }
}