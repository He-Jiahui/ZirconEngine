use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::input::ResultPaneTargetInput;
use super::passive::dispatch_passive_result_target;
use super::template::dispatch_template_result_target;
use super::viewport::dispatch_viewport_result_target;

pub(super) fn dispatch_result_pane_target_sequence(
    input: ResultPaneTargetInput<'_, '_, '_>,
) -> Option<NativePointerDispatchResult> {
    if let Some(result) = dispatch_viewport_result_target(&input) {
        return Some(result);
    }
    if let Some(result) = dispatch_template_result_target(&input) {
        return Some(result);
    }
    dispatch_passive_result_target(input)
}
