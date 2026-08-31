use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::passive::dispatch_passive_pane_target_button;
use super::input::ResultPaneTargetInput;

pub(super) fn dispatch_passive_result_target(
    input: ResultPaneTargetInput<'_, '_, '_>,
) -> Option<NativePointerDispatchResult> {
    dispatch_passive_pane_target_button(input.pointer, input.state, input.cleared_text_input_frame)
}
