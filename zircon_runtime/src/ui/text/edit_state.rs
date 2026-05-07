use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextEditAction, UiTextRange,
    UiTextSelection,
};

pub(crate) fn apply_text_edit_action(
    mut state: UiEditableTextState,
    action: UiTextEditAction,
) -> UiEditableTextState {
    if state.read_only {
        return state;
    }

    match action {
        UiTextEditAction::Insert { text } => replace_selection_or_range(&mut state, &text),
        UiTextEditAction::Backspace => backspace(&mut state),
        UiTextEditAction::Delete => delete(&mut state),
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
        UiTextEditAction::SetComposition { range, text } => {
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
            let range = composition_source_range(&state.text, range, &text);
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
        UiTextEditAction::CommitComposition => {
            if let Some(composition) = state.composition.take() {
                state.caret.offset = clamp_boundary(&state.text, composition.range.end);
                state.caret.affinity = UiTextCaretAffinity::Downstream;
                state.selection = None;
            }
        }
        UiTextEditAction::CancelComposition => {
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

fn composition_source_range(text: &str, range: UiTextRange, replacement: &str) -> UiTextRange {
    let start = clamp_boundary(text, range.start);
    let explicit_end = clamp_boundary(text, range.end).max(start);
    let end = if explicit_end == start {
        start
    } else {
        clamp_boundary(text, start + replacement.len()).max(explicit_end)
    };
    UiTextRange { start, end }
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
    text.char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index < offset)
        .last()
}

fn next_boundary(text: &str, offset: usize) -> Option<usize> {
    text.char_indices()
        .map(|(index, _)| index)
        .find(|index| *index > offset)
        .or_else(|| (offset < text.len()).then_some(text.len()))
}
