use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::super::NativePointerButtonState;
use super::super::super::viewport::dispatch_viewport_toolbar_button;

pub(super) fn dispatch_viewport_toolbar_target_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    presentation: &HostWindowPresentationData,
    pointer: &PanePointerRoute,
    surface_key: &str,
    control_id: Option<&str>,
    state: NativePointerButtonState,
    button: UiPointerButton,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    dispatch_viewport_toolbar_button(
        pane_host,
        presentation,
        pointer,
        surface_key,
        control_id,
        state,
        button,
        cleared_text_input_frame,
    )
}
