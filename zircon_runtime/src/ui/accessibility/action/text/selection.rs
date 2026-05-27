use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityNode,
    },
    dispatch::UiInputDispatchResult,
};

use crate::ui::surface::UiSurface;

use super::super::{
    result::{finish_handled, finish_unhandled, unsupported_role_action},
    text_state::sync_text_input_selection_metadata,
};

use self::payload::{
    set_text_selection_payload, MISSING_TEXT_SELECTION_CODE, MISSING_TEXT_SELECTION_REASON,
    MISSING_TEXT_SELECTION_STATUS,
};

mod payload;

pub(in crate::ui::accessibility::action) fn dispatch_set_text_selection(
    surface: &mut UiSurface,
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let target = request.target;
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::SetTextSelection)
    {
        return unsupported_role_action(
            result,
            target,
            "target does not expose set text selection action",
        );
    }
    if snapshot_node.role != UiA11yRole::TextInput {
        return unsupported_role_action(
            result,
            target,
            "set text selection requires text input role",
        );
    }
    let Some(selection) = set_text_selection_payload(request, snapshot_node) else {
        return finish_unhandled(
            result,
            Some(target),
            MISSING_TEXT_SELECTION_STATUS,
            MISSING_TEXT_SELECTION_CODE,
            MISSING_TEXT_SELECTION_REASON,
        );
    };

    let mut result = finish_handled(result, target, "accessibility.set_text_selection");
    sync_text_input_selection_metadata(
        surface,
        target,
        selection.caret,
        selection.anchor,
        selection.focus,
        &mut result,
    );
    result
}
