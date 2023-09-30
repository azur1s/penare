use crate::{PenareParams, data::WaveshapersData};
use std::sync::{Arc, Mutex};
use nih_plug::prelude::*;
use nih_plug_vizia::{
    vizia::prelude::*,
    widgets::*,
    ViziaState,
    create_vizia_editor,
};

mod waveshaper_display;

#[derive(Lens)]
struct Data {
    params: Arc<PenareParams>,
    waveshaper_data: Arc<Mutex<WaveshapersData>>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 700))
}

// Fonts
const FONT_REGULAR: &[u8] = include_bytes!("../assets/PhlattGrotesk-Regular.ttf");

pub(crate) fn create(
    params: Arc<PenareParams>,
    waveshaper_data: Arc<Mutex<WaveshapersData>>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, nih_plug_vizia::ViziaTheming::Custom, move |cx, _| {
        // Add and set default fonts
        cx.add_fonts_mem(&[
            FONT_REGULAR,
        ]);
        cx.set_default_font(&["Phlatt Grotesk"]);

        cx.add_theme(include_str!("editor/theme.css"));

        Data {
            params: params.clone(),
            waveshaper_data: waveshaper_data.clone(),
        }.build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            waveshaper_display::WaveshaperDisplay::new(
                cx,
                Data::waveshaper_data,
            )
            .width(Percentage(100.0))
            .height(Pixels(100.0));

            // Macro for commonly used components
            macro_rules! hstack {
                ($cx:ident, $f:expr) => {
                    HStack::new($cx, $f)
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0))
                    .col_between(Pixels(10.0))
                    .left(Pixels(20.0))
                };
            }
            macro_rules! slider {
                ($cx:ident, $label:expr, $param:ident) => {
                    hstack!($cx, |cx| {
                        ParamSlider::new(cx, Data::params, |p| &p.$param)
                        .height(Pixels(36.0));
                        Label::new(cx, $label);
                    })
                };
            }
            macro_rules! button {
                ($cx:ident, $label:expr, $param:ident) => {
                    hstack!($cx, |cx| {
                        ParamButton::new(cx, Data::params, |p| &p.$param)
                        .height(Pixels(36.0));
                        Label::new(cx, $label);
                    })
                };
            }
            macro_rules! header {
                ($cx:ident, $label:expr) => {
                    HStack::new($cx, |cx| {
                        Label::new(cx, &format!("{}", $label));
                    })
                    .class("header")
                    .child_space(Stretch(1.0))
                    .top(Pixels(10.0))
                    .bottom(Pixels(10.0));
                };
            }

            HStack::new(cx, |cx| {
                // Input-Output related parameters
                ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                    header!(cx, "mix");
                    slider!(cx, "mix", mix);
                    button!(cx, "hard clip output", output_clip);
                    slider!(cx, "output clip threshold", output_clip_threshold);
                    slider!(cx, "input gain", input_gain);
                    slider!(cx, "output gain", output_gain);

                    header!(cx, "filter");
                    slider!(cx, "excess mix", excess_mix);
                    slider!(cx, "filter 1 type", f1_type);
                    slider!(cx, "filter 1 freq", f1_freq);
                    slider!(cx, "filter 1 q", f1_q);
                    slider!(cx, "filter 2 type", f2_type);
                    slider!(cx, "filter 2 freq", f2_freq);
                    slider!(cx, "filter 2 q", f2_q);
                    button!(cx, "excess bypass", excess_bypass);

                    // Distortions parameter
                    header!(cx, "waveshaper");
                    slider!(cx, "function mix", function_mix);
                    slider!(cx, "+ function type", pos_function_type);
                    slider!(cx, "+ function parameter", pos_function_param);
                    slider!(cx, "+ function mix", pos_function_mix);
                    slider!(cx, "- function type", neg_function_type);
                    slider!(cx, "- function parameter", neg_function_param);
                    slider!(cx, "- function mix", neg_function_mix);
                    slider!(cx, "clip sign", clip_sign);
                    slider!(cx, "copy from", copy_function);
                    button!(cx, "flip phase", flip);
                })
                .class("params");
            })
            .width(Percentage(100.0));

            // Footer
            HStack::new(cx, |cx| {
                Label::new(cx, &format!(
                    "penare v{} by azur1s",
                    crate::Penare::VERSION,
                ));
            })
            .class("footer")
            .width(Percentage(100.0))
            .height(Pixels(40.0))
            .child_space(Stretch(1.0))
            .child_bottom(Stretch(1.0));
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}