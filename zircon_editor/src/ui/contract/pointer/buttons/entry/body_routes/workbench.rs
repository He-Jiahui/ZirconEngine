use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::workbench::dispatch_workbench_button;
use super::input::BodyButtonRouteInput;

pub(super) fn dispatch_workbench_body_route(
    input: &BodyButtonRouteInput<'_>,
) -> Option<NativePointerDispatchResult> {
    dispatch_workbench_button(
        input.ui,
        input.presentation,
        input.state,
        input.button,
        input.x,
        input.y,
        input.cleared_text_input_frame.clone(),
    )
}
