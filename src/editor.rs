use crate::PenareParams;
use std::{
    sync::{ Arc, atomic::Ordering },
    time::Duration,
};
use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_vizia::{
    vizia::prelude::*,
    widgets::*,
    ViziaState,
    assets,
    create_vizia_editor,
};

#[derive(Lens)]
struct Data {
    params: Arc<PenareParams>,
    peak_meter: Arc<AtomicF32>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (300, 400))
}

pub(crate) fn create(
    params: Arc<PenareParams>,
    peak_meter: Arc<AtomicF32>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, nih_plug_vizia::ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
            peak_meter: peak_meter.clone(),
        }.build(cx);

        PopupData::default().build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            PeakMeter::new(
                cx,
                Data::peak_meter
                    .map(|peak_meter| nih_plug::prelude::util::gain_to_db(peak_meter.load(Ordering::Relaxed))),
                Some(Duration::from_millis(600)),
            )
            .height(Pixels(50.0))
            .child_top(Stretch(1.0))
            .child_bottom(Stretch(1.0));

            ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                macro_rules! slider {
                    ($cx:ident, $label:expr, $param:ident) => {
                        HStack::new($cx, |cx| {
                            ParamSlider::new(cx, Data::params, |p| &p.$param);
                            Label::new(cx, $label);
                        })
                        .child_top(Stretch(1.0))
                        .child_bottom(Stretch(1.0))
                        .child_space(Pixels(8.0))
                    };
                }
                slider!(cx, "Mix", mix);
                slider!(cx, "Type", clip_type);
                Label::new(cx, "Gain");
                slider!(cx, "Pre Gain", pre_gain);
                slider!(cx, "Threshold", threshold);
                slider!(cx, "Post Gain", post_gain);
                Label::new(cx, "Filter");
                slider!(cx, "Excess Mix", excess_mix);
                slider!(cx, "Low Pass", low_pass);
                slider!(cx, "Low Pass Q", low_pass_q);
                slider!(cx, "High Pass", high_pass);
                slider!(cx, "High Pass Q", high_pass_q);
                HStack::new(cx, |cx| {
                    ParamButton::new(cx, Data::params, |p| &p.excess_bypass);
                    Label::new(cx, "Excess Signal Bypass");
                })
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .child_space(Pixels(8.0));
            })
            .width(Percentage(100.0))
            .top(Pixels(8.0));

            Label::new(cx, &format!(
                "{} by {} v{}",
                crate::Penare::NAME,
                crate::Penare::VENDOR,
                crate::Penare::VERSION,
            ))
            .width(Percentage(100.0))
            .height(Pixels(20.0))
            .font_size(12.0)
            .background_color(Color::rgb(200, 200, 200))
            .child_top(Stretch(1.0))
            .child_bottom(Stretch(1.0));
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}