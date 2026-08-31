use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityNode, component::UiValue, dispatch::UiTextInputConstraintReceipt,
    event_ui::UiNodeId, surface::UiTextRange,
};

use crate::ui::{
    surface::{UiSurface, input::TextInputRetainedGraphemeCount, text_input_constraints_for_node},
    text::CommittedTextEditIntent,
};

use super::super::text_state::text_input_is_read_only;

pub(super) struct TextInputSetValue {
    pub(super) text: String,
    pub(super) committed_edit: Option<CommittedTextEditIntent>,
    pub(super) constraint_note: Option<&'static str>,
    pub(super) constraint_receipt: Option<UiTextInputConstraintReceipt>,
}

pub(super) struct TextInputSetValueRejection {
    pub(super) code: &'static str,
    pub(super) reason: &'static str,
}

pub(super) fn prepare_text_input_set_value(
    surface: &UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    value: UiValue,
) -> Result<TextInputSetValue, TextInputSetValueRejection> {
    if text_input_is_read_only(surface, target) {
        return Err(TextInputSetValueRejection {
            code: "read_only",
            reason: "text input is read-only",
        });
    }

    let UiValue::String(text) = value else {
        return Err(TextInputSetValueRejection {
            code: "invalid_text_value",
            reason: "text input set value requires a string payload",
        });
    };
    let current_text = snapshot_node.state.value.as_deref().unwrap_or_default();
    let sanitized = text_input_constraints_for_node(surface, target)
        .sanitize_replacement_with_retained_grapheme_count(
            current_text,
            UiTextRange {
                start: 0,
                end: current_text.len(),
            },
            &text,
            TextInputRetainedGraphemeCount::DocumentIndex(0),
        );
    let constraint_receipt = sanitized.receipt_if_changed();
    let constraint_note = constraint_receipt
        .is_some()
        .then_some("accessibility_text_value_sanitized");
    let text = sanitized.text;
    let committed_edit = (current_text != text)
        .then(|| CommittedTextEditIntent::for_replacement(0..current_text.len(), text.len()));

    Ok(TextInputSetValue {
        text,
        committed_edit,
        constraint_note,
        constraint_receipt,
    })
}
