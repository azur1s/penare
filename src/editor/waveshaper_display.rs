use crate::{fxs::utils::hard_clip, data::WaveshapersData};
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};
use nih_plug_vizia::vizia::{prelude::*, vg};


pub struct WaveshaperDisplay {
    waveshaper_data: Arc<Mutex<WaveshapersData>>,
}

impl WaveshaperDisplay {
    pub fn new<LWaveshapersData>(
        cx: &mut Context,
        waveshaper_data: LWaveshapersData,
    ) -> Handle<Self> where 
        LWaveshapersData: Lens<Target = Arc<Mutex<WaveshapersData>>>,
    {
        Self {
            waveshaper_data: waveshaper_data.get(cx),
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

        let data = self.waveshaper_data.lock().unwrap();
        let input_gain = data.get_input_gain();
        let output_gain = data.get_output_gain();
        let pos_function_type = data.get_pos_function_type();
        let neg_function_type = data.get_neg_function_type();
        let pos_function_param = data.get_pos_function_param();
        let neg_function_param = data.get_neg_function_param();
        let clip = data.get_clip();
        let clip_threshold = data.get_clip_threshold();

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
        path.move_to(0.0, bounds.h * 0.5);
        path.line_to(bounds.w, bounds.h * 0.5);
        path.move_to(bounds.w * 0.5, 0.0);
        path.line_to(bounds.w * 0.5, bounds.h);

        // Draw [-1, 1] lines
        path.move_to(0.0, -1.0 * a * scale.recip() + a);
        path.line_to(bounds.w, -1.0 * a * scale.recip() + a);
        path.move_to(0.0, 1.0 * a * scale.recip() + a);
        path.line_to(bounds.w, 1.0 * a * scale.recip() + a);

        let sin = |x: f32| (-x * PI / (0.5 * bounds.w)).sin();

        // Draw sin function
        for x in 0..(bounds.w as usize) {
            let x = x as f32;
            let y = sin(x);
            let y = y * a * scale.recip() + a;
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

        for x in 0..(bounds.w as usize) {
            let x = x as f32;
            // Sin function
            let y = sin(x) * input_gain;
            // Apply function
            let y = if !data.get_copy().is_off() {
                let (ft, fp) = if data.get_copy().is_positive() {
                    (pos_function_type, pos_function_param)
                } else {
                    (neg_function_type, neg_function_param)
                };
                ft.apply(y, fp)
            } else {
                if -y >= 0.0 {
                    pos_function_type.apply(y, pos_function_param)
                } else {
                    neg_function_type.apply(y, neg_function_param)
                }
            };
            // Flip
            let y = if data.get_flip() { -y } else { y };
            // Clip output
            let y = if clip {
                hard_clip(y, clip_threshold)
            } else {
                y
            } * output_gain;
            // Scale to view
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