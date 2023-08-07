use crate::fxs::utils::hard_clip;
use std::f32::consts::PI;
use nih_plug::{prelude::*, util::gain_to_db};

#[derive(Clone, Copy, Enum, PartialEq)]
pub enum FunctionType {
    // Classic hard clip
    HardClip = 0,
    // "Scaled" clipping (t * x)
    ScaledClip,
    // tanh(2x) * t
    TwoTanh,
    // sign(x) * sqrt(|x|) * t
    Sqrt,
    // I don't know what is this
    // 2 * sign(x) * (t - (t / 1 + |x|))
    Reciprocal,
    // Reciprocal but tanh :))
    // 2 * sign(x) * tanh(t - (t / 1 + |x|))
    ReciprocalTanh,
    // tanh(2atanh(2x)) * t
    TanhTwoAtanh,
    // (1 - t) * x + t * sin(2pi * x * (1 + 3t))
    Sinusoidal,
    // sign(x) * {|x| > t : -|x| + 2t, |x|}
    Singlefold,
    // sign(x) * (4|x| / T) * |((|x| - T/4) % T) - T/2|
    Sillyfold,
    // What the actual hell?
    // sign(x) * {|x| > sin(2pi * |x| + (1+3t)): sin(|x| * t), |x| }
    BrokenSin,
    // Bitcrushers
    Floor,
    Round,
    Bitcrush,
}

impl FunctionType {
    pub fn apply(&self, x: f32, a: f32) -> f32 {
        use FunctionType::*;
        let sig = x.signum();
        let xa = x.abs();
        match self {
            HardClip       => hard_clip(x, a),
            ScaledClip     => hard_clip(x * a, a),
            TwoTanh        => (2.0 * x).tanh() * a,
            Sqrt           => sig * (xa.sqrt() * a),
            Reciprocal     => 2.0 * sig * (a - (a / (xa + 1.0))),
            ReciprocalTanh => 2.0 * sig * ((a - (a / (xa + 1.0))).tanh()),
            TanhTwoAtanh => sig * match xa {
                x if x < a / 2.0 => (2.0 * (2.0 * x / a).atanh()).tanh() * a,
                _ => a,
            },
            Sinusoidal => {
                // Normalize
                let ab = gain_to_db(a.abs()) / 30.0;
                let w1 = (2.0 * PI * x * (1.0 + 3.0 * ab)).sin();
                let w2 = (1.0 - ab) * x + ab * w1;
                let w3 = (1.0 - 0.3 * ab) * w2;
                w3
            },
            Singlefold => sig * match xa {
                x if x > a => -xa + 2.0 * (a.abs()),
                _          => xa,
            },
            Sillyfold => sig * 4.0 * xa / a * (((xa - a * 0.25) % a) - a * 0.5).abs(),
            BrokenSin => sig * match xa {
                x if x > (2.0 * PI * x + (1.0 + 3.0 * a)).sin() => (x * a).sin(),
                _ => xa
            },
            Floor => sig * ((x * sig * a.abs()).floor() / a).abs(),
            Round => sig * ((x * sig * a.abs()).round() / a).abs(),
            Bitcrush => {
                let b = 2f32.powf(-a);
                b * (x / b).round()
            },
        }
    }
}

impl From<usize> for FunctionType {
    fn from(id: usize) -> FunctionType {
        Self::from_index(id)
    }
}

impl From<FunctionType> for usize {
    fn from(t: FunctionType) -> usize {
        t as usize
    }
}

impl std::fmt::Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionType::HardClip       => write!(f, "HardClip"),
            FunctionType::ScaledClip     => write!(f, "ScaledClip"),
            FunctionType::TwoTanh        => write!(f, "2Tanh"),
            FunctionType::Sqrt           => write!(f, "Sqrt"),
            FunctionType::Reciprocal     => write!(f, "Reciprocal"),
            FunctionType::ReciprocalTanh => write!(f, "ReciprocalTanh"),
            FunctionType::TanhTwoAtanh   => write!(f, "Tanh2Atanh"),
            FunctionType::Sinusoidal     => write!(f, "Sinusoidal"),
            FunctionType::Singlefold     => write!(f, "Singlefold"),
            FunctionType::Sillyfold      => write!(f, "Sillyfold"),
            FunctionType::BrokenSin      => write!(f, "BrokenSin"),
            FunctionType::Floor          => write!(f, "Floor"),
            FunctionType::Round          => write!(f, "Round"),
            FunctionType::Bitcrush       => write!(f, "Bitcrush"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FunctionType::*;

    #[test]
    #[allow(unused_variables)]
    fn test() {
        let sample = (5..15).map(|x| x as f32 * 0.05).collect::<Vec<_>>();
        let a = 0.5;

        let hard = sample.iter().map(|x| HardClip.apply(*x, a)).collect::<Vec<_>>();
        let test = sample.iter().map(|x| ScaledClip.apply(*x, a)).collect::<Vec<_>>();

        fn fmt(v: &[f32]) -> String {
            v.iter().map(|x| format!("{:6.2}", x)).collect::<Vec<_>>().join("")
        }

        println!("     {}", fmt(&sample));
        println!("Hard {}", fmt(&hard));
        println!("Test {}", fmt(&test));
    }
}