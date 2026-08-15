use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::super::NativePointerButtonState;
use super::click::invoke_viewport_toolbar_click;
use super::damage::viewport_toolbar_click_damage_result;

pub(in crate::ui::retained_host::host_contract) fn dispatch_viewport_toolbar_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    presentation: &HostWindowPresentationData,
    pointer: &PanePointerRoute,
    surface_key: &SharedString,
    control_id: Option<&SharedString>,
    state: NativePointerButtonState,
    button: UiPointerButton,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    if state != NativePointerButtonState::Pressed || button != UiPointerButton::Primary {
        return NativePointerDispatchResult::idle();
    }
    let routed_control_id =
        invoke_viewport_toolbar_click(pane_host, pointer, surface_key, control_id);
    viewport_toolbar_click_damage_result(
        presentation,
        routed_control_id.as_str(),
        &pointer.frame,
        cleared_text_input_frame,
    )
}
