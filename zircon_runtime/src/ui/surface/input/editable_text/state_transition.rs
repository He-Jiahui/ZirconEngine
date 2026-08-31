use std::borrow::Cow;

use zircon_runtime_interface::ui::{
    dispatch::{
        UiImeDeleteSurrounding, UiImePreeditClause, UiTextByteRange, UiTextInputConstraintReceipt,
    },
    surface::{UiEditableTextState, UiTextEditAction, UiTextRange, UiTextSelection},
};

use crate::ui::text::{
    CommittedTextEditIntent, TextEditStateTransition, apply_text_edit_action,
    apply_text_edit_action_with_intent, apply_text_edit_actions_with_intent,
    clamp_grapheme_boundary, next_grapheme_boundary, previous_grapheme_boundary,
};

use super::super::text_constraints::{
    SanitizedTextInputPreedit, SanitizedTextInputReplacement, TextInputConstraints,
    TextInputRetainedGraphemeCount,
};

pub(super) struct TextInputStateTransition {
    pub(super) state: UiEditableTextState,
    pub(super) constraint_receipt: UiTextInputConstraintReceipt,
    pub(super) committed_edit: Option<CommittedTextEditIntent>,
}

impl TextInputStateTransition {
    pub(super) fn unchanged(state: UiEditableTextState) -> Self {
        Self {
            state,
            constraint_receipt: UiTextInputConstraintReceipt::default(),
            committed_edit: None,
        }
    }

    pub(super) fn from_edit(transition: TextEditStateTransition) -> Self {
        Self {
            state: transition.state,
            constraint_receipt: UiTextInputConstraintReceipt::default(),
            committed_edit: transition.committed,
        }
    }
}

pub(super) fn committed_text_state(
    editable: UiEditableTextState,
    text: String,
    constraints: TextInputConstraints,
    retained_graphemes: TextInputRetainedGraphemeCount,
) -> TextInputStateTransition {
    let (transition, constraint_receipt) = if editable.composition.is_some() {
        let range = committed_text_replaced_range(&editable);
        let SanitizedTextInputReplacement { text, receipt } = constraints
            .sanitize_replacement_with_retained_grapheme_count(
                &editable.text,
                range,
                &text,
                retained_graphemes,
            );
        let composed = apply_text_edit_action_with_intent(
            editable,
            UiTextEditAction::SetComposition { range, text },
        );
        (
            apply_text_edit_action_with_intent(composed.state, UiTextEditAction::CommitComposition),
            receipt,
        )
    } else {
        let range = committed_text_replaced_range(&editable);
        let SanitizedTextInputReplacement { text, receipt } = constraints
            .sanitize_replacement_with_retained_grapheme_count(
                &editable.text,
                range,
                &text,
                retained_graphemes,
            );
        (
            apply_text_edit_action_with_intent(editable, UiTextEditAction::Insert { text }),
            receipt,
        )
    };
    TextInputStateTransition {
        state: transition.state,
        constraint_receipt,
        committed_edit: transition.committed,
    }
}

pub(super) fn preedit_text_state(
    editable: UiEditableTextState,
    preedit: &str,
    cursor_range: Option<UiTextByteRange>,
    preedit_clauses: &[UiImePreeditClause],
    constraints: TextInputConstraints,
    retained_graphemes: TextInputRetainedGraphemeCount,
) -> TextInputStateTransition {
    let range = preedit_text_replaced_range(&editable);
    let SanitizedTextInputPreedit {
        text: sanitized_preedit,
        cursor_range,
        preedit_clauses,
        receipt: constraint_receipt,
    } = constraints.sanitize_preedit_replacement_with_retained_grapheme_count(
        &editable.text,
        range,
        preedit,
        cursor_range,
        preedit_clauses,
        retained_graphemes,
    );
    let mut next = apply_text_edit_action(
        editable,
        UiTextEditAction::SetComposition {
            range,
            text: sanitized_preedit,
        },
    );
    if let Some(composition) = next.composition.as_mut() {
        composition.preedit_clauses = preedit_clauses;
    }

    if let Some(cursor_range) = cursor_range {
        if let Some(composition) = next.composition.as_ref() {
            let anchor = composition.range.start + cursor_range.start_byte as usize;
            let focus = composition.range.start + cursor_range.end_byte as usize;
            next = if anchor == focus {
                apply_text_edit_action(
                    next,
                    UiTextEditAction::MoveCaret {
                        offset: focus,
                        extend_selection: false,
                    },
                )
            } else {
                apply_text_edit_action(next, UiTextEditAction::SetSelection { anchor, focus })
            };
        }
    }

    TextInputStateTransition {
        state: next,
        constraint_receipt,
        committed_edit: None,
    }
}

pub(super) fn committed_text_replaced_range(editable: &UiEditableTextState) -> UiTextRange {
    editable
        .composition
        .as_ref()
        .map(|composition| composition.range)
        .or_else(|| editable.selection.as_ref().map(UiTextSelection::range))
        .unwrap_or(UiTextRange {
            start: editable.caret.offset,
            end: editable.caret.offset,
        })
}

pub(super) fn preedit_text_replaced_range(editable: &UiEditableTextState) -> UiTextRange {
    editable
        .composition
        .as_ref()
        .map(|composition| composition.range)
        .or_else(|| editable.selection.as_ref().map(UiTextSelection::range))
        .unwrap_or(UiTextRange {
            start: editable.caret.offset,
            end: editable.caret.offset,
        })
}

pub(super) fn retained_document_replaced_range(editable: &UiEditableTextState) -> UiTextRange {
    let Some(composition) = editable.composition.as_ref() else {
        return committed_text_replaced_range(editable);
    };
    let end = composition
        .restore_text
        .as_ref()
        .and_then(|restore_text| composition.range.start.checked_add(restore_text.len()))
        .unwrap_or(composition.range.end);
    UiTextRange {
        start: composition.range.start,
        end,
    }
}

pub(super) fn delete_surrounding_text_state(
    editable: UiEditableTextState,
    delete: UiImeDeleteSurrounding,
) -> Option<TextInputStateTransition> {
    if editable.read_only || (delete.before_bytes == 0 && delete.after_bytes == 0) {
        return None;
    }
    let (committed_text, caret) = committed_text_and_caret_for_surrounding_delete(&editable);
    let range = surrounding_delete_range(committed_text.as_ref(), caret, delete);
    if range.start == range.end {
        return None;
    }
    let committed = restore_committed_state_for_surrounding_delete(editable, caret);
    let transition = apply_text_edit_actions_with_intent(
        committed,
        [
            UiTextEditAction::SetSelection {
                anchor: range.start,
                focus: range.end,
            },
            UiTextEditAction::Delete,
        ],
    )
    .ok()?;
    Some(TextInputStateTransition::from_edit(transition))
}

fn committed_text_and_caret_for_surrounding_delete(
    editable: &UiEditableTextState,
) -> (Cow<'_, str>, usize) {
    let caret = clamp_grapheme_boundary(&editable.text, editable.caret.offset);
    let Some(composition) = editable.composition.as_ref() else {
        return (Cow::Borrowed(editable.text.as_str()), caret);
    };
    let start = clamp_grapheme_boundary(&editable.text, composition.range.start);
    let end = clamp_grapheme_boundary(&editable.text, composition.range.end).max(start);
    let composition_range = UiTextRange { start, end };
    let restore_text = composition.restore_text.as_deref().unwrap_or_default();
    let committed_caret =
        map_visible_offset_to_committed_offset(caret, composition_range, restore_text.len());
    if composition.restore_text.is_none() {
        return (Cow::Borrowed(editable.text.as_str()), committed_caret);
    }
    let mut committed =
        String::with_capacity(editable.text.len() - (end - start) + restore_text.len());
    committed.push_str(&editable.text[..start]);
    committed.push_str(restore_text);
    committed.push_str(&editable.text[end..]);
    (Cow::Owned(committed), committed_caret)
}

fn restore_committed_state_for_surrounding_delete(
    editable: UiEditableTextState,
    committed_caret: usize,
) -> UiEditableTextState {
    let committed = apply_text_edit_action(editable, UiTextEditAction::CancelComposition);
    let committed_caret = clamp_grapheme_boundary(&committed.text, committed_caret);
    apply_text_edit_action(
        committed,
        UiTextEditAction::MoveCaret {
            offset: committed_caret,
            extend_selection: false,
        },
    )
}

pub(super) fn map_visible_offset_to_committed_offset(
    caret: usize,
    visible_range: UiTextRange,
    replacement_len: usize,
) -> usize {
    if visible_range.start == visible_range.end {
        return caret;
    }
    if caret <= visible_range.start {
        caret
    } else if caret >= visible_range.end {
        visible_range.start + replacement_len + (caret - visible_range.end)
    } else {
        visible_range.start + replacement_len
    }
}

fn surrounding_delete_range(
    text: &str,
    caret: usize,
    delete: UiImeDeleteSurrounding,
) -> UiTextRange {
    let caret = clamp_grapheme_boundary(text, caret);
    let start = caret.saturating_sub(delete.before_bytes as usize);
    let end = caret
        .saturating_add(delete.after_bytes as usize)
        .min(text.len());
    UiTextRange {
        start: grapheme_start_at_or_before(text, start),
        end: grapheme_end_at_or_after(text, end),
    }
}

fn grapheme_start_at_or_before(text: &str, offset: usize) -> usize {
    let offset = clamp_grapheme_boundary(text, offset);
    let Some(previous) = previous_grapheme_boundary(text, offset) else {
        return 0;
    };
    if next_grapheme_boundary(text, previous) == Some(offset) {
        offset
    } else {
        previous
    }
}

fn grapheme_end_at_or_after(text: &str, offset: usize) -> usize {
    let offset = ceil_text_boundary(text, offset);
    let Some(previous) = previous_grapheme_boundary(text, offset) else {
        return offset;
    };
    if next_grapheme_boundary(text, previous) == Some(offset) {
        offset
    } else {
        next_grapheme_boundary(text, offset).unwrap_or(text.len())
    }
}

fn ceil_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset < text.len() && !text.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::{delete_surrounding_text_state, retained_document_replaced_range};
    use zircon_runtime_interface::ui::{
        dispatch::UiImeDeleteSurrounding,
        surface::{UiEditableTextState, UiTextCaret, UiTextComposition, UiTextRange},
    };

    #[test]
    fn delete_surrounding_uses_the_visible_paint_only_composition_text() {
        let state = UiEditableTextState {
            text: "aXb".to_owned(),
            caret: UiTextCaret {
                offset: 2,
                ..Default::default()
            },
            composition: Some(UiTextComposition {
                range: UiTextRange { start: 1, end: 2 },
                preedit_clauses: Vec::new(),
                text: "X".to_owned(),
                restore_text: None,
            }),
            ..Default::default()
        };

        let next = delete_surrounding_text_state(state, UiImeDeleteSurrounding::new(1, 0))
            .expect("visible text has a byte before the mapped caret");

        assert_eq!(next.state.text, "Xb");
        assert!(next.state.composition.is_none());
        let intent = next.committed_edit.expect("surrounding delete intent");
        assert_eq!(intent.old, 0..1);
        assert_eq!(intent.new, 0..0);
    }

    #[test]
    fn retained_document_range_uses_composition_restore_length_not_visible_preedit_length() {
        let state = UiEditableTextState {
            text: "aXYf".to_owned(),
            composition: Some(UiTextComposition {
                range: UiTextRange { start: 1, end: 3 },
                preedit_clauses: Vec::new(),
                text: "XY".to_owned(),
                restore_text: Some("bcde".to_owned()),
            }),
            ..Default::default()
        };

        assert_eq!(
            retained_document_replaced_range(&state),
            UiTextRange { start: 1, end: 5 }
        );
    }
}
