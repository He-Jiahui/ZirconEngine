use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layers::popup_text_order;
use super::super::metrics::workbench_popup_row_metrics;
use super::geometry::popup_row_shortcut_rect;
use super::style::popup_row_text_command_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_row_shortcut(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    shortcut: String,
    color: [u8; 4],
    opacity: f32,
) {
    if shortcut.is_empty() || intersect(row_rect, clip).is_none() {
        return;
    }
    let metrics = workbench_popup_row_metrics();
    let style = popup_row_text_command_style(color, &metrics);
    commands.push(HostPaintCommand::text(
        popup_row_shortcut_rect(row_rect, &metrics),
        Some(clip.clone()),
        popup_text_order(order),
        shortcut,
        style.color,
        style.font_size,
        style.line_height,
        style.paint_style,
        opacity,
    ));
}
