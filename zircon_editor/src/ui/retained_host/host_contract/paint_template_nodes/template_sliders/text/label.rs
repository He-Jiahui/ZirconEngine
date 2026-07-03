use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::workbench_slider_metrics;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: String,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    let metrics = workbench_slider_metrics();
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + metrics.horizontal_inset,
            y: rect.y + (rect.height - metrics.line_height).max(0.0) * 0.5,
            width: metrics.label_width,
            height: metrics.line_height,
        },
        Some(clip.clone()),
        order,
        label,
        style.label_text,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
