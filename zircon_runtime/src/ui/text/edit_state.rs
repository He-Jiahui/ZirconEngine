use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextEditAction, UiTextRange,
    UiTextSelection,
};

use super::{next_grapheme_boundary, previous_grapheme_boundary};

pub(crate) fn apply_text_edit_action(
    mut state: UiEditableTextState,
    action: UiTextEditAction,
) -> UiEditableTextState {
    match action {
        UiTextEditAction::Insert { text } if !state.read_only => {
            replace_selection_or_range(&mut state, &text)
        }
        UiTextEditAction::Backspace if !state.read_only => backspace(&mut state),
        UiTextEditAction::Delete if !state.read_only => delete(&mut state),
        UiTextEditAction::MoveCaret {
            offset,
            extend_selection,
        } => move_caret(&mut state, offset, extend_selection),
        UiTextEditAction::SetSelection { anchor, focus } => {
            let anchor = clamp_boundary(&state.text, anchor);
            let focus = clamp_boundary(&state.text, focus);
            state.caret.offset = focus;
            state.selection = Some(UiTextSelection { anchor, focus });
        }
        UiTextEditAction::SetComposition { range, text } if !state.read_only => {
            let mut source_range = range;
            if let Some(composition) = state.composition.take() {
                if let Some(restore_text) = composition.restore_text {
                    let restored_source_range = UiTextRange {
                        start: composition.range.start,
                        end: composition.range.start + restore_text.len(),
                    };
                    replace_range_preserving_composition(
                        &mut state,
                        composition.range.start,
                        composition.range.end,
                        &restore_text,
                    );
                    if source_range == composition.range {
                        source_range = restored_source_range;
                    }
                }
            }
            let range = composition_source_range(&state.text, source_range);
            let restore_text = state.text[range.start..range.end].to_string();
            replace_range_preserving_composition(&mut state, range.start, range.end, &text);
            state.composition = Some(zircon_runtime_interface::ui::surface::UiTextComposition {
                range: UiTextRange {
                    start: range.start,
                    end: range.start + text.len(),
                },
                text,
                restore_text: Some(restore_text),
            });
        }
        UiTextEditAction::CommitComposition if !state.read_only => {
            if let Some(composition) = state.composition.take() {
                state.caret.offset = clamp_boundary(&state.text, composition.range.end);
                state.caret.affinity = UiTextCaretAffinity::Downstream;
                state.selection = None;
            }
        }
        UiTextEditAction::CancelComposition if !state.read_only => {
            if let Some(composition) = state.composition.take() {
                if let Some(restore_text) = composition.restore_text {
                    replace_range_preserving_composition(
                        &mut state,
                        composition.range.start,
                        composition.range.end,
                        &restore_text,
                    );
                }
            }
        }
        UiTextEditAction::Insert { .. }
        | UiTextEditAction::Backspace
        | UiTextEditAction::Delete
        | UiTextEditAction::SetComposition { .. }
        | UiTextEditAction::CommitComposition
        | UiTextEditAction::CancelComposition => {}
    }

    state
}

fn replace_selection_or_range(state: &mut UiEditableTextState, text: &str) {
    if let Some(selection) = state.selection.take() {
        let range = selection.range();
        replace_range(state, range.start, range.end, text);
    } else {
        let offset = clamp_boundary(&state.text, state.caret.offset);
        replace_range(state, offset, offset, text);
    }
}

fn backspace(state: &mut UiEditableTextState) {
    if state
        .selection
        .as_ref()
        .is_some_and(|selection| selection.anchor != selection.focus)
    {
        replace_selection_or_range(state, "");
        return;
    }
    let caret = clamp_boundary(&state.text, state.caret.offset);
    let Some(previous) = previous_boundary(&state.text, caret) else {
        return;
    };
    replace_range(state, previous, caret, "");
}

fn delete(state: &mut UiEditableTextState) {
    if state
        .selection
        .as_ref()
        .is_some_and(|selection| selection.anchor != selection.focus)
    {
        replace_selection_or_range(state, "");
        return;
    }
    let caret = clamp_boundary(&state.text, state.caret.offset);
    let Some(next) = next_boundary(&state.text, caret) else {
        return;
    };
    replace_range(state, caret, next, "");
}

fn move_caret(state: &mut UiEditableTextState, offset: usize, extend_selection: bool) {
    let offset = clamp_boundary(&state.text, offset);
    if extend_selection {
        let anchor = state
            .selection
            .as_ref()
            .map(|selection| selection.anchor)
            .unwrap_or(state.caret.offset);
        state.selection = Some(UiTextSelection {
            anchor: clamp_boundary(&state.text, anchor),
            focus: offset,
        });
    } else {
        state.selection = None;
    }
    state.caret = UiTextCaret {
        offset,
        affinity: UiTextCaretAffinity::Downstream,
    };
}

fn replace_range(state: &mut UiEditableTextState, start: usize, end: usize, replacement: &str) {
    replace_range_preserving_composition(state, start, end, replacement);
    state.composition = None;
}

fn replace_range_preserving_composition(
    state: &mut UiEditableTextState,
    start: usize,
    end: usize,
    replacement: &str,
) {
    let start = clamp_boundary(&state.text, start);
    let end = clamp_boundary(&state.text, end).max(start);
    state.text.replace_range(start..end, replacement);
    state.caret.offset = start + replacement.len();
    state.caret.affinity = UiTextCaretAffinity::Downstream;
    state.selection = None;
}

fn composition_source_range(text: &str, range: UiTextRange) -> UiTextRange {
    let start = clamp_boundary(text, range.start);
    UiTextRange {
        start,
        end: clamp_boundary(text, range.end).max(start),
    }
}

fn clamp_boundary(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    if text.is_char_boundary(offset) {
        return offset;
    }
    text.char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= offset)
        .last()
        .unwrap_or(0)
}

fn previous_boundary(text: &str, offset: usize) -> Option<usize> {
    previous_grapheme_boundary(text, offset)
}

fn next_boundary(text: &str, offset: usize) -> Option<usize> {
    next_grapheme_boundary(text, offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_composition_replaces_explicit_range_without_extending_to_preedit_len() {
        let state = editable_text_state("abcdef", 5, Some(UiTextRange { start: 1, end: 5 }));

        let next = apply_text_edit_action(
            state,
            UiTextEditAction::SetComposition {
                range: UiTextRange { start: 1, end: 5 },
                text: "WXYZQ".to_string(),
            },
        );

        let composition = next.composition.as_ref().expect("composition");
        assert_eq!(next.text, "aWXYZQf");
        assert_eq!(next.caret.offset, 6);
        assert_eq!(composition.range, UiTextRange { start: 1, end: 6 });
        assert_eq!(composition.restore_text.as_deref(), Some("bcde"));
    }

    #[test]
    fn set_composition_update_reuses_restored_source_range() {
        let state = editable_text_state("abcdef", 5, Some(UiTextRange { start: 1, end: 5 }));
        let first = apply_text_edit_action(
            state,
            UiTextEditAction::SetComposition {
                range: UiTextRange { start: 1, end: 5 },
                text: "WXYZQ".to_string(),
            },
        );
        let visible_composition_range = first.composition.as_ref().unwrap().range;

        let next = apply_text_edit_action(
            first,
            UiTextEditAction::SetComposition {
                range: visible_composition_range,
                text: "UV".to_string(),
            },
        );

        let composition = next.composition.as_ref().expect("composition");
        assert_eq!(next.text, "aUVf");
        assert_eq!(next.caret.offset, 3);
        assert_eq!(composition.range, UiTextRange { start: 1, end: 3 });
        assert_eq!(composition.restore_text.as_deref(), Some("bcde"));
    }

    fn editable_text_state(
        text: &str,
        caret_offset: usize,
        selection_range: Option<UiTextRange>,
    ) -> UiEditableTextState {
        UiEditableTextState {
            text: text.to_string(),
            caret: UiTextCaret {
                offset: caret_offset,
                affinity: UiTextCaretAffinity::Downstream,
            },
            selection: selection_range.map(|range| UiTextSelection {
                anchor: range.start,
                focus: range.end,
            }),
            composition: None,
            read_only: false,
        }
    }
}
