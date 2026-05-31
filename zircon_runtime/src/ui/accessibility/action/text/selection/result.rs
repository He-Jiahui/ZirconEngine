use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus, dispatch::UiInputDispatchResult, event_ui::UiNodeId,
};

use crate::ui::surface::UiSurface;

use super::super::super::{
    result::{finish_handled, finish_unhandled},
    text_state::sync_text_input_selection_metadata,
};
use super::payload::{MISSING_TEXT_SELECTION_CODE, MISSING_TEXT_SELECTION_REASON};

pub(super) fn finish_missing_text_selection(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        MISSING_TEXT_SELECTION_CODE,
        MISSING_TEXT_SELECTION_REASON,
    )
}

pub(super) fn finish_set_text_selection(
    surface: &mut UiSurface,
    target: UiNodeId,
    caret: usize,
    anchor: usize,
    focus: usize,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.set_text_selection");
    sync_text_input_selection_metadata(surface, target, caret, anchor, focus, &mut result);
    result
}
