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
