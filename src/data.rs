use crate::fxs::waveshaper::FunctionType;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use atomic_float::AtomicF32;
use nih_plug::prelude::*;

pub struct WaveshapersData {
    /// Input gain
    pub input_gain: AtomicF32,
    /// Output gain
    pub output_gain: AtomicF32,
    /// ID of the function type. Use [`FunctionType::from_id`] to get the function type
    /// from the ID.
    /// The first element is the positive function type,
    /// and the second element is the negative function type.
    pub function_types: [AtomicUsize; 2],
    /// Parameters for the function. The first element is the positive function parameter,
    /// and the second element is the negative function parameter.
    pub function_params: [AtomicF32; 2],
    /// Clip output
    pub clip: AtomicBool,
    /// Clip threshold
    pub clip_threshold: AtomicF32,
}

impl Default for WaveshapersData {
    fn default() -> Self {
        let f = FunctionType::HardClip.id();
        let db = util::db_to_gain(0.0);
        Self {
            input_gain: AtomicF32::new(db),
            output_gain: AtomicF32::new(db),
            function_types: [AtomicUsize::new(f), AtomicUsize::new(f)],
            function_params: [AtomicF32::new(db), AtomicF32::new(db)],
            clip: AtomicBool::new(true),
            clip_threshold: AtomicF32::new(db),
        }
    }
}

// TODO: Probably turn all of this into macros
impl WaveshapersData {
    pub fn get_input_gain(&self) -> f32 {
        self.input_gain.load(Ordering::Relaxed)
    }

    pub fn get_output_gain(&self) -> f32 {
        self.output_gain.load(Ordering::Relaxed)
    }

    pub fn get_pos_function_type(&self) -> FunctionType {
        FunctionType::from_id(self.function_types[0].load(Ordering::Relaxed))
    }

    pub fn get_neg_function_type(&self) -> FunctionType {
        FunctionType::from_id(self.function_types[1].load(Ordering::Relaxed))
    }

    pub fn get_pos_function_param(&self) -> f32 {
        self.function_params[0].load(Ordering::Relaxed)
    }

    pub fn get_neg_function_param(&self) -> f32 {
        self.function_params[1].load(Ordering::Relaxed)
    }

    pub fn get_clip(&self) -> bool {
        self.clip.load(Ordering::Relaxed)
    }

    pub fn get_clip_threshold(&self) -> f32 {
        self.clip_threshold.load(Ordering::Relaxed)
    }

    pub fn set_input_gain(&self, input_gain: f32) {
        self.input_gain.store(input_gain, Ordering::Relaxed);
    }

    pub fn set_output_gain(&self, output_gain: f32) {
        self.output_gain.store(output_gain, Ordering::Relaxed);
    }

    pub fn set_pos_function_type(&self, function_type: FunctionType) {
        self.function_types[0].store(function_type.id(), Ordering::Relaxed);
    }

    pub fn set_neg_function_type(&self, function_type: FunctionType) {
        self.function_types[1].store(function_type.id(), Ordering::Relaxed);
    }

    pub fn set_pos_function_param(&self, function_param: f32) {
        self.function_params[0].store(function_param, Ordering::Relaxed);
    }

    pub fn set_neg_function_param(&self, function_param: f32) {
        self.function_params[1].store(function_param, Ordering::Relaxed);
    }

    pub fn set_clip(&self, clip: bool) {
        self.clip.store(clip, Ordering::Relaxed);
    }

    pub fn set_clip_threshold(&self, clip_threshold: f32) {
        self.clip_threshold.store(clip_threshold, Ordering::Relaxed);
    }
}