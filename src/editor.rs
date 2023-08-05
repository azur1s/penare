use crate::PenareParams;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use nih_plug::prelude::*;
use nih_plug_vizia::{
    vizia::prelude::*,
    widgets::*,
    ViziaState,
    assets,
    create_vizia_editor,
};

mod waveshaper_display;

#[derive(Lens)]
struct Data {
    params: Arc<PenareParams>,
    pos_function_type: Arc<AtomicUsize>,
    pos_function_param: Arc<AtomicF32>,
    neg_function_type: Arc<AtomicUsize>,
    neg_function_param: Arc<AtomicF32>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 700))
}

pub(crate) fn create(
    params: Arc<PenareParams>,
    pos_function_type: Arc<AtomicUsize>,
    pos_function_param: Arc<AtomicF32>,
    neg_function_type: Arc<AtomicUsize>,
    neg_function_param: Arc<AtomicF32>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, nih_plug_vizia::ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
            pos_function_type: pos_function_type.clone(),
            pos_function_param: pos_function_param.clone(),
            neg_function_type: neg_function_type.clone(),
            neg_function_param: neg_function_param.clone(),
        }.build(cx);

        PopupData::default().build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            // TODO: Implement waveform display
            waveshaper_display::WaveshaperDisplay::new(
                cx,
                Data::pos_function_type,
                Data::pos_function_param,
                Data::neg_function_type,
                Data::neg_function_param,
            )
            .width(Percentage(100.0))
            .height(Pixels(200.0))
            .background_color(Color::rgb(0, 0, 0));

            ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                VStack::new(cx, |cx| {
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
                    macro_rules! label {
                        ($cx:ident, $label:expr) => {
                            HStack::new($cx, |cx| {
                                Label::new(cx, $label)
                                .font_size(24.0);
                            })
                            .child_space(Stretch(1.0));
                        };
                    }

                    label!(cx, "Mix");
                    slider!(cx, "Mix", mix);
                    button!(cx, "Hard Clip Output", output_clip);
                    slider!(cx, "Output Clip Threshold", output_clip_threshold);

                    label!(cx, "Waveshaper");
                    slider!(cx, "Pre Gain", pre_gain);
                    slider!(cx, "Function Mix", function_mix);
                    slider!(cx, "+ Function Type", pos_function_type);
                    slider!(cx, "+ Function Parameter", pos_function_param);
                    slider!(cx, "+ Function Mix", pos_function_mix);
                    slider!(cx, "- Function Type", neg_function_type);
                    slider!(cx, "- Function Parameter", neg_function_param);
                    slider!(cx, "- Function Mix", neg_function_mix);
                    slider!(cx, "Post Gain", post_gain);
                    slider!(cx, "Copy From", copy_function);
                    button!(cx, "Clip Function", clip_function);
                    button!(cx, "Flip Phase", flip);

                    label!(cx, "Rectifier");
                    button!(cx, "Rectify", rectify);
                    slider!(cx, "Rectify Mix", rectify_mix);
                    slider!(cx, "Rectified Signal Mix In", rectify_mix_in);
                    slider!(cx, "Rectify Type", rectify_type);
                    button!(cx, "Flip Rectified Signal", rectify_flip);

                    label!(cx, "Crusher");
                    button!(cx, "Crush", crush);
                    slider!(cx, "Crush Mix", crush_mix);
                    slider!(cx, "Crush Mix In", crush_mix_in);
                    slider!(cx, "Crush Type", crush_type);
                    slider!(cx, "Crush Step", crush_step);

                    label!(cx, "Filter");
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
            .width(Percentage(100.0));

            HStack::new(cx, |cx| {
                let small = 12.0;
                Label::new(cx, crate::Penare::VENDOR)
                .font_size(small);
                Label::new(cx, crate::Penare::NAME)
                .font_size(24.0);
                Label::new(cx, crate::Penare::VERSION)
                .font_size(small);
            })
            .width(Percentage(100.0))
            .height(Pixels(40.0))
            .background_color(Color::rgb(200, 200, 200))
            .child_space(Stretch(1.0))
            .child_top(Stretch(1.0))
            .child_bottom(Stretch(1.0));
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}