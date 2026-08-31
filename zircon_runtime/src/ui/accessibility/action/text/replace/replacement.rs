use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityNode, dispatch::UiTextInputConstraintReceipt,
    surface::UiTextRange,
};

use crate::ui::{
    surface::input::{TextInputConstraints, TextInputRetainedGraphemeCount},
    text::CommittedTextEditIntent,
};

pub(super) struct SelectedTextReplacement {
    pub(super) text: String,
    pub(super) caret_offset: usize,
    pub(super) committed_edit: Option<CommittedTextEditIntent>,
    pub(super) constraint_note: Option<&'static str>,
    pub(super) constraint_receipt: Option<UiTextInputConstraintReceipt>,
}

pub(super) fn selected_text_replacement(
    snapshot_node: &UiAccessibilityNode,
    selected_range: UiTextRange,
    replacement: &str,
    constraints: TextInputConstraints,
    retained_graphemes: TextInputRetainedGraphemeCount,
) -> SelectedTextReplacement {
    let current_text = snapshot_node.state.value.as_deref().unwrap_or_default();
    let sanitized = constraints.sanitize_replacement_with_retained_grapheme_count(
        current_text,
        selected_range,
        replacement,
        retained_graphemes,
    );
    let constraint_receipt = sanitized.receipt_if_changed();
    let constraint_note = constraint_receipt
        .is_some()
        .then_some("accessibility_replace_selected_text_sanitized");
    let sanitized = sanitized.text;
    let committed_edit = (current_text.get(selected_range.start..selected_range.end)
        != Some(sanitized.as_str()))
    .then(|| {
        CommittedTextEditIntent::for_replacement(
            selected_range.start..selected_range.end,
            sanitized.len(),
        )
    });
    let mut text = String::with_capacity(
        current_text.len() - (selected_range.end - selected_range.start) + sanitized.len(),
    );
    text.push_str(&current_text[..selected_range.start]);
    text.push_str(&sanitized);
    text.push_str(&current_text[selected_range.end..]);
    let caret_offset = selected_range.start + sanitized.len();
    SelectedTextReplacement {
        text,
        caret_offset,
        committed_edit,
        constraint_note,
        constraint_receipt,
    }
}
