use nih_plug::prelude::*;

#[derive(Clone, Copy, Enum, PartialEq)]
pub enum FunctionType {
    // Classic hard clip
    Hard,
    // "Scaled" clipping (t * x)
    Scaled,
    // I made this up
    TwoTanh,
    // I don't know what is this
    Reciprocal,
    // Fig. 4.14 in DAFX
    // Probably best used as distortion
    Softdrive,
    // tanh(2artanh(2x) * t)
    InvTwoTanh,
}

impl FunctionType {
    pub fn apply(&self, x: f32, threshold: f32) -> f32 {
        let sig = x.signum();
        let xa = x.abs();
        match self {
            FunctionType::Hard       => x.min(threshold).max(-threshold),
            FunctionType::Scaled     => x * threshold,
            FunctionType::TwoTanh    => (2.0 * x).tanh() * threshold,
            FunctionType::Reciprocal => 2.0 * sig * (threshold - (threshold / (xa + 1.0))),
            FunctionType::Softdrive  => sig * match xa {
                x if x <= 1.0 / 3.0 * threshold => 2.0 * x,
                x if x <= 2.0 / 3.0 * threshold => (3.0 - (2.0 - 3.0 * x).powi(2)) / 3.0,
                _ => threshold,
            },
            FunctionType::InvTwoTanh => sig * match xa {
                x if x < threshold / 2.0 => (2.0 * (2.0 * x / threshold).atanh()).tanh() * threshold,
                _ => threshold,
            }
        }
    }
}

impl std::fmt::Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionType::Hard        => write!(f, "Hard"),
            FunctionType::Scaled      => write!(f, "Scaled"),
            FunctionType::TwoTanh     => write!(f, "2Tanh"),
            FunctionType::Reciprocal  => write!(f, "Reciprocal"),
            FunctionType::Softdrive   => write!(f, "Softdrive"),
            FunctionType::InvTwoTanh  => write!(f, "Inv2Tanh"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FunctionType::*;

    #[test]
    #[allow(unused_variables)]
    fn test() {
        let sample = (5..25).map(|x| x as f32 * 0.02).collect::<Vec<_>>();
        let threshold = 0.4;

        let hard  = sample.iter().map(|x| Hard      .apply(*x, threshold)).collect::<Vec<_>>();
        let scld  = sample.iter().map(|x| Scaled    .apply(*x, threshold)).collect::<Vec<_>>();
        let ttanh = sample.iter().map(|x| TwoTanh   .apply(*x, threshold)).collect::<Vec<_>>();
        let recip = sample.iter().map(|x| Reciprocal.apply(*x, threshold)).collect::<Vec<_>>();
        let softd = sample.iter().map(|x| Softdrive .apply(*x, threshold)).collect::<Vec<_>>();
        let invth = sample.iter().map(|x| InvTwoTanh.apply(*x, threshold)).collect::<Vec<_>>();

        fn fmt(v: &[f32]) -> String {
            v.iter().map(|x| format!("{:6.2}", x)).collect::<Vec<_>>().join("")
        }

        println!("      {}", fmt(&sample));
        println!("Hard  {}", fmt(&hard));
        // println!("Scld  {}", fmt(&scld));
        // println!("2Tanh {}", fmt(&ttanh));
        // println!("Recip {}", fmt(&recip));
        println!("Softd {}", fmt(&softd));
        println!("InvTh {}", fmt(&invth));
    }
}