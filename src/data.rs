use crate::{fxs::waveshaper::FunctionType, params::TriState};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use paste::paste;

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
    /// and the second element is the negative function parameter
    pub function_params: [AtomicF32; 2],
    /// Function mixs in percentage
    pub function_mixs: [AtomicF32; 2],
    /// Clip output
    pub clip: AtomicBool,
    /// Clip threshold
    pub clip_threshold: AtomicF32,
    /// Copy functions
    pub copy: AtomicUsize,
    /// Flip phase
    pub flip: AtomicBool,
}

impl Default for WaveshapersData {
    fn default() -> Self {
        let f = FunctionType::HardClip.into();
        let db = util::db_to_gain(0.0);
        Self {
            input_gain: AtomicF32::new(db),
            output_gain: AtomicF32::new(db),
            function_types: [AtomicUsize::new(f), AtomicUsize::new(f)],
            function_params: [AtomicF32::new(db), AtomicF32::new(db)],
            function_mixs: [AtomicF32::new(0.0), AtomicF32::new(0.0)],
            clip: AtomicBool::new(true),
            clip_threshold: AtomicF32::new(db),
            copy: AtomicUsize::new(TriState::Off.into()),
            flip: AtomicBool::new(false),
        }
    }
}

// Macro for defining getters function
macro_rules! get {
    // Normal value getters
    ($name:ident $t:ty) => {
        paste! {
            pub fn [<get_ $name>](&self) -> $t {
                self.$name.load(Ordering::Relaxed).into()
            }
        }
    };
    // "Polar" value getters
    ($name:ident polar $t:ty) => {
        paste! {
            pub fn [<get_pos_ $name>](&self) -> $t {
                self.[<$name s>][0].load(Ordering::Relaxed).into()
            }
            pub fn [<get_neg_ $name>](&self) -> $t {
                self.[<$name s>][1].load(Ordering::Relaxed).into()
            }
        }
    };
}
// Macro for defining setters function
macro_rules! set {
    ($name:ident $t:ty) => {
        paste! {
            pub fn [<set_ $name>](&self, $name: $t) {
                self.$name.store($name.into(), Ordering::Relaxed);
            }
        }
    };
    ($name:ident polar $t:ty) => {
        paste! {
            pub fn [<set_ $name s>](&self, pos: $t, neg: $t) {
                self.[<$name s>][0].store(pos.into(), Ordering::Relaxed);
                self.[<$name s>][1].store(neg.into(), Ordering::Relaxed);
            }
        }
    };
}

#[allow(dead_code)]
impl WaveshapersData {
    get!(input_gain           f32);
    get!(output_gain          f32);
    get!(function_type  polar FunctionType);
    get!(function_param polar f32);
    get!(function_mix   polar f32);
    get!(clip                 bool);
    get!(clip_threshold       f32);
    get!(copy                 TriState);
    get!(flip                 bool);

    set!(input_gain           f32);
    set!(output_gain          f32);
    set!(function_type  polar FunctionType);
    set!(function_param polar f32);
    set!(function_mix   polar f32);
    set!(clip                 bool);
    set!(clip_threshold       f32);
    set!(copy                 TriState);
    set!(flip                 bool);
}