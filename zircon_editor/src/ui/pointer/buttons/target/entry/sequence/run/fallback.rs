use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::super::super::fallback::pane_button_fallback_damage;
use super::super::super::input::PaneButtonDispatchInput;

pub(super) fn pane_button_sequence_fallback(
    input: PaneButtonDispatchInput<'_>,
) -> NativePointerDispatchResult {
    pane_button_fallback_damage(
        input.presentation,
        &input.pointer,
        input.state,
        input.cleared_text_input_frame,
    )
}
