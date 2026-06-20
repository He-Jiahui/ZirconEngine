use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::frame_geometry::union_optional_frames;
use super::super::globals::PaneSurfaceHostContext;
use super::super::redraw::NativePointerDispatchResult;
use super::super::template_geometry::template_popup_bounds;
use super::super::window::UiHostWindow;
use super::target::active_popup_dismiss_target;
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;

pub(in crate::ui::retained_host::host_contract) fn dispatch_workbench_popup_outside_primary_press(
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
