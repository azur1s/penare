use crate::{fxs::utils::{hard_clip, mix_between}, data::UIData};
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};
use nih_plug_vizia::vizia::{prelude::*, vg};

pub struct WaveshaperDisplay {
    /// Reference to the waveshapers data
    ui_data: Arc<Mutex<UIData>>,
}

impl WaveshaperDisplay {
    /// Create a new waveshaper display
    pub fn new<LUIData>(
        cx: &mut Context,
        ui_data: LUIData,
    ) -> Handle<Self> where 
        LUIData: Lens<Target = Arc<Mutex<UIData>>>,
    {
        Self {
            ui_data: ui_data.get(cx),
        }.build(cx, |_cx| ())
    }
}

impl View for WaveshaperDisplay {
    fn element(&self) -> Option<&'static str> {
        Some("waveshaper-display")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        // Get UI data
        let data = self.ui_data.lock().unwrap();
        let pos_function_type  = data.get_pos_function_type();
        let neg_function_type  = data.get_neg_function_type();
        let pos_function_param = data.get_pos_function_param();
        let neg_function_param = data.get_neg_function_param();
        let pos_function_mix   = data.get_pos_function_mix();
        let neg_function_mix   = data.get_neg_function_mix();

        let waveform = data.get_waveform();

        // Calculate commonly used variables
        let line_width = cx.style.dpi_factor as f32 * 1.5;
        // 1 <= scale <= 2;
        let scale = 1.5f32;
        let a = bounds.h * 0.5;

        // Draw background color
        let mut path = vg::Path::new();
        let paint = vg::Paint::color(cx.background_color().cloned().unwrap_or_default().into());
        path.rect(0.0, 0.0, bounds.w, bounds.h);
        canvas.fill_path(&mut path, &paint);

        // Draw x and y axis
        let mut path = vg::Path::new();
        let paint = vg::Paint::color(cx.border_color().cloned().unwrap_or_default().into())
            .with_line_width(line_width);
        // X axis
        path.move_to(0.0, bounds.h * 0.5);
        path.line_to(bounds.w, bounds.h * 0.5);
        // Y axis
        path.move_to(bounds.w * 0.5, 0.0);
        path.line_to(bounds.w * 0.5, bounds.h);

        // Draw [-1, 1] dB lines
        path.move_to(0.0, -1.0 * a * scale.recip() + a);
        path.line_to(bounds.w, -1.0 * a * scale.recip() + a);
        path.move_to(0.0, 1.0 * a * scale.recip() + a);
        path.line_to(bounds.w, 1.0 * a * scale.recip() + a);

        canvas.stroke_path(&mut path, &paint);

        // Scale the waveform (from its length to the width) so that it fits in the display
        let waveform_scale = bounds.w / waveform[0].len() as f32;

        // Draw waveform
        let mut path = vg::Path::new();
        let paint = vg::Paint::color(cx.caret_color().cloned().unwrap_or_default().into())
            .with_line_width(line_width);

        for x in 0..(bounds.w as usize) {
            let x = x as f32;
            let i = (x / waveform_scale) as usize;
            if i >= waveform[0].len() {
                break;
            }
            let y = waveform[0][i];
            let y = bounds.h - (y * a * scale.recip() + a);
            if x == 0.0 {
                path.move_to(x as f32, y);
            } else {
                path.line_to(x as f32, y);
            }
        }

        canvas.stroke_path(&mut path, &paint);

        let mut path = vg::Path::new();
        let paint = vg::Paint::color(cx.font_color().cloned().unwrap_or_default().into())
            .with_line_width(line_width);

        // Draw waveshaped sin function
        for x in 0..(bounds.w as usize) {
            let x = x as f32;
            // Sin function
            let y_original = (-x * PI / (0.5 * bounds.w)).sin();
            let y = y_original;
            // Apply function
            let (ft, fp, fm) = if match (data.get_copy().is_on(), data.get_copy().is_positive(), -y >= 0.0) {
                (true,  true,  _   ) => true,
                (true,  false, _   ) => false,
                (false, _,    true ) => true,
                (false, _,    false) => false,
            } {
                (pos_function_type, pos_function_param, pos_function_mix)
            } else {
                (neg_function_type, neg_function_param, neg_function_mix)
            };
            let y = mix_between(y, ft.apply(y, fp), fm);
            // Clip sign
            let y = if data.get_clip_sign() {
                if y_original >= 0.0 {
                    y.max(0.0)
                } else {
                    y.min(0.0)
                }
            } else {
                y
            };
            // Flip
            let y = if data.get_flip() { -y } else { y };
            // Function mix
            let y = mix_between(y_original, y, data.get_mix());
            // Final clip
            let y = if data.get_clip() {
                hard_clip(y, data.get_clip_threshold())
            } else {
                y
            };
            // Scale Y axis to view
            let y = y * a * scale.recip() + a;
            // Draw
            if x == 0.0 {
                path.move_to(x as f32, y);
            } else {
                path.line_to(x as f32, y);
            }
        }

        canvas.stroke_path(&mut path, &paint);
    }
}