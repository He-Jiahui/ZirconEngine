use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::ViewportToolbarPointerHit;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::routing::PanePointerRoute;
use super::super::super::viewport_toolbar_damage::viewport_toolbar_press_damage_frame;
use super::super::super::NativePointerButtonState;

pub(super) fn dispatch_viewport_toolbar_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    presentation: &HostWindowPresentationData,
    pointer: &PanePointerRoute,
    hit: &ViewportToolbarPointerHit,
    state: NativePointerButtonState,
    button: UiPointerButton,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    if state != NativePointerButtonState::Pressed || button != UiPointerButton::Primary {
        return NativePointerDispatchResult::idle();
    }
    let control_id = hit.control_id.clone();
    pane_host.invoke_viewport_toolbar_pointer_clicked(
        hit.surface_key.clone(),
        hit.control_id.clone(),
        hit.control_x,
        hit.control_y,
        hit.control_width,
        hit.control_height,
        pointer.local_x,
        pointer.local_y,
    );
    match viewport_toolbar_press_damage_frame(
        presentation,
        control_id.as_str(),
        &pointer.frame,
        cleared_text_input_frame,
    ) {
        Some(damage) => NativePointerDispatchResult::region_with_frame_update(damage),
        None => NativePointerDispatchResult::full_frame(),
    }
}

pub(super) fn dispatch_viewport_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    kind: i32,
    button_id: i32,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    pane_host.invoke_viewport_pointer_event(kind, button_id, pointer.local_x, pointer.local_y, 0.0);
    if let Some(frame) = cleared_text_input_frame {
        return NativePointerDispatchResult::region(frame);
    }
    NativePointerDispatchResult::idle()
}
