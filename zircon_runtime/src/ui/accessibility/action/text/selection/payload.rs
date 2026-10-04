use zircon_runtime_interface::ui::accessibility::{
    UiAccessibilityActionRequest, UiAccessibilityNode,
};

use crate::ui::text::clamp_grapheme_boundary;

pub(super) const MISSING_TEXT_SELECTION_CODE: &str = "missing_text_selection";
pub(super) const MISSING_TEXT_SELECTION_REASON: &str =
    "set text selection action requires text_selection";

pub(super) struct SetTextSelectionPayload {
    pub(super) caret: usize,
    pub(super) anchor: usize,
    pub(super) focus: usize,
}

// 外部选择下标按当前快照文本夹到字素边界；后续提交仍需确认目标的真实可编辑状态。
pub(super) fn set_text_selection_payload(
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
) -> Option<SetTextSelectionPayload> {
    let selection = request.text_selection.as_ref()?;
    let text = snapshot_node.state.value.as_deref().unwrap_or_default();
    Some(SetTextSelectionPayload {
        caret: clamp_grapheme_boundary(text, selection.caret),
        anchor: clamp_grapheme_boundary(text, selection.anchor),
        focus: clamp_grapheme_boundary(text, selection.focus),
    })
}
