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
    ViziaState::new(|| (800, 700))
}

// Fonts
const FONT_REGULAR: &[u8] = include_bytes!("../assets/CommitMono-Regular.otf");
const FONT_BOLD: &[u8] = include_bytes!("../assets/CommitMono-Bold.otf");

pub(crate) fn create(
    params: Arc<PenareParams>,
    waveshaper_data: Arc<Mutex<WaveshapersData>>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, nih_plug_vizia::ViziaTheming::Custom, move |cx, _| {
        // Add and set default fonts
        cx.add_fonts_mem(&[
            FONT_REGULAR,
            FONT_BOLD,
        ]);
        cx.set_default_font(&["CommitMono"]);

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
            .height(Pixels(200.0));

            // Macro for commonly used components
            macro_rules! hstack {
                ($cx:ident, $f:expr) => {
                    HStack::new($cx, $f)
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0))
                    .col_between(Pixels(10.0))
                    .left(Pixels(10.0))
                };
            }
            macro_rules! slider {
                ($cx:ident, $label:expr, $param:ident) => {
                    hstack!($cx, |cx| {
                        ParamSlider::new(cx, Data::params, |p| &p.$param);
                        Label::new(cx, $label);
                    })
                };
            }
            macro_rules! button {
                ($cx:ident, $label:expr, $param:ident) => {
                    hstack!($cx, |cx| {
                        ParamButton::new(cx, Data::params, |p| &p.$param);
                        Label::new(cx, $label);
                    })
                };
            }
            macro_rules! header {
                ($cx:ident, $label:expr) => {
                    HStack::new($cx, |cx| {
                        Label::new(cx, &format!("λ {}", $label));
                    })
                    .class("header")
                    .child_space(Stretch(1.0));
                };
            }

            HStack::new(cx, |cx| {
                // Input-Output related parameters
                ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                    VStack::new(cx, |cx| {
                        header!(cx, "Mix");
                        slider!(cx, "Mix", mix);
                        button!(cx, "Hard Clip Output", output_clip);
                        slider!(cx, "Output Clip Threshold", output_clip_threshold);
                        slider!(cx, "Input Gain", input_gain);
                        slider!(cx, "Output Gain", output_gain);

                        header!(cx, "Filter");
                        slider!(cx, "Excess Mix", excess_mix);
                        slider!(cx, "Filter 1 Type", f1_type);
                        slider!(cx, "Filter 1 Freq", f1_freq);
                        slider!(cx, "Filter 1 Q", f1_q);
                        slider!(cx, "Filter 2 Type", f2_type);
                        slider!(cx, "Filter 2 Freq", f2_freq);
                        slider!(cx, "Filter 2 Q", f2_q);
                        button!(cx, "Excess Signal Bypass", excess_bypass);
                    })
                    .row_between(Pixels(10.0));
                })
                .class("params");

                // Distortions parameter
                ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                    header!(cx, "Waveshaper");
                    slider!(cx, "Function Mix", function_mix);
                    slider!(cx, "+ Function Type", pos_function_type);
                    slider!(cx, "+ Function Parameter", pos_function_param);
                    slider!(cx, "+ Function Mix", pos_function_mix);
                    slider!(cx, "- Function Type", neg_function_type);
                    slider!(cx, "- Function Parameter", neg_function_param);
                    slider!(cx, "- Function Mix", neg_function_mix);
                    slider!(cx, "Copy From", copy_function);
                    button!(cx, "Flip Phase", flip);
                })
                .class("params");
            })
            .width(Percentage(100.0));

            // Footer
            HStack::new(cx, |cx| {
                Label::new(cx, &format!(
                    "{} - {} - v{}",
                    crate::Penare::VENDOR,
                    crate::Penare::NAME,
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