use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::fallback::cleared_text_input_fallback_result;
use super::input::BodyButtonRouteInput;
use super::pane::dispatch_pane_body_route;
use super::workbench::dispatch_workbench_body_route;

pub(super) fn dispatch_body_button_route_sequence(
    input: BodyButtonRouteInput<'_>,
) -> NativePointerDispatchResult {
    if let Some(result) = dispatch_workbench_body_route(&input) {
        return result;
    }
    if let Some(result) = dispatch_pane_body_route(&input) {
        return result;
    }
    cleared_text_input_fallback_result(input.cleared_text_input_frame)
}
