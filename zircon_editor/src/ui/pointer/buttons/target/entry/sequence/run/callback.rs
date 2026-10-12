use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::super::super::fallback::pane_button_fallback_damage;
use super::super::super::callback_targets::dispatch_callback_pane_targets;
use super::super::super::input::PaneButtonDispatchInput;

pub(super) fn dispatch_callback_target_fallback(
    pane_host: &PaneSurfaceHostContext<'_>,
    input: &PaneButtonDispatchInput<'_>,
    host_kind: i32,
) -> Option<NativePointerDispatchResult> {
    if !dispatch_callback_pane_targets(
        pane_host,
        &input.pointer,
        input.state,
        input.button,
        host_kind,
        input.button_id,
    ) {
        return None;
    }
    Some(pane_button_fallback_damage(
        input.presentation,
        &input.pointer,
        input.state,
        input.cleared_text_input_frame.clone(),
    ))
}
