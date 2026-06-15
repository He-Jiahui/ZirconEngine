use super::data::{
    FrameRect, HostPaneInteractionStateData, HostWindowPresentationData, TemplatePaneNodeData,
};
use super::frame_geometry::{contains_point, union_frame, union_optional_frames};
use super::globals::PaneSurfaceHostContext;
use super::redraw::NativePointerDispatchResult;
use super::template_geometry::{frame_from_template_node, template_popup_bounds};
use super::template_popup_layout::template_option_popup_frame_within;
use super::window::UiHostWindow;
use crate::ui::retained_host::primitives::SharedString;
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;

pub(super) fn dispatch_workbench_popup_outside_primary_press(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    extra_damage: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    let interaction = ui.get_pane_interaction_state();
    let bounds = template_popup_bounds(
        &presentation.host_shell.native_window_bounds,
        &presentation.workbench_window_nodes,
    );
    let target = active_popup_dismiss_target(presentation, &interaction, &bounds)?;
    if target.contains_point(x, y) {
        return None;
    }

    let damage_frame = target.damage_frame.clone();
    let damage =
        union_optional_frames(extra_damage, Some(damage_frame.clone())).unwrap_or(damage_frame);
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    pane_host
        .invoke_surface_control_clicked(target.control_id, WORKBENCH_POPUP_CANCEL_ACTION_ID.into());
    ui.clear_hovered_template_node_for_pointer_move();
    Some(NativePointerDispatchResult::region_with_frame_update(
        damage,
    ))
}

/// Keeps hit containment separate from repaint damage: dropdowns contain both
/// their trigger and popup rows, while repaint needs the union of both regions.
struct PopupDismissTarget {
    control_id: SharedString,
    control_frame: FrameRect,
    popup_frame: FrameRect,
    damage_frame: FrameRect,
}

impl PopupDismissTarget {
    fn contains_point(&self, x: f32, y: f32) -> bool {
        contains_point(&self.control_frame, x, y) || contains_point(&self.popup_frame, x, y)
    }
}

fn active_popup_dismiss_target(
    presentation: &HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    bounds: &FrameRect,
) -> Option<PopupDismissTarget> {
    let nodes = &presentation.workbench_window_nodes;
    for row in (0..nodes.row_count()).rev() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        let Some(target) = popup_dismiss_target_for_node(&node, bounds) else {
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
