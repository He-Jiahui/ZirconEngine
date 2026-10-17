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
    unreal_viewport_overflow_dismiss_target(nodes)
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

/// The authored More popup is a real source ScrollableBox, without synthetic
/// menu/options rows. Require its exact source ancestry and common frame owner.
pub(in crate::ui::retained_host::host_contract) fn unreal_viewport_overflow_dismiss_target(
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> Option<PopupDismissTarget> {
    let toolbar = nodes.iter().find(|node| {
        node.control_id.as_str() == "UnrealViewportToolbar"
            && node.source_node_id.as_str() == "toolbar"
    })?;
    let row = nodes.iter().find(|node| {
        node.control_id.as_str() == "UnrealViewportToolbarRow"
            && node.parent_node_id == toolbar.node_id
    })?;
    let popup = nodes.iter().find(|node| {
        node.control_id.as_str() == "UnrealViewportToolbarOverflow"
            && node.toolbar_overflow_role.as_str() == "toolbar-overflow"
            && node.popup_open
            && !node.disabled
            && node.parent_node_id == toolbar.node_id
    })?;
    let more = nodes.iter().find(|node| {
        node.control_id.as_str() == "UnrealViewportToolbarMore"
            && node.parent_node_id == row.node_id
            && !node.disabled
    })?;
    if !toolbar
        .source_path
        .as_str()
        .ends_with("ui/editor/unreal/components/viewport.zui")
        || popup.source_node_id.as_str() != "toolbar_overflow"
        || row.source_node_id.as_str() != "toolbar_row"
        || more.source_node_id.as_str() != "toolbar_more"
        || popup.source_path != toolbar.source_path
        || row.source_path != toolbar.source_path
        || more.source_path != toolbar.source_path
        || popup.instance_path != toolbar.instance_path
        || row.instance_path != toolbar.instance_path
        || more.instance_path != toolbar.instance_path
    {
        return None;
    }
    let owner = toolbar.source_surface_frame.as_ref()?;
    // Transparent source containers may have no render-command owner anchor.
    // Validate every ID against the one immutable published arranged tree.
    for projected in [&toolbar, &row, &popup, &more] {
        let arranged = owner.arranged_tree.get(projected.surface_node_id?)?;
        if arranged.node_path.0 != projected.node_id.as_str()
            || arranged.control_id.as_deref() != Some(projected.control_id.as_str())
        {
            return None;
        }
        if projected
            .source_surface_frame
            .as_ref()
            .is_some_and(|bound| !std::sync::Arc::ptr_eq(owner, bound))
        {
            return None;
        }
    }
    let popup_frame = frame_from_template_node(&popup);
    let control_frame = frame_from_template_node(&more);
    if [
        popup_frame.x,
        popup_frame.y,
        control_frame.x,
        control_frame.y,
    ]
    .iter()
    .any(|axis| !axis.is_finite())
    {
        return None;
    }
    if [
        popup_frame.width,
        popup_frame.height,
        control_frame.width,
        control_frame.height,
    ]
    .iter()
    .any(|extent| !extent.is_finite() || *extent <= 0.0)
    {
        return None;
    }
    let damage_frame = union_frame(&control_frame, &popup_frame);
    Some(PopupDismissTarget {
        control_id: popup.control_id.clone(),
        control_frame,
        popup_frame,
        damage_frame,
    })
}
