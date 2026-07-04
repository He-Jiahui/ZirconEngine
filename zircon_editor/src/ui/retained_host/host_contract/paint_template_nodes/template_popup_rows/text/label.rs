use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_popup_row_adornments::PopupRowAdornmentKind;
use super::super::metrics::workbench_popup_row_metrics;
use super::super::surface::POPUP_ROW_ORDER_OFFSET;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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
    let right_reserved = if adornment.is_some() {
        metrics.adornment_reserved_width
    } else {
        metrics.text_right
    };
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: row_rect.x + metrics.text_left,
            y: row_rect.y + metrics.text_top,
            width: (row_rect.width - metrics.text_left - right_reserved).max(1.0),
            height: (row_rect.height - metrics.text_top - metrics.text_bottom)
                .max(metrics.min_text_rect_height),
        },
        Some(clip.clone()),
        order + POPUP_ROW_ORDER_OFFSET + 3,
        label,
        color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
