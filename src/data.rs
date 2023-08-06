use crate::fxs::waveshaper::FunctionType;
use std::sync::atomic::{AtomicUsize, Ordering};
use atomic_float::AtomicF32;
use nih_plug::prelude::*;

pub struct WaveshapersData {
    /// ID of the function type. Use [`FunctionType::from_id`] to get the function type
    /// from the ID.
    /// The first element is the positive function type,
    /// and the second element is the negative function type.
    pub function_types: [AtomicUsize; 2],
    /// Parameters for the function. The first element is the positive function parameter,
    /// and the second element is the negative function parameter.
    pub function_params: [AtomicF32; 2],
}

impl Default for WaveshapersData {
    fn default() -> Self {
        let f = FunctionType::HardClip.id();
        let p = util::db_to_gain(0.0);
        Self {
            function_types: [AtomicUsize::new(f), AtomicUsize::new(f)],
            function_params: [AtomicF32::new(p), AtomicF32::new(p)],
        }
    }
}

impl WaveshapersData {
    pub fn get_pos_function_type(&self) -> Option<FunctionType> {
        FunctionType::from_id(self.function_types[0].load(Ordering::Relaxed))
    }

    pub fn get_neg_function_type(&self) -> Option<FunctionType> {
        FunctionType::from_id(self.function_types[1].load(Ordering::Relaxed))
    }

    pub fn get_pos_function_param(&self) -> f32 {
        self.function_params[0].load(Ordering::Relaxed)
    }

    pub fn get_neg_function_param(&self) -> f32 {
        self.function_params[1].load(Ordering::Relaxed)
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
}