use std::ops::Range;

use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextEditAction, UiTextRange,
    UiTextSelection,
};
use zircon_runtime_interface::ui::text::UiTextEditKind;

use super::{clamp_grapheme_boundary, next_grapheme_boundary, previous_grapheme_boundary};

pub(crate) fn apply_text_edit_action(
    state: UiEditableTextState,
    action: UiTextEditAction,
) -> UiEditableTextState {
    apply_text_edit_action_with_intent(state, action).state
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommittedTextEditIntent {
    pub(crate) old: Range<usize>,
    pub(crate) new: Range<usize>,
    pub(crate) kind: UiTextEditKind,
}

impl CommittedTextEditIntent {
    pub(crate) fn for_replacement(old: Range<usize>, replacement_len: usize) -> Self {
        let new = old.start..old.start.saturating_add(replacement_len);
        let kind = if old.is_empty() && !new.is_empty() {
            UiTextEditKind::Insert
        } else if !old.is_empty() && new.is_empty() {
            UiTextEditKind::Delete
        } else {
            UiTextEditKind::Replace
        };
        Self { old, new, kind }
    }

    pub(crate) fn replacement<'a>(&self, state: &'a UiEditableTextState) -> Option<&'a str> {
        state.text.get(self.new.clone())
    }

    pub(crate) fn is_valid_for_state(&self, state: &UiEditableTextState) -> bool {
        let Some(old_len) = self.old.end.checked_sub(self.old.start) else {
            return false;
        };
        let Some(new_len) = self.new.end.checked_sub(self.new.start) else {
            return false;
        };
        if self.new.start != self.old.start || (old_len == 0 && new_len == 0) {
            return false;
        }
        if self.new.end > state.text.len()
            || clamp_grapheme_boundary(&state.text, self.new.start) != self.new.start
            || clamp_grapheme_boundary(&state.text, self.new.end) != self.new.end
        {
            return false;
        }
        let Some(previous_len) = state
            .text
            .len()
            .checked_sub(new_len)
            .and_then(|len| len.checked_add(old_len))
        else {
            return false;
        };
        if self.old.end > previous_len || self.replacement(state).is_none() {
            return false;
        }
        match self.kind {
            UiTextEditKind::Insert => old_len == 0 && new_len != 0,
            UiTextEditKind::Delete => old_len != 0 && new_len == 0,
            UiTextEditKind::Replace => old_len != 0 && new_len != 0,
            UiTextEditKind::CompositionCommit | UiTextEditKind::Undo | UiTextEditKind::Redo => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextEditStateTransition {
    pub(crate) state: UiEditableTextState,
    pub(crate) committed: Option<CommittedTextEditIntent>,
}

impl TextEditStateTransition {
    pub(crate) fn state_only(state: UiEditableTextState) -> Self {
        Self {
            state,
            committed: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextEditActionSequenceError {
    MultipleCommittedEdits,
}

pub(crate) fn apply_text_edit_actions_with_intent(
    mut state: UiEditableTextState,
    actions: impl IntoIterator<Item = UiTextEditAction>,
) -> Result<TextEditStateTransition, TextEditActionSequenceError> {
    let mut committed = None;
    for action in actions {
        let transition = apply_text_edit_action_with_intent(state, action);
        if transition.committed.is_some() && committed.is_some() {
            return Err(TextEditActionSequenceError::MultipleCommittedEdits);
        }
        state = transition.state;
        if transition.committed.is_some() {
            committed = transition.committed;
        }
    }
    Ok(TextEditStateTransition { state, committed })
}

pub(crate) fn apply_text_edit_action_with_intent(
    mut state: UiEditableTextState,
    action: UiTextEditAction,
) -> TextEditStateTransition {
    let committed = match action {
        UiTextEditAction::Insert { text } if !state.read_only => {
            replace_selection_or_range(&mut state, &text)
        }
        UiTextEditAction::Backspace if !state.read_only => backspace(&mut state),
        UiTextEditAction::Delete if !state.read_only => delete(&mut state),
        UiTextEditAction::MoveCaret {
            offset,
            extend_selection,
        } => {
            move_caret(&mut state, offset, extend_selection);
            None
        }
        UiTextEditAction::SetSelection { anchor, focus } => {
            let anchor = clamp_grapheme_boundary(&state.text, anchor);
            let focus = clamp_grapheme_boundary(&state.text, focus);
            state.caret = UiTextCaret {
                offset: focus,
                affinity: UiTextCaretAffinity::Downstream,
            };
            state.selection = Some(UiTextSelection { anchor, focus });
            None
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
                preedit_clauses: Vec::new(),
                text,
                restore_text: Some(restore_text),
            });
            None
        }
        UiTextEditAction::CommitComposition if !state.read_only => {
            let committed = state
                .composition
                .as_ref()
                .and_then(composition_commit_intent);
            if let Some(composition) = state.composition.take() {
                state.caret.offset = clamp_grapheme_boundary(&state.text, composition.range.end);
                state.caret.affinity = UiTextCaretAffinity::Downstream;
                state.selection = None;
            }
            committed
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
            None
        }
        UiTextEditAction::Insert { .. }
        | UiTextEditAction::Backspace
        | UiTextEditAction::Delete
        | UiTextEditAction::SetComposition { .. }
        | UiTextEditAction::CommitComposition => None,
    };

    TextEditStateTransition { state, committed }
}

fn replace_selection_or_range(
    state: &mut UiEditableTextState,
    text: &str,
) -> Option<CommittedTextEditIntent> {
    let tracks_committed_source = state.composition.is_none();
    if let Some(selection) = state.selection.take() {
        let range = selection.range();
        let start = clamp_grapheme_boundary(&state.text, range.start);
        let end = clamp_grapheme_boundary(&state.text, range.end).max(start);
        let unchanged = state.text.get(start..end) == Some(text);
        replace_range(state, start, end, text);
        (tracks_committed_source && !unchanged)
            .then(|| committed_edit_intent(start..end, text.len()))
    } else {
        let offset = clamp_grapheme_boundary(&state.text, state.caret.offset);
        replace_range(state, offset, offset, text);
        (tracks_committed_source && !text.is_empty())
            .then(|| committed_edit_intent(offset..offset, text.len()))
    }
}

fn backspace(state: &mut UiEditableTextState) -> Option<CommittedTextEditIntent> {
    let tracks_committed_source = state.composition.is_none();
    if state
        .selection
        .as_ref()
        .is_some_and(|selection| selection.anchor != selection.focus)
    {
        return replace_selection_or_range(state, "");
    }
    let caret = clamp_grapheme_boundary(&state.text, state.caret.offset);
    let Some(previous) = previous_boundary(&state.text, caret) else {
        return None;
    };
    replace_range(state, previous, caret, "");
    tracks_committed_source.then(|| committed_edit_intent(previous..caret, 0))
}

fn delete(state: &mut UiEditableTextState) -> Option<CommittedTextEditIntent> {
    let tracks_committed_source = state.composition.is_none();
    if state
        .selection
        .as_ref()
        .is_some_and(|selection| selection.anchor != selection.focus)
    {
        return replace_selection_or_range(state, "");
    }
    let caret = clamp_grapheme_boundary(&state.text, state.caret.offset);
    let Some(next) = next_boundary(&state.text, caret) else {
        return None;
    };
    replace_range(state, caret, next, "");
    tracks_committed_source.then(|| committed_edit_intent(caret..next, 0))
}

fn move_caret(state: &mut UiEditableTextState, offset: usize, extend_selection: bool) {
    let offset = clamp_grapheme_boundary(&state.text, offset);
    if extend_selection {
        let anchor = state
            .selection
            .as_ref()
            .map(|selection| selection.anchor)
            .unwrap_or(state.caret.offset);
        state.selection = Some(UiTextSelection {
            anchor: clamp_grapheme_boundary(&state.text, anchor),
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
    replace_aligned_range_preserving_composition(state, start, end, replacement);
    state.composition = None;
}

fn replace_range_preserving_composition(
    state: &mut UiEditableTextState,
    start: usize,
    end: usize,
    replacement: &str,
) {
    let start = clamp_grapheme_boundary(&state.text, start);
    let end = clamp_grapheme_boundary(&state.text, end).max(start);
    replace_aligned_range_preserving_composition(state, start, end, replacement);
}

fn replace_aligned_range_preserving_composition(
    state: &mut UiEditableTextState,
    start: usize,
    end: usize,
    replacement: &str,
) {
    debug_assert!(start <= end && state.text.is_char_boundary(start));
    debug_assert!(state.text.is_char_boundary(end));
    state.text.replace_range(start..end, replacement);
    state.caret.offset = start + replacement.len();
    state.caret.affinity = UiTextCaretAffinity::Downstream;
    state.selection = None;
}

fn composition_source_range(text: &str, range: UiTextRange) -> UiTextRange {
    let start = clamp_grapheme_boundary(text, range.start);
    UiTextRange {
        start,
        end: clamp_grapheme_boundary(text, range.end).max(start),
    }
}

fn committed_edit_intent(old: Range<usize>, replacement_len: usize) -> CommittedTextEditIntent {
    CommittedTextEditIntent::for_replacement(old, replacement_len)
}

fn composition_commit_intent(
    composition: &zircon_runtime_interface::ui::surface::UiTextComposition,
) -> Option<CommittedTextEditIntent> {
    let restore_text = composition.restore_text.as_ref()?;
    if restore_text == &composition.text {
        return None;
    }
    let restore_len = restore_text.len();
    let old_end = composition.range.start.checked_add(restore_len)?;
    let new_end = composition
        .range
        .start
        .checked_add(composition.text.len())?;
    Some(CommittedTextEditIntent {
        old: composition.range.start..old_end,
        new: composition.range.start..new_end,
        kind: UiTextEditKind::CompositionCommit,
    })
}

fn previous_boundary(text: &str, offset: usize) -> Option<usize> {
    previous_grapheme_boundary(text, offset)
}

fn next_boundary(text: &str, offset: usize) -> Option<usize> {
    next_grapheme_boundary(text, offset)
}

#[cfg(test)]
#[path = "tests/edit_state.rs"]
mod tests;
