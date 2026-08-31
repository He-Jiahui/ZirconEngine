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
    replace_range_preserving_composition(state, start, end, replacement);
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

    #[test]
    fn external_selection_and_composition_offsets_do_not_split_a_grapheme() {
        let selected = apply_text_edit_action(
            editable_text_state("a\u{0301}b", 0, None),
            UiTextEditAction::SetSelection {
                anchor: 1,
                focus: 2,
            },
        );
        assert_eq!(
            selected.selection,
            Some(UiTextSelection {
                anchor: 0,
                focus: 0,
            })
        );
        assert_eq!(selected.caret.offset, 0);

        let composed = apply_text_edit_action(
            editable_text_state("a\u{0301}b", 0, None),
            UiTextEditAction::SetComposition {
                range: UiTextRange { start: 1, end: 2 },
                text: "x".to_string(),
            },
        );
        assert_eq!(composed.text, "xa\u{0301}b");
        assert_eq!(
            composed
                .composition
                .as_ref()
                .map(|composition| composition.range),
            Some(UiTextRange { start: 0, end: 1 })
        );
    }

    #[test]
    fn committed_insert_reports_exact_old_new_ranges_without_another_text_copy() {
        let selected = apply_text_edit_action(
            editable_text_state("alpha", 5, None),
            UiTextEditAction::SetSelection {
                anchor: 1,
                focus: 4,
            },
        );

        let transition = apply_text_edit_action_with_intent(
            selected,
            UiTextEditAction::Insert {
                text: "XYZ".to_string(),
            },
        );

        let intent = transition.committed.as_ref().expect("committed intent");
        assert_eq!(transition.state.text, "aXYZa");
        assert_eq!(intent.old, 1..4);
        assert_eq!(intent.new, 1..4);
        assert_eq!(intent.kind, UiTextEditKind::Replace);
        assert_eq!(intent.replacement(&transition.state), Some("XYZ"));
    }

    #[test]
    fn grapheme_delete_and_caret_move_distinguish_committed_and_state_only_changes() {
        let deleted = apply_text_edit_action_with_intent(
            editable_text_state("a\u{0301}b", 3, None),
            UiTextEditAction::Backspace,
        );
        let intent = deleted.committed.as_ref().expect("delete intent");
        assert_eq!(deleted.state.text, "b");
        assert_eq!(intent.old, 0..3);
        assert_eq!(intent.new, 0..0);
        assert_eq!(intent.kind, UiTextEditKind::Delete);

        let moved = apply_text_edit_action_with_intent(
            deleted.state,
            UiTextEditAction::MoveCaret {
                offset: 1,
                extend_selection: false,
            },
        );
        assert!(moved.committed.is_none());
    }

    #[test]
    fn composition_preedit_is_transient_and_commit_reports_the_original_source_range() {
        let preedit = apply_text_edit_action_with_intent(
            editable_text_state("abcdef", 5, Some(UiTextRange { start: 1, end: 5 })),
            UiTextEditAction::SetComposition {
                range: UiTextRange { start: 1, end: 5 },
                text: "XY".to_string(),
            },
        );
        assert!(preedit.committed.is_none());

        let committed =
            apply_text_edit_action_with_intent(preedit.state, UiTextEditAction::CommitComposition);
        let intent = committed.committed.as_ref().expect("composition commit");
        assert_eq!(committed.state.text, "aXYf");
        assert_eq!(intent.old, 1..5);
        assert_eq!(intent.new, 1..3);
        assert_eq!(intent.kind, UiTextEditKind::CompositionCommit);
        assert_eq!(intent.replacement(&committed.state), Some("XY"));
    }

    #[test]
    fn state_only_selection_followed_by_delete_produces_one_committed_intent() {
        let transition = apply_text_edit_actions_with_intent(
            editable_text_state("alpha beta", 10, None),
            [
                UiTextEditAction::SetSelection {
                    anchor: 6,
                    focus: 10,
                },
                UiTextEditAction::Delete,
            ],
        )
        .expect("selection plus delete contains one committed edit");

        let intent = transition.committed.as_ref().expect("delete intent");
        assert_eq!(transition.state.text, "alpha ");
        assert_eq!(intent.old, 6..10);
        assert_eq!(intent.new, 6..6);
        assert_eq!(intent.kind, UiTextEditKind::Delete);
    }

    #[test]
    fn multiple_committed_actions_are_rejected_instead_of_losing_an_intent() {
        let error = apply_text_edit_actions_with_intent(
            editable_text_state("ab", 2, None),
            [UiTextEditAction::Backspace, UiTextEditAction::Backspace],
        )
        .expect_err("two committed edits cannot be represented by one intent");

        assert_eq!(error, TextEditActionSequenceError::MultipleCommittedEdits);
    }

    #[test]
    fn unchanged_replacements_do_not_create_document_edit_intents() {
        let empty_insert = apply_text_edit_action_with_intent(
            editable_text_state("alpha", 2, None),
            UiTextEditAction::Insert {
                text: String::new(),
            },
        );
        assert!(empty_insert.committed.is_none());

        let selected = apply_text_edit_action(
            editable_text_state("alpha", 4, None),
            UiTextEditAction::SetSelection {
                anchor: 1,
                focus: 4,
            },
        );
        let identical = apply_text_edit_action_with_intent(
            selected,
            UiTextEditAction::Insert {
                text: "lph".to_string(),
            },
        );
        assert_eq!(identical.state.text, "alpha");
        assert!(identical.committed.is_none());
    }

    #[test]
    fn unchanged_composition_commit_is_state_only() {
        let preedit = apply_text_edit_action_with_intent(
            editable_text_state("alpha", 4, Some(UiTextRange { start: 1, end: 4 })),
            UiTextEditAction::SetComposition {
                range: UiTextRange { start: 1, end: 4 },
                text: "lph".to_string(),
            },
        );
        let committed =
            apply_text_edit_action_with_intent(preedit.state, UiTextEditAction::CommitComposition);

        assert_eq!(committed.state.text, "alpha");
        assert!(committed.state.composition.is_none());
        assert!(committed.committed.is_none());
    }

    #[test]
    fn committed_intent_validation_rejects_malformed_or_state_only_ranges() {
        let state = editable_text_state("a\u{0301}b", 3, None);
        let cases = [
            CommittedTextEditIntent {
                old: 3..2,
                new: 3..3,
                kind: UiTextEditKind::Delete,
            },
            CommittedTextEditIntent {
                old: 0..0,
                new: 3..4,
                kind: UiTextEditKind::Insert,
            },
            CommittedTextEditIntent {
                old: 1..1,
                new: 1..1,
                kind: UiTextEditKind::Replace,
            },
            CommittedTextEditIntent {
                old: 3..3,
                new: 3..3,
                kind: UiTextEditKind::Insert,
            },
        ];

        for intent in cases {
            assert!(!intent.is_valid_for_state(&state));
        }
    }

    #[test]
    fn committed_intent_validation_accepts_exact_grapheme_bounded_replacement() {
        let state = editable_text_state("a\u{0301}XYb", 5, None);
        let intent = CommittedTextEditIntent {
            old: 3..4,
            new: 3..5,
            kind: UiTextEditKind::Replace,
        };

        assert!(intent.is_valid_for_state(&state));
        assert_eq!(intent.replacement(&state), Some("XY"));
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
