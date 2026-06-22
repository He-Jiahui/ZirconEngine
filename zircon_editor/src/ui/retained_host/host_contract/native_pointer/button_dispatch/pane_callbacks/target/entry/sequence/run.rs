mod callback;
mod fallback;
mod result;

use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use self::callback::dispatch_callback_target_fallback;
use self::fallback::pane_button_sequence_fallback;
use self::result::dispatch_result_target;
use super::super::super::super::kind::{host_pointer_kind, viewport_pointer_kind};
use super::super::input::PaneButtonDispatchInput;

pub(super) fn dispatch_pane_button_sequence(
    input: PaneButtonDispatchInput<'_>,
) -> NativePointerDispatchResult {
    let kind = viewport_pointer_kind(input.state);
    let host_kind = host_pointer_kind(input.state);
    let pane_host = input.ui.global::<PaneSurfaceHostContext>();

    if let Some(result) = dispatch_callback_target_fallback(&pane_host, &input, host_kind) {
        return result;
    }

    if let Some(result) = dispatch_result_target(&pane_host, &input, kind) {
        return result;
    }

    pane_button_sequence_fallback(input)
}
