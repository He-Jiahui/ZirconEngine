use zircon_runtime_interface::ui::accessibility::{
    UiAccessibilityActionRequest, UiAccessibilityActionStatus, UiAccessibilityNode,
};

use super::super::super::text_state::clamp_text_boundary;

pub(super) const MISSING_TEXT_SELECTION_STATUS: UiAccessibilityActionStatus =
    UiAccessibilityActionStatus::Rejected;
pub(super) const MISSING_TEXT_SELECTION_CODE: &str = "missing_text_selection";
pub(super) const MISSING_TEXT_SELECTION_REASON: &str =
    "set text selection action requires text_selection";

pub(super) struct SetTextSelectionPayload {
    pub(super) caret: usize,
    pub(super) anchor: usize,
    pub(super) focus: usize,
}

pub(super) fn set_text_selection_payload(
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
) -> Option<SetTextSelectionPayload> {
    let selection = request.text_selection.as_ref()?;
    let text = snapshot_node.state.value.as_deref().unwrap_or_default();
    Some(SetTextSelectionPayload {
        caret: clamp_text_boundary(text, selection.caret),
        anchor: clamp_text_boundary(text, selection.anchor),
        focus: clamp_text_boundary(text, selection.focus),
    })
}
