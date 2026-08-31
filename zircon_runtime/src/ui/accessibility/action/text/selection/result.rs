use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus, dispatch::UiInputDispatchResult, event_ui::UiNodeId,
};

use crate::ui::surface::UiSurface;

use super::super::super::{
    result::{finish_handled, finish_unhandled},
    text_state::{commit_accessibility_text_state, selected_text_state},
    value_target::set_value_property,
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
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let Some(value_property) = set_value_property(surface, target) else {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "missing_value_property",
            "target has no retained editable text value property",
        );
    };
    let state = match selected_text_state(surface, target, caret, anchor, focus) {
        Ok(state) => state,
        Err(error) => {
            return finish_unhandled(
                result,
                Some(target),
                UiAccessibilityActionStatus::Rejected,
                error.diagnostic_code(),
                error.reason(),
            );
        }
    };
    if let Err(error) = commit_accessibility_text_state(
        surface,
        target,
        value_property.as_str(),
        &state,
        &mut result,
    ) {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            error.diagnostic_code(),
            error.reason(),
        );
    }
    let mut result = finish_handled(result, target, "accessibility.set_text_selection");
    surface.redact_secure_text_result(target, &mut result);
    result
}
