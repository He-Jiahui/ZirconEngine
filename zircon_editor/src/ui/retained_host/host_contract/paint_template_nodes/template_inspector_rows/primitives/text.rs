use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_geometry::is_paintable_rect;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) {
    if text.trim().is_empty() || !is_paintable_rect(&rect) {
        return;
    }
    let metrics = current_host_metrics();
    let line_height = metrics.line_height(metrics.font_body);
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        metrics.font_body,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
