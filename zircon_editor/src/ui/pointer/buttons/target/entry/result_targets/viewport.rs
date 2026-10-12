use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::viewport::dispatch_viewport_pane_target_button;
use super::input::ResultPaneTargetInput;

pub(super) fn dispatch_viewport_result_target(
    input: &ResultPaneTargetInput<'_, '_, '_>,
) -> Option<NativePointerDispatchResult> {
    dispatch_viewport_pane_target_button(
        input.pane_host,
        input.presentation,
        input.pointer,
        input.kind,
        input.state,
        input.button,
        input.button_id,
        input.modifiers,
        input.cleared_text_input_frame.clone(),
    )
}
