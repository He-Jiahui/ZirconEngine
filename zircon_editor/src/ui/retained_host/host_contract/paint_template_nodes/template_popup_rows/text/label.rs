use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_popup_row_adornments::PopupRowAdornmentKind;
use super::super::geometry::frame_is_within;
use super::super::layers::popup_text_order;
use super::super::metrics::workbench_popup_row_metrics;
use super::geometry::popup_row_label_rect;
use super::style::popup_row_text_command_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_row_label(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: String,
    color: [u8; 4],
    adornment: Option<PopupRowAdornmentKind>,
    opacity: f32,
) {
    if label.is_empty() || intersect(row_rect, clip).is_none() {
        return;
    }
    let metrics = workbench_popup_row_metrics();
    let style = popup_row_text_command_style(color, &metrics);
    let text_rect = popup_row_label_rect(row_rect, &metrics, adornment.is_some());
    if !frame_is_within(row_rect, &text_rect)
        || intersect(&text_rect, clip).is_none()
        || text_rect.height < style.line_height
    {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        popup_text_order(order),
        label,
        style.color,
        style.font_size,
        style.line_height,
        style.paint_style,
        opacity,
    ));
}
