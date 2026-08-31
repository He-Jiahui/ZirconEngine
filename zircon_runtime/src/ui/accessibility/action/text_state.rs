use zircon_runtime_interface::ui::{
    accessibility::UiA11yTextSelection,
    binding::UiBindingSourceKind,
    dispatch::UiInputDispatchResult,
    event_ui::UiNodeId,
    surface::{UiEditableTextState, UiTextCaretAffinity, UiTextRange, UiTextSelection},
};

use crate::ui::surface::{
    UiSurface,
    input::{
        UiEditableTextTransactionError, commit_editable_text_transaction,
        editable_text_state_for_node,
    },
};
use crate::ui::{
    dispatch::UiTextDocumentSession,
    text::{CommittedTextEditIntent, clamp_grapheme_boundary},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiAccessibilityTextStateError {
    MissingEditableState,
    Transaction(UiEditableTextTransactionError),
}

impl UiAccessibilityTextStateError {
    pub(super) const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::MissingEditableState => "missing_editable_text_state",
            Self::Transaction(error) => error.diagnostic_code(),
        }
    }

    pub(super) const fn reason(self) -> &'static str {
        match self {
            Self::MissingEditableState => "target has no retained editable text state",
            Self::Transaction(_) => "editable text state transaction was rejected",
        }
    }
}

pub(super) fn text_input_is_read_only(surface: &UiSurface, target: UiNodeId) -> bool {
    editable_text_state_for_node(surface, target).is_some_and(|state| state.read_only)
}

pub(super) fn collapsed_text_state(
    surface: &UiSurface,
    target: UiNodeId,
    text: String,
    collapse_offset: usize,
) -> Result<UiEditableTextState, UiAccessibilityTextStateError> {
    selection_text_state(
        surface,
        target,
        text,
        collapse_offset,
        collapse_offset,
        collapse_offset,
    )
}

pub(super) fn selected_text_state(
    surface: &UiSurface,
    target: UiNodeId,
    caret: usize,
    anchor: usize,
    focus: usize,
) -> Result<UiEditableTextState, UiAccessibilityTextStateError> {
    let text = editable_text_state_for_node(surface, target)
        .ok_or(UiAccessibilityTextStateError::MissingEditableState)?
        .text;
    selection_text_state(surface, target, text, caret, anchor, focus)
}

fn selection_text_state(
    surface: &UiSurface,
    target: UiNodeId,
    text: String,
    caret: usize,
    anchor: usize,
    focus: usize,
) -> Result<UiEditableTextState, UiAccessibilityTextStateError> {
    let mut state = editable_text_state_for_node(surface, target)
        .ok_or(UiAccessibilityTextStateError::MissingEditableState)?;
    state.text = text;
    state.caret.offset = caret;
    // Accessibility offsets identify logical boundaries, never visual bidi edges.
    state.caret.affinity = UiTextCaretAffinity::Downstream;
    state.selection = Some(UiTextSelection { anchor, focus });
    state.composition = None;
    Ok(state)
}

pub(super) fn commit_accessibility_text_state(
    surface: &mut UiSurface,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    result: &mut UiInputDispatchResult,
) -> Result<bool, UiAccessibilityTextStateError> {
    commit_accessibility_text_state_transaction(
        surface,
        None,
        target,
        value_property,
        state,
        None,
        result,
    )
}

pub(super) fn commit_accessibility_text_edit(
    surface: &mut UiSurface,
    text_documents: Option<&mut UiTextDocumentSession>,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    committed_edit: Option<CommittedTextEditIntent>,
    result: &mut UiInputDispatchResult,
) -> Result<bool, UiAccessibilityTextStateError> {
    commit_accessibility_text_state_transaction(
        surface,
        text_documents,
        target,
        value_property,
        state,
        committed_edit,
        result,
    )
}

#[allow(clippy::too_many_arguments)]
fn commit_accessibility_text_state_transaction(
    surface: &mut UiSurface,
    text_documents: Option<&mut UiTextDocumentSession>,
    target: UiNodeId,
    value_property: &str,
    state: &UiEditableTextState,
    committed_edit: Option<CommittedTextEditIntent>,
    result: &mut UiInputDispatchResult,
) -> Result<bool, UiAccessibilityTextStateError> {
    let transaction = commit_editable_text_transaction(
        surface,
        text_documents,
        target,
        value_property,
        state,
        UiBindingSourceKind::AccessibilityAction,
        committed_edit,
        result,
    )
    .map_err(UiAccessibilityTextStateError::Transaction)?;
    if let Some(binding_report) = transaction.binding_report {
        result.diagnostics.notes.push(format!(
            "accessibility_binding_updates:applied={},unchanged={},rejected={}",
            binding_report.applied_count,
            binding_report.unchanged_count,
            binding_report.rejected_count
        ));
        result
            .diagnostics
            .notes
            .push("accessibility_binding_source:AccessibilityAction".to_string());
        result.record_binding_report(binding_report);
    }
    for (property, dirty) in transaction.changed_properties {
        result.diagnostics.notes.push(format!(
            "accessibility_text_state_changed:{property}:{dirty:?}"
        ));
    }
    Ok(transaction.value_changed)
}

pub(super) fn text_selection_range(
    text: &str,
    selection: Option<&UiA11yTextSelection>,
) -> UiTextRange {
    let selection = selection
        .cloned()
        .unwrap_or_else(|| UiA11yTextSelection::collapsed(text.len()));
    let anchor = clamp_grapheme_boundary(text, selection.anchor);
    let focus = clamp_grapheme_boundary(text, selection.focus);
    UiTextRange {
        start: anchor.min(focus),
        end: anchor.max(focus),
    }
}
