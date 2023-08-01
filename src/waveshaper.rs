use nih_plug::prelude::*;

#[derive(Clone, Copy, Enum, PartialEq)]
pub enum FunctionType {
    // Classic hard clip
    Hard,
    // "Scaled" clipping (t * x)
    Scaled,
    // tanh(2x) * t
    TwoTanh,
    // I don't know what is this
    Reciprocal,
    // Fig. 4.14 in DAFX
    // Probably best used as distortion
    Softdrive,
    // tanh(2atanh(2x)) * t
    TanhTwoAtanh,
}

impl FunctionType {
    pub fn apply(&self, x: f32, a: f32) -> f32 {
        use FunctionType::*;
        let sig = x.signum();
        let xa = x.abs();
        let c = |x: f32| x.min(a).max(-a);
        match self {
            Hard       => c(x),
            Scaled     => c(x * a),
            TwoTanh    => (2.0 * x).tanh() * a,
            Reciprocal => 2.0 * sig * (a - (a / (xa + 1.0))),
            // Softdrive generate some white noise when clipping
            // I don't know if it's a correct behavior or not
            Softdrive  => sig * match xa {
                x if x <= 1.0 / 3.0 * a => 2.0 * x,
                x if x <= 2.0 / 3.0 * a => (3.0 - (2.0 - 3.0 * x).powi(2)) / 3.0,
                _ => a,
            },
            TanhTwoAtanh => sig * match xa {
                x if x < a / 2.0 => (2.0 * (2.0 * x / a).atanh()).tanh() * a,
                _ => a,
            },
        }
    }
}

impl std::fmt::Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionType::Hard         => write!(f, "Hard"),
            FunctionType::Scaled       => write!(f, "Scaled"),
            FunctionType::TwoTanh      => write!(f, "2Tanh"),
            FunctionType::Reciprocal   => write!(f, "Reciprocal"),
            FunctionType::Softdrive    => write!(f, "Softdrive"),
            FunctionType::TanhTwoAtanh => write!(f, "Tanh2Atanh"),
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

        let hard = sample.iter().map(|x| Hard.apply(*x, a)).collect::<Vec<_>>();
        let test = sample.iter().map(|x| Scaled.apply(*x, a)).collect::<Vec<_>>();

        fn fmt(v: &[f32]) -> String {
            v.iter().map(|x| format!("{:6.2}", x)).collect::<Vec<_>>().join("")
        }

        println!("     {}", fmt(&sample));
        println!("Hard {}", fmt(&hard));
        println!("Test {}", fmt(&test));
    }
}