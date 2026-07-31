use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{layout, style};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_preview_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    preview_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: style::DragOverlayPalette,
    metrics: &layout::DragOverlayMetrics,
) {
    let Some(label) = preview_label(node) else {
        return;
    };
    let text_rect = layout::preview_text_frame(preview_rect, metrics);
    if text_rect.width <= 0.0 || text_rect.height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label,
        palette.preview_text,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn preview_label(node: &TemplatePaneNodeData) -> Option<String> {
    [
        node.drag_payload_label.as_str(),
        node.text.as_str(),
        node.drag_payload_reference.as_str(),
        node.value_text.as_str(),
    ]
    .into_iter()
    .map(str::trim)
    .find(|value| !value.is_empty())
    .map(str::to_string)
}
