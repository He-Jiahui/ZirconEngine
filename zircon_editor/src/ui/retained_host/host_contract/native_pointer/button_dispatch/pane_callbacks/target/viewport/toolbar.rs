use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::ViewportToolbarPointerHit;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::super::NativePointerButtonState;
use super::super::super::viewport::dispatch_viewport_toolbar_button;

pub(super) fn dispatch_viewport_toolbar_target_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    presentation: &HostWindowPresentationData,
    pointer: &PanePointerRoute,
    hit: &ViewportToolbarPointerHit,
    state: NativePointerButtonState,
    button: UiPointerButton,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    dispatch_viewport_toolbar_button(
        pane_host,
        presentation,
        pointer,
        hit,
        state,
        button,
        cleared_text_input_frame,
    )
}
