use std::{collections::VecDeque, ops::Range};

use zircon_runtime_interface::ui::{
    surface::{UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextSelection},
    text::UiTextEditKind,
};

use crate::ui::text::{CommittedTextEditIntent, TextEditStateTransition};

pub(super) const MVP_TEXT_HISTORY_MAX_ENTRIES: usize = 100;
pub(super) const MVP_TEXT_HISTORY_MAX_DELTA_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) enum UiTextHistoryDirection {
    Undo,
    Redo,
}

pub(in crate::ui) enum UiTextHistoryCommit {
    Record(UiTextHistoryEntry),
    Barrier,
    Undo,
    Redo,
}

pub(in crate::ui) struct UiTextHistoryEntry {
    old: Range<usize>,
    new: Range<usize>,
    removed: String,
    inserted: String,
    before: UiTextInteractionState,
    after: UiTextInteractionState,
}

impl UiTextHistoryEntry {
    pub(super) fn new(
        intent: &CommittedTextEditIntent,
        removed: String,
        inserted: String,
        current: &UiEditableTextState,
        next: &UiEditableTextState,
    ) -> Self {
        let before = if current.composition.is_some() {
            UiTextInteractionState::for_composition_source(&intent.old)
        } else {
            UiTextInteractionState::from_state(current)
        };
        Self {
            old: intent.old.clone(),
            new: intent.new.clone(),
            removed,
            inserted,
            before,
            after: UiTextInteractionState::from_state(next),
        }
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.removed.len().saturating_add(self.inserted.len())
    }

    pub(super) fn expected_range(&self, direction: UiTextHistoryDirection) -> Range<usize> {
        match direction {
            UiTextHistoryDirection::Undo => self.new.clone(),
            UiTextHistoryDirection::Redo => self.old.clone(),
        }
    }

    pub(super) fn expected_text(&self, direction: UiTextHistoryDirection) -> &str {
        match direction {
            UiTextHistoryDirection::Undo => &self.inserted,
            UiTextHistoryDirection::Redo => &self.removed,
        }
    }

    pub(super) fn transition(
        &self,
        mut current: UiEditableTextState,
        direction: UiTextHistoryDirection,
    ) -> Option<TextEditStateTransition> {
        let range = self.expected_range(direction);
        if current.text.get(range.clone()) != Some(self.expected_text(direction)) {
            return None;
        }
        let (replacement, interaction, kind) = match direction {
            UiTextHistoryDirection::Undo => (&self.removed, &self.before, UiTextEditKind::Undo),
            UiTextHistoryDirection::Redo => (&self.inserted, &self.after, UiTextEditKind::Redo),
        };
        let new_end = range.start.checked_add(replacement.len())?;
        current.text.replace_range(range.clone(), replacement);
        interaction.apply_to(&mut current);
        Some(TextEditStateTransition {
            state: current,
            committed: Some(CommittedTextEditIntent {
                old: range.clone(),
                new: range.start..new_end,
                kind,
            }),
        })
    }
}

struct UiTextInteractionState {
    caret: UiTextCaret,
    selection: Option<UiTextSelection>,
}

impl UiTextInteractionState {
    fn from_state(state: &UiEditableTextState) -> Self {
        Self {
            caret: state.caret.clone(),
            selection: state.selection.clone(),
        }
    }

    fn for_composition_source(range: &Range<usize>) -> Self {
        let selection = (!range.is_empty()).then(|| UiTextSelection {
            anchor: range.start,
            focus: range.end,
        });
        Self {
            caret: UiTextCaret {
                offset: range.end,
                affinity: UiTextCaretAffinity::Downstream,
            },
            selection,
        }
    }

    fn apply_to(&self, state: &mut UiEditableTextState) {
        state.caret = self.caret.clone();
        state.selection = self.selection.clone();
        state.composition = None;
    }
}

#[derive(Default)]
pub(super) struct UiTextDocumentHistory {
    undo: VecDeque<UiTextHistoryEntry>,
    redo: VecDeque<UiTextHistoryEntry>,
    retained_bytes: usize,
}

impl UiTextDocumentHistory {
    pub(super) fn latest(&self, direction: UiTextHistoryDirection) -> Option<&UiTextHistoryEntry> {
        match direction {
            UiTextHistoryDirection::Undo => self.undo.back(),
            UiTextHistoryDirection::Redo => self.redo.back(),
        }
    }

    pub(super) fn commit(&mut self, commit: UiTextHistoryCommit) {
        match commit {
            UiTextHistoryCommit::Record(entry) => self.record(entry),
            UiTextHistoryCommit::Barrier => self.clear(),
            UiTextHistoryCommit::Undo => {
                if let Some(entry) = self.undo.pop_back() {
                    self.redo.push_back(entry);
                }
            }
            UiTextHistoryCommit::Redo => {
                if let Some(entry) = self.redo.pop_back() {
                    self.undo.push_back(entry);
                }
            }
        }
    }

    pub(super) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.retained_bytes = 0;
    }

    fn record(&mut self, entry: UiTextHistoryEntry) {
        self.clear_redo();
        self.retained_bytes = self.retained_bytes.saturating_add(entry.retained_bytes());
        self.undo.push_back(entry);
        while self.undo.len() > MVP_TEXT_HISTORY_MAX_ENTRIES
            || self.retained_bytes > MVP_TEXT_HISTORY_MAX_DELTA_BYTES
        {
            let Some(removed) = self.undo.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(removed.retained_bytes());
        }
    }

    fn clear_redo(&mut self) {
        for entry in self.redo.drain(..) {
            self.retained_bytes = self.retained_bytes.saturating_sub(entry.retained_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::surface::UiTextRange;

    fn state(text: &str, caret: usize) -> UiEditableTextState {
        UiEditableTextState {
            text: text.to_string(),
            caret: UiTextCaret {
                offset: caret,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn delta_entry_round_trips_text_and_interaction_state() {
        let mut before = state("abcd", 3);
        before.selection = Some(UiTextSelection {
            anchor: 1,
            focus: 3,
        });
        let after = state("aZd", 2);
        let entry = UiTextHistoryEntry::new(
            &CommittedTextEditIntent {
                old: 1..3,
                new: 1..2,
                kind: UiTextEditKind::Replace,
            },
            "bc".to_string(),
            "Z".to_string(),
            &before,
            &after,
        );

        let undone = entry
            .transition(after, UiTextHistoryDirection::Undo)
            .expect("undo transition");
        assert_eq!(undone.state.text, "abcd");
        assert_eq!(undone.state.selection, before.selection);
        let redone = entry
            .transition(undone.state, UiTextHistoryDirection::Redo)
            .expect("redo transition");
        assert_eq!(redone.state.text, "aZd");
        assert_eq!(redone.state.caret.offset, 2);
    }

    #[test]
    fn composition_history_restores_the_committed_source_selection() {
        let mut preedit = state("aXYd", 3);
        preedit.composition = Some(zircon_runtime_interface::ui::surface::UiTextComposition {
            range: UiTextRange { start: 1, end: 3 },
            text: "XY".to_string(),
            restore_text: Some("bc".to_string()),
            preedit_clauses: Vec::new(),
        });
        let after = state("aZd", 2);
        let entry = UiTextHistoryEntry::new(
            &CommittedTextEditIntent {
                old: 1..3,
                new: 1..2,
                kind: UiTextEditKind::CompositionCommit,
            },
            "bc".to_string(),
            "Z".to_string(),
            &preedit,
            &after,
        );

        let undone = entry
            .transition(after, UiTextHistoryDirection::Undo)
            .expect("undo transition");
        assert_eq!(undone.state.text, "abcd");
        assert_eq!(
            undone.state.selection,
            Some(UiTextSelection {
                anchor: 1,
                focus: 3,
            })
        );
    }

    #[test]
    fn history_bounds_entries_and_new_edits_discard_the_redo_branch() {
        let mut history = UiTextDocumentHistory::default();
        for _ in 0..=MVP_TEXT_HISTORY_MAX_ENTRIES {
            history.commit(UiTextHistoryCommit::Record(UiTextHistoryEntry::new(
                &CommittedTextEditIntent {
                    old: 0..0,
                    new: 0..1,
                    kind: UiTextEditKind::Insert,
                },
                String::new(),
                "x".to_string(),
                &state("", 0),
                &state("x", 1),
            )));
        }
        assert_eq!(history.undo.len(), MVP_TEXT_HISTORY_MAX_ENTRIES);
        assert_eq!(history.retained_bytes, MVP_TEXT_HISTORY_MAX_ENTRIES);

        history.commit(UiTextHistoryCommit::Undo);
        assert_eq!(history.redo.len(), 1);
        history.commit(UiTextHistoryCommit::Record(UiTextHistoryEntry::new(
            &CommittedTextEditIntent {
                old: 0..0,
                new: 0..1,
                kind: UiTextEditKind::Insert,
            },
            String::new(),
            "y".to_string(),
            &state("", 0),
            &state("y", 1),
        )));
        assert!(history.redo.is_empty());
        assert_eq!(history.undo.len(), MVP_TEXT_HISTORY_MAX_ENTRIES);
        assert_eq!(history.retained_bytes, MVP_TEXT_HISTORY_MAX_ENTRIES);
    }
}
