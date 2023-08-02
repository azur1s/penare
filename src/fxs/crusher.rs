use nih_plug::prelude::*;

#[derive(Clone, Copy, Enum, PartialEq)]
pub enum CrushType {
    Floor,
    Round,
    Bits,
    BitsQ,
    BitsQU16,
}

impl CrushType {
    pub fn apply(&self, x: f32, a: f32) -> f32 {
        match self {
            CrushType::Floor => x.signum() * ((x * x.signum() * a.abs()).floor() / a).abs(),
            CrushType::Round => x.signum() * ((x * x.signum() * a.abs()).round() / a).abs(),
            CrushType::Bits  => {
                let b = 2f32.powf(-a);
                b * (x / b).round()
            }
            CrushType::BitsQ => {
                let b = 2f32.powf(-a);
                let q = 0xFFFFF as f32; // 0xFFFFF = 1048575
                b * (x / b).round() + q - q
            },
            CrushType::BitsQU16 => {
                let b = 2f32.powf(-a);
                let q = 0xFFFF as f32; // 0xFFFF = 65535
                b * (x / b).round() + q - q
            }
        }
    }
}

#[test]
fn test_quantize() {
    let xs = vec![0.0, 0.1, 0.2, 0.5, 0.7, 0.9, 1.0];
    let os = xs.iter()
        .map(|x| CrushType::BitsQU16.apply(*x, 1.2))
        .collect::<Vec<f32>>();
    assert_ne!(xs, os);
}