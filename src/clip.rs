use nih_plug::prelude::*;

#[derive(Clone, Copy, Enum, PartialEq)]
pub enum ClipType {
    Hard,
    TwoTanh,
    Reciprocal,
}

impl ClipType {
    pub fn apply(&self, x: f32, threshold: f32) -> f32 {
        let sig = x.signum();
        match self {
            ClipType::Hard        => x,
            ClipType::TwoTanh     => (2.0 * x).tanh() * threshold,
            ClipType::Reciprocal  => 2.0 * sig * (threshold - (threshold / (x.abs() + 1.0))),
        }
        .max(-threshold).min(threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::ClipType::*;

    #[test]
    fn test() {
        let sample = (-10..10).map(|x| x as f32 * 0.1).collect::<Vec<_>>();
        let threshold = 0.5;

        let hard = sample.iter().map(|x| Hard.apply(*x, threshold)).collect::<Vec<_>>();
        let ttanh = sample.iter().map(|x| TwoTanh.apply(*x, threshold)).collect::<Vec<_>>();
        let recip = sample.iter().map(|x| Reciprocal.apply(*x, threshold)).collect::<Vec<_>>();

        fn fmt(v: &[f32]) -> String {
            v.iter().map(|x| format!("{:6.2}", x)).collect::<Vec<_>>().join("")
        }

        println!("Hard  {}", fmt(&hard));
        println!("2Tanh {}", fmt(&ttanh));
        println!("Recip {}", fmt(&recip));
    }
}

impl std::fmt::Display for ClipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipType::Hard        => write!(f, "Hard"),
            ClipType::TwoTanh     => write!(f, "2Tanh"),
            ClipType::Reciprocal  => write!(f, "Reciprocal"),
        }
    }
}