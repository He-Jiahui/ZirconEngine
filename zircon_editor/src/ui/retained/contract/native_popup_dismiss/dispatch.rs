use super::super::componentized_workbench_regions::{
    module_tools_menu_admission_is_live, module_tools_menu_spec,
    module_tools_menu_spec_for_trigger, module_tools_popups,
};
use super::super::data::{FrameRect, HostPresentationGeneration};
use super::super::frame_geometry::union_optional_frames;
use super::super::globals::PaneSurfaceHostContext;
use super::super::redraw::NativePointerDispatchResult;
use super::super::surface_hit_test::hit_test_workbench_window_template_node_with_index;
use super::super::template_geometry::template_popup_bounds;
use super::super::window::UiHostWindow;
use super::target::active_popup_dismiss_target;
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;

pub(in crate::ui::retained_host::host_contract) fn dispatch_workbench_popup_outside_primary_press(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
    extra_damage: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    let presentation = generation.structure();
    let interaction = generation.pane_interaction_state();
    let popup_rows = generation.workbench_hit_index().popup_rows();
    let bounds = template_popup_bounds(
        &presentation.host_shell.native_window_bounds,
        &presentation.workbench_window_nodes,
    );
    let mut target = active_popup_dismiss_target(
        &presentation.workbench_window_nodes,
        interaction,
        &bounds,
        popup_rows,
    )?;
    let is_module_tools_popup = module_tools_menu_spec(target.control_id.as_str()).is_some();
    if is_module_tools_popup {
        // The same live foreground proof owns paint, hit and outside-click containment.
        let popup = module_tools_popups(presentation, &bounds)
            .into_iter()
            .find(|popup| {
                presentation
                    .workbench_window_nodes
                    .get(popup.row)
                    .is_some_and(|node| node.control_id == target.control_id)
            })?;
        let clip = generation
            .workbench_hit_index()
            .module_tools_popup_clip(popup.row)?;
        if clip != &popup.clip {
            return None;
        }
        let visible = popup.visible_frame()?;
        target.control_frame = visible.clone();
        target.popup_frame = visible.clone();
        target.damage_frame = visible;
    }
    if target.contains_point(x, y) {
        return None;
    }
    if is_module_tools_popup {
        // A source-proven sibling trigger must reach its existing press/release owner.
        // Foreground popup hits still win; all other outside presses retain clickaway.
        let hit = hit_test_workbench_window_template_node_with_index(
            presentation,
            generation.workbench_hit_index(),
            x,
            y,
        );
        if hit.as_ref().is_some_and(|hit| {
            module_tools_menu_spec_for_trigger(hit.control_id.as_str()).is_some_and(|spec| {
                module_tools_menu_admission_is_live(presentation, spec.menu_control_id)
            })
        }) {
            return None;
        }
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
