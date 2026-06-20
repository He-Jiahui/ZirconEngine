use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_node_images::{is_icon_node, is_icon_only_node, leading_icon_size};
use super::super::template_node_labels::template_node_label;
use super::metrics::{MIN_TEXT_RECT_HEIGHT, TEXT_HORIZONTAL_INSET, TEXT_VERTICAL_INSET};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_rect_for_node(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let horizontal = TEXT_HORIZONTAL_INSET
        .min((rect.width * 0.25).max(0.0))
        .max(0.0);
    let vertical = TEXT_VERTICAL_INSET
        .min(((rect.height - MIN_TEXT_RECT_HEIGHT) * 0.5).max(1.0))
        .max(0.0);
    let mut x = rect.x + horizontal;
    let mut width = (rect.width - horizontal * 2.0).max(0.0);
    if is_leading_icon_text_node(node) {
        let leading = (leading_icon_size(rect) + TEXT_HORIZONTAL_INSET).min(width);
        x += leading;
        width = (width - leading).max(0.0);
    }
    FrameRect {
        x,
        y: rect.y + vertical,
        width,
        height: (rect.height - vertical * 2.0).max(0.0),
    }
}

fn is_leading_icon_text_node(node: &TemplatePaneNodeData) -> bool {
    is_icon_node(node) && !is_icon_only_node(node) && !template_node_label(node, None).is_empty()
}
