use std::collections::BTreeSet;

use crate::core::editing::engine::{
    HistoryContextId, HistoryDetailPage, HistoryRecordDetail, HistoryStatus, SelectionSnapshot,
    TransactionId,
};
use crate::core::editor_message::DocumentId;
use crate::ui::workbench::snapshot::TransactionHistorySnapshot;

fn record(sequence: u64, label: &str) -> HistoryRecordDetail {
    HistoryRecordDetail {
        id: TransactionId::from_sequence(sequence),
        label: label.to_string(),
        timestamp_frame: sequence * 10,
        command_count: sequence as usize,
        participants: BTreeSet::new(),
        selection_before: SelectionSnapshot::default(),
        selection_after: SelectionSnapshot::default(),
        significant: true,
    }
}

#[test]
fn history_projection_marks_the_applied_and_redo_segments_from_the_authoritative_top() {
    let history = HistoryContextId::Document(DocumentId::new(41));
    let page = HistoryDetailPage::new(
        HistoryStatus {
            len: 3,
            top: Some(TransactionId::from_sequence(2)),
            saved_top: Some(TransactionId::from_sequence(1)),
            saved_top_reachable: true,
            can_undo: true,
            can_redo: true,
            dirty: true,
            generation: 7,
        },
        vec![record(1, "Create"), record(2, "Rename"), record(3, "Move")],
        None,
    );

    let snapshot = TransactionHistorySnapshot::from_page(history, page);

    assert_eq!(snapshot.context, history);
    assert_eq!(snapshot.generation, 7);
    assert_eq!(snapshot.total_count, 3);
    assert!(!snapshot.truncated);
    assert_eq!(
        snapshot
            .rows
            .iter()
            .map(|row| (
                row.label.as_str(),
                row.applied,
                row.is_top,
                row.is_saved_top
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Create", true, false, true),
            ("Rename", true, true, false),
            ("Move", false, false, false),
        ]
    );
}

#[test]
fn history_projection_reports_a_bounded_page_without_inventing_application_state() {
    let history = HistoryContextId::Global;
    let page = HistoryDetailPage::new(
        HistoryStatus {
            len: 130,
            top: None,
            saved_top: None,
            saved_top_reachable: true,
            can_undo: false,
            can_redo: true,
            dirty: false,
            generation: 9,
        },
        vec![record(90, "Oldest visible"), record(91, "Next visible")],
        None,
    );

    let snapshot = TransactionHistorySnapshot::from_page(history, page);

    assert_eq!(snapshot.rows.len(), 2);
    assert!(snapshot.truncated);
    assert!(snapshot.rows.iter().all(|row| !row.applied));
    assert!(snapshot.can_redo);
    assert!(!snapshot.can_undo);
}

#[test]
fn history_projection_marks_the_visible_prefix_applied_when_top_is_on_a_later_page() {
    let history = HistoryContextId::Global;
    let page = HistoryDetailPage::new(
        HistoryStatus {
            len: 130,
            top: Some(TransactionId::from_sequence(130)),
            saved_top: None,
            saved_top_reachable: true,
            can_undo: true,
            can_redo: false,
            dirty: true,
            generation: 10,
        },
        vec![record(1, "Oldest visible"), record(2, "Next visible")],
        None,
    );

    let snapshot = TransactionHistorySnapshot::from_page(history, page);

    assert!(snapshot.truncated);
    assert!(snapshot.rows.iter().all(|row| row.applied));
    assert!(snapshot.rows.iter().all(|row| !row.is_top));
}
