use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::pane_route::dispatch_pane_route_button;
use super::input::BodyButtonRouteInput;

pub(super) fn dispatch_pane_body_route(
    input: &BodyButtonRouteInput<'_>,
) -> Option<NativePointerDispatchResult> {
    dispatch_pane_route_button(
        input.ui,
        input.presentation,
        input.state,
        input.button,
        input.button_id,
        input.modifiers,
        input.x,
        input.y,
        input.cleared_text_input_frame.clone(),
    )
}
