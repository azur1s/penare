pub fn hard_clip(x: f32, threshold: f32) -> f32 {
    x.min(threshold).max(-threshold)
}

/// Mix between two values.
/// 0.0 = a, 1.0 = b
pub fn mix_between(a: f32, b: f32, mix: f32) -> f32 {
    a * (1.0 - mix) + b * mix
}

pub fn mix_in(a: f32, b: f32, mix: f32) -> f32 {
    a + b * mix
}