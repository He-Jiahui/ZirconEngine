use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityNode,
    },
    dispatch::UiInputDispatchResult,
};

use crate::ui::surface::UiSurface;

use super::super::result::unsupported_role_action;

use self::payload::set_text_selection_payload;
use self::result::{finish_missing_text_selection, finish_set_text_selection};

mod payload;
mod result;

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
        return finish_missing_text_selection(result, target);
    };

    finish_set_text_selection(
        surface,
        target,
        selection.caret,
        selection.anchor,
        selection.focus,
        result,
    )
}
