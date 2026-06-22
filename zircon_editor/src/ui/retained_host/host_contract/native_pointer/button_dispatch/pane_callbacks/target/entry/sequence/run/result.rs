use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::input::PaneButtonDispatchInput;
use super::super::super::result_targets::dispatch_result_pane_targets;

pub(super) fn dispatch_result_target(
    pane_host: &PaneSurfaceHostContext<'_>,
    input: &PaneButtonDispatchInput<'_>,
    kind: i32,
) -> Option<NativePointerDispatchResult> {
    dispatch_result_pane_targets(
        input.ui,
        pane_host,
        input.presentation,
        &input.pointer,
        kind,
        input.state,
        input.button,
        input.button_id,
        input.cleared_text_input_frame.clone(),
    )
}
