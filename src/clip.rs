use nih_plug::prelude::*;

#[derive(Clone, Copy, Enum, PartialEq)]
pub enum ClipType {
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

impl ClipType {
    pub fn apply(&self, x: f32, threshold: f32) -> f32 {
        let sig = x.signum();
        let xa = x.abs();
        match self {
            ClipType::Hard       => x.min(threshold).max(-threshold),
            ClipType::Scaled     => x * threshold,
            ClipType::TwoTanh    => (2.0 * x).tanh() * threshold,
            ClipType::Reciprocal => 2.0 * sig * (threshold - (threshold / (xa + 1.0))),
            ClipType::Softdrive  => sig * match xa {
                x if x <= 1.0 / 3.0 * threshold => 2.0 * x,
                x if x <= 2.0 / 3.0 * threshold => (3.0 - (2.0 - 3.0 * x).powi(2)) / 3.0,
                _ => threshold,
            },
            ClipType::InvTwoTanh => sig * match xa {
                x if x < threshold / 2.0 => (2.0 * (2.0 * xa / threshold).atanh()).tanh() * threshold,
                _ => threshold,
            }
        }
    }
}

impl std::fmt::Display for ClipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipType::Hard        => write!(f, "Hard"),
            ClipType::Scaled      => write!(f, "Scaled"),
            ClipType::TwoTanh     => write!(f, "2Tanh"),
            ClipType::Reciprocal  => write!(f, "Reciprocal"),
            ClipType::Softdrive   => write!(f, "Softdrive"),
            ClipType::InvTwoTanh  => write!(f, "Inv2Tanh"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClipType::*;

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