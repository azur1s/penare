use crate::{fxs::waveshaper::FunctionType, data::WaveshapersData};
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
        Some("function-display")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let data = self.waveshaper_data.lock().unwrap();
        let pos_function_type = data.get_pos_function_type().unwrap_or(FunctionType::HardClip);
        let neg_function_type = data.get_neg_function_type().unwrap_or(FunctionType::HardClip);
        let pos_function_param = data.get_pos_function_param();
        let neg_function_param = data.get_neg_function_param();

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
        let paint = vg::Paint::color(Color::rgb(127, 127, 127).into())
            .with_line_width(line_width);
        path.move_to(0.0, bounds.h * 0.5);
        path.line_to(bounds.w, bounds.h * 0.5);
        path.move_to(bounds.w * 0.5, 0.0);
        path.line_to(bounds.w * 0.5, bounds.h);

        let sin = |x: f32| (-x * PI / (0.5 * bounds.w)).sin();

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
        let paint = vg::Paint::color(Color::rgb(255, 255, 255).into())
            .with_line_width(line_width);

        for x in 0..(bounds.w as usize) {
            let x = x as f32;
            // Sin function
            let y = sin(x);
            // Apply function
            let y = if -y >= 0.0 {
                pos_function_type.apply(y, pos_function_param)
            } else {
                neg_function_type.apply(y, neg_function_param)
            };
            // Scale to view
            let y = y * a * scale.recip()
             + a;
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