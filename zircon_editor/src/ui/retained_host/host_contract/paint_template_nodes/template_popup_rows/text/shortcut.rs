use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::frame_is_within;
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
    let text_rect = popup_row_shortcut_rect(row_rect, &metrics);
    if !frame_is_within(row_rect, &text_rect)
        || !frame_is_within(clip, &text_rect)
        || text_rect.height < style.line_height
    {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
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
