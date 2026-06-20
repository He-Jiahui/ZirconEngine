use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::{
    slider_range_min_label, slider_range_min_value_rect, SLIDER_FONT_SIZE, SLIDER_LINE_HEIGHT,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_range_min_value(
    commands: &mut Vec<HostPaintCommand>,
    style: &WorkbenchSliderStyle,
    rect: &FrameRect,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    range_min_percent: f32,
    opacity: f32,
) {
    let Some(value_rect) = slider_range_min_value_rect(rect, track_rect) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        value_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.value_surface),
        Some(style.range_value_border),
        1.0,
        4.0,
        opacity,
    ));
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: value_rect.x + 6.0,
            y: value_rect.y + (value_rect.height - SLIDER_LINE_HEIGHT).max(0.0) * 0.5,
            width: (value_rect.width - 12.0).max(1.0),
            height: SLIDER_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order + 1,
        slider_range_min_label(range_min_percent),
        style.value_text,
        SLIDER_FONT_SIZE,
        SLIDER_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
