use std::sync::Arc;

use super::CommandEvalSnapshotHandle;
use crate::core::commands::{CommandEvalCtx, WhenClause};

#[test]
fn equivalent_context_reuses_the_command_eval_generation() {
    let handle = CommandEvalSnapshotHandle::default();

    assert_eq!(
        handle.snapshot_with_generation(),
        (0, CommandEvalCtx::default())
    );
    assert!(!handle.replace(CommandEvalCtx::default()));
    assert_eq!(handle.generation(), 0);
}

#[test]
fn semantic_context_changes_advance_the_generation_once() {
    let handle = CommandEvalSnapshotHandle::default();
    let selected = CommandEvalCtx::interactive().with_selection_count(1);

    assert!(handle.replace(selected.clone()));
    assert_eq!(handle.snapshot_with_generation(), (1, selected.clone()));
    assert!(!handle.replace(selected));
    assert_eq!(handle.generation(), 1);
}

#[test]
fn cloned_handles_share_one_versioned_snapshot() {
    let handle = CommandEvalSnapshotHandle::default();
    let clone = handle.clone();
    let project_open = CommandEvalCtx::interactive().with_project_open(true);

    assert!(clone.replace(project_open.clone()));
    assert_eq!(handle.snapshot_with_generation(), (1, project_open));
}

#[test]
fn shared_snapshot_reuses_one_context_arc_until_semantics_change() {
    let handle = CommandEvalSnapshotHandle::default();

    let first = handle.shared_snapshot();
    let second = handle.shared_snapshot();
    assert!(Arc::ptr_eq(&first, &second));

    assert!(handle.replace(CommandEvalCtx::interactive().with_project_open(true)));
    let changed = handle.shared_snapshot();
    assert!(!Arc::ptr_eq(&first, &changed));
    assert!(WhenClause::ProjectOpen.eval(changed.as_ref()));
}
