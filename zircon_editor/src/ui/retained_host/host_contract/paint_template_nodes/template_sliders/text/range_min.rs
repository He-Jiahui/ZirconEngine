use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::{
    slider_range_min_label, slider_range_min_value_rect, workbench_slider_metrics,
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
    let metrics = workbench_slider_metrics();
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
        metrics.value_radius,
        opacity,
    ));
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: value_rect.x + metrics.value_text_inset_x,
            y: value_rect.y + (value_rect.height - metrics.line_height).max(0.0) * 0.5,
            width: (value_rect.width - metrics.value_text_inset_x * 2.0).max(1.0),
            height: metrics.line_height,
        },
        Some(clip.clone()),
        order + 1,
        slider_range_min_label(range_min_percent),
        style.value_text,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
