use super::super::data::{FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData};
use super::super::frame_geometry::{contains_point, union_frame};
use super::super::template_geometry::frame_from_template_node;
use super::super::template_popup_layout::template_option_popup_frame_within;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};

/// Keeps hit containment separate from repaint damage: dropdowns contain both
/// their trigger and popup rows, while repaint needs the union of both regions.
pub(in crate::ui::retained_host::host_contract) struct PopupDismissTarget {
    pub(in crate::ui::retained_host::host_contract) control_id: SharedString,
    pub(in crate::ui::retained_host::host_contract) control_frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract) popup_frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract) damage_frame: FrameRect,
}

impl PopupDismissTarget {
    pub(in crate::ui::retained_host::host_contract) fn contains_point(
        &self,
        x: f32,
        y: f32,
    ) -> bool {
        contains_point(&self.control_frame, x, y) || contains_point(&self.popup_frame, x, y)
    }
}

pub(in crate::ui::retained_host::host_contract) fn active_popup_dismiss_target(
    nodes: &ModelRc<TemplatePaneNodeData>,
    interaction: &HostPaneInteractionStateData,
    bounds: &FrameRect,
    popup_rows: &[usize],
) -> Option<PopupDismissTarget> {
    for row in popup_rows.iter().rev().copied() {
        let Some(node) = nodes.get(row) else {
            continue;
        };
        let Some(target) = popup_dismiss_target_for_node(node, bounds) else {
            continue;
        };
        let is_hovered_popup =
            node.control_id.as_str() == interaction.hovered_template_control_id.as_str();
        if is_hovered_popup || node.focused || node.selected {
            return Some(target);
        }
    }
    None
}

fn popup_dismiss_target_for_node(
    node: &TemplatePaneNodeData,
    bounds: &FrameRect,
) -> Option<PopupDismissTarget> {
    if !node.popup_open || node.disabled || node.control_id.is_empty() {
        return None;
    }

    let control_frame = frame_from_template_node(node);
    let popup_frame = if node.structured_options.row_count() > 0 {
        template_option_popup_frame_within(
            node,
            &control_frame,
            node.structured_options.row_count(),
            bounds,
        )
        .unwrap_or_else(|| control_frame.clone())
    } else if node.structured_menu_items.row_count() > 0 {
        control_frame.clone()
    } else {
        return None;
    };
    let damage_frame = union_frame(&control_frame, &popup_frame);
    Some(PopupDismissTarget {
        control_id: node.control_id.clone(),
        control_frame,
        popup_frame,
        damage_frame,
    })
}
