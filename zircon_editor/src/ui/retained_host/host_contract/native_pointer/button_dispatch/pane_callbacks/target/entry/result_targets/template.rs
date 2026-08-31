use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::template::dispatch_template_pane_target_button;
use super::input::ResultPaneTargetInput;

pub(super) fn dispatch_template_result_target(
    input: &ResultPaneTargetInput<'_, '_, '_>,
) -> Option<NativePointerDispatchResult> {
    dispatch_template_pane_target_button(
        input.ui,
        input.pane_host,
        input.pointer,
        input.state,
        input.button,
        input.cleared_text_input_frame.clone(),
    )
}
