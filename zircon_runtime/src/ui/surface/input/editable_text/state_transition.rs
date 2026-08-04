use std::borrow::Cow;

use zircon_runtime_interface::ui::{
    dispatch::{UiImeDeleteSurrounding, UiImePreeditClause, UiTextByteRange},
    surface::{UiEditableTextState, UiTextEditAction, UiTextRange, UiTextSelection},
};

use crate::ui::text::{apply_text_edit_action, next_grapheme_boundary, previous_grapheme_boundary};

use super::super::{text_constraints::TextInputConstraints, text_state::clamp_text_boundary};

pub(super) fn committed_text_state(
    editable: UiEditableTextState,
    text: String,
    constraints: TextInputConstraints,
) -> UiEditableTextState {
    if editable.composition.is_some() {
        let range = editable
            .composition
            .as_ref()
            .map(|composition| composition.range)
            .unwrap_or(UiTextRange {
                start: editable.caret.offset,
                end: editable.caret.offset,
            });
        let text = constraints.sanitize_replacement(&editable.text, range, &text);
        let composed =
            apply_text_edit_action(editable, UiTextEditAction::SetComposition { range, text });
        apply_text_edit_action(composed, UiTextEditAction::CommitComposition)
    } else {
        let range = editable
            .selection
            .as_ref()
            .map(UiTextSelection::range)
            .unwrap_or(UiTextRange {
                start: editable.caret.offset,
                end: editable.caret.offset,
            });
        let text = constraints.sanitize_replacement(&editable.text, range, &text);
        apply_text_edit_action(editable, UiTextEditAction::Insert { text })
    }
}

pub(super) fn preedit_text_state(
    editable: UiEditableTextState,
    preedit: &str,
    cursor_range: Option<UiTextByteRange>,
    preedit_clauses: &[UiImePreeditClause],
    constraints: TextInputConstraints,
) -> UiEditableTextState {
    let range = editable
        .composition
        .as_ref()
        .map(|composition| composition.range)
        .or_else(|| editable.selection.as_ref().map(UiTextSelection::range))
        .unwrap_or(UiTextRange {
            start: editable.caret.offset,
            end: editable.caret.offset,
        });
    let sanitized_preedit = constraints.sanitize_replacement(&editable.text, range, preedit);
    let preedit_clauses = (sanitized_preedit == preedit)
        .then(|| preedit_clauses.to_vec())
        .unwrap_or_default();
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
            let anchor = composition.range.start
                + clamp_text_boundary(&composition.text, cursor_range.start_byte as usize);
            let focus = composition.range.start
                + clamp_text_boundary(&composition.text, cursor_range.end_byte as usize);
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

    next
}

pub(super) fn delete_surrounding_text_state(
    editable: UiEditableTextState,
    delete: UiImeDeleteSurrounding,
) -> Option<UiEditableTextState> {
    if editable.read_only || (delete.before_bytes == 0 && delete.after_bytes == 0) {
        return None;
    }
    let (committed_text, caret) = committed_text_and_caret_for_surrounding_delete(&editable);
    let range = surrounding_delete_range(committed_text.as_ref(), caret, delete);
    if range.start == range.end {
        return None;
    }
    let committed = restore_committed_state_for_surrounding_delete(editable, caret);
    let selected = apply_text_edit_action(
        committed,
        UiTextEditAction::SetSelection {
            anchor: range.start,
            focus: range.end,
        },
    );
    Some(apply_text_edit_action(selected, UiTextEditAction::Delete))
}

fn committed_text_and_caret_for_surrounding_delete(
    editable: &UiEditableTextState,
) -> (Cow<'_, str>, usize) {
    let caret = clamp_text_boundary(&editable.text, editable.caret.offset);
    let Some(composition) = editable.composition.as_ref() else {
        return (Cow::Borrowed(editable.text.as_str()), caret);
    };
    let start = clamp_text_boundary(&editable.text, composition.range.start);
    let end = clamp_text_boundary(&editable.text, composition.range.end).max(start);
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
    let committed_caret = clamp_text_boundary(&committed.text, committed_caret);
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
    let caret = clamp_text_boundary(text, caret);
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
    let offset = clamp_text_boundary(text, offset);
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
    use super::delete_surrounding_text_state;
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

        assert_eq!(next.text, "Xb");
        assert!(next.composition.is_none());
    }
}
