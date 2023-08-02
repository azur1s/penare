// Based off of nih-plug's Diopser filter
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/diopser/src/filter.rs

#[derive(Clone, Copy, Debug)]
pub struct Biquad {
    pub coeff: Coeff,
    s1: f32,
    s2: f32,
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            coeff: Coeff::id(),
            s1: 0.0,
            s2: 0.0,
        }
    }
}

impl Biquad {
    pub fn process(&mut self, sample: f32) -> f32 {
        let result = self.coeff.b0 * sample + self.s1;
        self.s1 = self.coeff.b1 * sample - self.coeff.a1 * result + self.s2;
        self.s2 = self.coeff.b2 * sample - self.coeff.a2 * result;

        result
    }

    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Coeff {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl Coeff {
    pub fn id() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    pub fn lowpass(sample_rate: f32, freq: f32, q: f32) -> Self {
        let o0 = std::f32::consts::TAU * (freq / sample_rate);
        let cos_o0 = o0.cos();
        let a = o0.sin() / (2.0 * q);

        let a0 = 1.0 + a;
        let b0 = ((1.0 - cos_o0) / 2.0) / a0;
        let b1 = (1.0 - cos_o0) / a0;
        let b2 = ((1.0 - cos_o0) / 2.0) / a0;
        let a1 = (-2.0 * cos_o0) / a0;
        let a2 = (1.0 - a) / a0;

        Self { b0, b1, b2, a1, a2 }
    }

    pub fn highpass(sample_rate: f32, freq: f32, q: f32) -> Self {
        let o0 = std::f32::consts::TAU * (freq / sample_rate);
        let cos_o0 = o0.cos();
        let a = o0.sin() / (2.0 * q);

        let a0 = 1.0 + a;
        let b0 = ((1.0 + cos_o0) / 2.0) / a0;
        let b1 = -(1.0 + cos_o0) / a0;
        let b2 = ((1.0 + cos_o0) / 2.0) / a0;
        let a1 = (-2.0 * cos_o0) / a0;
        let a2 = (1.0 - a) / a0;

        Self { b0, b1, b2, a1, a2 }
    }
}