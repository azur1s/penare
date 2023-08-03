use std::f32::consts::PI;

// Based off of renzol2's fx projects
// https://github.com/renzol2/fx/
use nih_plug::prelude::*;

#[derive(Clone, Copy, Debug, Enum, PartialEq)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
}

#[derive(Clone, Copy, Debug)]
pub struct Biquad {
    pub filter_type: FilterType,
    // Coefficients
    a0: f32,
    a1: f32,
    a2: f32,
    b1: f32,
    b2: f32,

    // Filter parameters
    pub freq: f32,
    pub q: f32,
    pub gain: f32,

    // Unit delays
    s1: f32,
    s2: f32,

    // Sample rate
    pub sample_rate: f32,
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Lowpass,
            a0: 0.0,
            a1: 0.0,
            a2: 0.0,
            b1: 0.0,
            b2: 0.0,
            freq: 0.5,
            q: 0.707,
            gain: 0.0,
            s1: 0.0,
            s2: 0.0,
            sample_rate: 1.0,
        }
    }
}

impl Biquad {
    pub fn calculate_coeff(&mut self) {
        // let v = 10.0_f32.powf(self.gain.abs() / 20.0);
        let k = (PI * (self.freq / self.sample_rate)).tan();
        let norm = (1.0 + k / self.q + k * k).recip();

        match self.filter_type {
            FilterType::Lowpass => {
                self.a0 = k * k * norm;
                self.a1 = 2.0 * self.a0;
                self.a2 = self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            },
            FilterType::Highpass => {
                self.a0 = norm;
                self.a1 = -2.0 * self.a0;
                self.a2 = self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            },
            FilterType::Bandpass => {
                self.a0 = k / self.q * norm;
                self.a1 = 0.0;
                self.a2 = -self.a0;
                self.b1 = 2.0 * (k * k - 1.0) * norm;
                self.b2 = (1.0 - k / self.q + k * k) * norm;
            },
        }
    }

    /// Process a signal through the filter and also
    /// returns the filtered out signal
    pub fn process(&mut self, x: f32) -> (f32, f32) {
        let output = x * self.a0 + self.s1;
        self.s1 = x * self.a1 + self.s2 - self.b1 * output;
        self.s2 = x * self.a2 - self.b2 * output;
        (output, x - output)
    }

    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}