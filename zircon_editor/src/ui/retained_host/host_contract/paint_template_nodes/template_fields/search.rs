use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::workbench_field_metrics;
use crate::ui::retained_host::host_contract::search_field_clear_action_frame;
use zircon_runtime_interface::ui::layout::UiFrame;

mod glyph;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use glyph::{
    push_search_field_clear_glyph, push_search_field_glyph,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_search_field(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.component_role.as_str(), "search-field")
        || matches!(node.role.as_str(), "SearchField")
        || search_identity_text(node.control_id.as_str())
        || search_identity_text(node.binding_id.as_str())
        || search_identity_text(node.action_id.as_str())
        || search_identity_text(node.edit_action_id.as_str())
        || search_identity_text(node.commit_action_id.as_str())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_text_left(
    node: &TemplatePaneNodeData,
) -> f32 {
    let metrics = workbench_field_metrics();
    if is_search_field(node) {
        metrics.search_text_left
    } else {
        metrics.input_pad_left
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_paint_rect(
    node: &TemplatePaneNodeData,
    rect: FrameRect,
) -> FrameRect {
    let metrics = workbench_field_metrics();
    let max_height = metrics.search_max_height;
    if !is_search_field(node) || rect.height <= max_height {
        return rect;
    }

    let height = max_height.round().max(0.0);
    FrameRect {
        y: (rect.y + ((rect.height - height) * 0.5)).round(),
        height,
        ..rect
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_label_is_placeholder(
    node: &TemplatePaneNodeData,
    label: &str,
) -> bool {
    is_search_field(node) && node.value_text.trim().is_empty() && !label.trim().is_empty()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_has_clear_action(
    node: &TemplatePaneNodeData,
) -> bool {
    is_search_field(node) && node.has_clear_action && !node.value_text.trim().is_empty()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_field_clear_action_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> Option<FrameRect> {
    if !search_field_has_clear_action(node) {
        return None;
    }

    let action =
        search_field_clear_action_frame(UiFrame::new(rect.x, rect.y, rect.width, rect.height))?;
    let action = FrameRect {
        x: action.x,
        y: action.y,
        width: action.width,
        height: action.height,
    };
    super::geometry::frame_is_within(&action, rect).then_some(action)
}

fn search_identity_text(value: &str) -> bool {
    value.to_ascii_lowercase().contains("search")
}
