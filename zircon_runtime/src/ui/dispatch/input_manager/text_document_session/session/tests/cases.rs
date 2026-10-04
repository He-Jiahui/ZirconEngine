use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    surface::UiEditableTextState,
    text::{UiTextByteSelection, UiTextEditKind, UiTextEditSource},
    tree::UiTreeNode,
};

use crate::ui::surface::UiSurface;

use super::{
    TextDocumentStoreEditCommit, UiTextDocumentBinding, UiTextDocumentBindingKey,
    UiTextDocumentSession, UiTextDocumentSessionError, UiTextHistoryCommit, UiTextHistoryDirection,
    UiTextHistoryEntry,
};
use crate::ui::text::CommittedTextEditIntent;

#[test]
fn detached_owner_closes_its_retained_document() {
    let tree_id = UiTreeId::new("text-document-session.detach");
    let node_id = UiNodeId::new(7);
    let mut surface = UiSurface::new(tree_id.clone());
    surface.tree.nodes.insert(
        node_id,
        UiTreeNode::new(node_id, UiNodePath::new("root/editor")),
    );
    let mut session = UiTextDocumentSession::default();
    session.synchronize_owners(&surface.tree, surface.session_identity());
    session.synchronize_source(&tree_id, node_id, 0, "retained");
    assert_eq!(session.store.report().document_count, 1);

    surface.tree.nodes.remove(&node_id);
    session.synchronize_owners(&surface.tree, surface.session_identity());

    assert!(session.bindings.is_empty());
    assert_eq!(session.store.report().document_count, 0);
}

#[test]
fn retained_grapheme_query_requires_the_synchronized_source_epoch() {
    let tree_id = UiTreeId::new("text-document-session.graphemes");
    let node_id = UiNodeId::new(8);
    let mut session = UiTextDocumentSession::default();
    session.synchronize_source(&tree_id, node_id, 4, "a\u{0301}bc");

    assert_eq!(
        session
            .retained_grapheme_count(&tree_id, node_id, 4, 3..4)
            .expect("the synchronized revision owns the grapheme index"),
        2
    );
    assert_eq!(
        session.retained_grapheme_count(&tree_id, node_id, 3, 3..4),
        Err(super::super::UiTextDocumentSessionError::SourceNotSynchronized)
    );
}

#[test]
fn finish_edit_reuses_one_tree_key_for_binding_error_and_history() {
    let source = include_str!("../../session.rs");
    let finish_edit = source
        .split("pub(in crate::ui) fn finish_edit(")
        .nth(1)
        .expect("finish_edit exists")
        .split("    fn binding(")
        .next()
        .expect("binding follows finish_edit");
    assert_eq!(finish_edit.matches("tree_id.clone()").count(), 1);
}

#[test]
fn finish_edit_preserves_unchanged_and_changed_history_policy() {
    let tree_id = UiTreeId::new("text-document-session.finish-edit");
    let node_id = UiNodeId::new(9);
    let key = UiTextDocumentBindingKey {
        tree_id: tree_id.clone(),
        node_id,
    };
    let mut session = UiTextDocumentSession::default();
    session.synchronize_source(&tree_id, node_id, 0, "a");
    let document = session
        .document_key(&tree_id, node_id, 0)
        .expect("source is synchronized");
    let unchanged = TextDocumentStoreEditCommit::Unchanged {
        document_id: document.document_id,
        revision: document.revision,
    };
    session.histories.insert(key.clone(), Default::default());
    session.synchronization_errors.insert(
        key.clone(),
        UiTextDocumentSessionError::SourceNotSynchronized,
    );
    session.finish_edit(
        &tree_id,
        node_id,
        1,
        &unchanged,
        UiTextHistoryCommit::Barrier,
    );
    assert!(session.histories.contains_key(&key));
    assert_eq!(session.bindings[&key].source_epoch, 1);
    assert!(!session.synchronization_errors.contains_key(&key));

    let changed = session
        .store
        .replace_with_receipt(
            document.document_id,
            document.revision,
            0..1,
            "A",
            node_id,
            UiTextEditSource::Keyboard,
            UiTextEditKind::Replace,
            UiTextByteSelection::collapsed(1),
        )
        .expect("changed edit commits");
    let TextDocumentStoreEditCommit::Changed { public_receipt, .. } = &changed else {
        panic!("replacement should produce a changed receipt");
    };
    let intent = CommittedTextEditIntent {
        old: 0..1,
        new: 0..1,
        kind: UiTextEditKind::Replace,
    };
    let before = UiEditableTextState {
        text: "a".to_owned(),
        ..Default::default()
    };
    let after = UiEditableTextState {
        text: "A".to_owned(),
        ..Default::default()
    };
    let entry = UiTextHistoryEntry::new(&intent, "a".into(), "A".into(), &before, &after);
    session.finish_edit(
        &tree_id,
        node_id,
        2,
        &changed,
        UiTextHistoryCommit::Record(entry),
    );
    assert_eq!(session.bindings[&key].revision, public_receipt.revision);
    assert_eq!(session.bindings[&key].source_epoch, 2);
    assert_eq!(
        session.histories[&key]
            .latest(UiTextHistoryDirection::Undo)
            .expect("changed edit is recorded")
            .expected_text(UiTextHistoryDirection::Undo),
        "A"
    );
    session.finish_edit(&tree_id, node_id, 3, &changed, UiTextHistoryCommit::Barrier);
    assert!(!session.histories.contains_key(&key));
    assert_eq!(session.bindings[&key].source_epoch, 3);
}

// Exact pre-optimization finish_edit, retained only as a Release comparison.
fn legacy_finish_edit(
    session: &mut UiTextDocumentSession,
    tree_id: &UiTreeId,
    node_id: UiNodeId,
    source_epoch: u64,
    commit: &TextDocumentStoreEditCommit,
    history_commit: UiTextHistoryCommit,
) {
    let (document_id, revision) = match commit {
        TextDocumentStoreEditCommit::Unchanged {
            document_id,
            revision,
        } => (*document_id, *revision),
        TextDocumentStoreEditCommit::Changed { public_receipt, .. } => {
            (public_receipt.document_id, public_receipt.revision)
        }
    };
    session.bindings.insert(
        UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        },
        UiTextDocumentBinding {
            document_id,
            revision,
            source_epoch,
        },
    );
    session
        .synchronization_errors
        .remove(&UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        });
    if matches!(commit, TextDocumentStoreEditCommit::Changed { .. }) {
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        match history_commit {
            UiTextHistoryCommit::Barrier => {
                session.histories.remove(&key);
            }
            history_commit => session
                .histories
                .entry(key)
                .or_default()
                .commit(history_commit),
        }
    }
}

fn changed_barrier_benchmark_case() -> (
    UiTextDocumentSession,
    UiTreeId,
    UiNodeId,
    TextDocumentStoreEditCommit,
) {
    let tree_id = UiTreeId::new("runtime-document-session/".repeat(6));
    let node_id = UiNodeId::new(10);
    let mut session = UiTextDocumentSession::default();
    session.synchronize_source(&tree_id, node_id, 0, "a");
    let document = session
        .document_key(&tree_id, node_id, 0)
        .expect("benchmark source is synchronized");
    let commit = session
        .store
        .replace_with_receipt(
            document.document_id,
            document.revision,
            0..1,
            "A",
            node_id,
            UiTextEditSource::Keyboard,
            UiTextEditKind::Replace,
            UiTextByteSelection::collapsed(1),
        )
        .expect("benchmark changed edit commits");
    (session, tree_id, node_id, commit)
}

fn finish_edit_sample(legacy: bool, iterations: usize) -> u128 {
    use std::hint::black_box;
    use std::time::Instant;

    let (mut session, tree_id, node_id, commit) = changed_barrier_benchmark_case();
    let start = Instant::now();
    for iteration in 0..iterations {
        if legacy {
            legacy_finish_edit(
                &mut session,
                &tree_id,
                node_id,
                iteration as u64 + 1,
                black_box(&commit),
                UiTextHistoryCommit::Barrier,
            );
        } else {
            session.finish_edit(
                &tree_id,
                node_id,
                iteration as u64 + 1,
                black_box(&commit),
                UiTextHistoryCommit::Barrier,
            );
        }
    }
    black_box(session.bindings.len());
    start.elapsed().as_nanos()
}

#[test]
#[ignore = "release-only performance evidence"]
fn finish_edit_single_key_release_p95() {
    const MARKER: &str = "RUNTIME82_FINISH_EDIT_SINGLE_KEY_BENCH_V1";
    const WARMUPS: usize = 4;
    const SAMPLES: usize = 17;
    const ITERATIONS: usize = 16_384;

    let (mut legacy, tree_id, node_id, commit) = changed_barrier_benchmark_case();
    let (mut optimized, _, _, _) = changed_barrier_benchmark_case();
    legacy_finish_edit(
        &mut legacy,
        &tree_id,
        node_id,
        1,
        &commit,
        UiTextHistoryCommit::Barrier,
    );
    optimized.finish_edit(&tree_id, node_id, 1, &commit, UiTextHistoryCommit::Barrier);
    assert_eq!(legacy.bindings, optimized.bindings);
    assert_eq!(legacy.histories.len(), optimized.histories.len());
    assert_eq!(
        legacy.synchronization_errors,
        optimized.synchronization_errors
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for pair in 0..(WARMUPS + SAMPLES) {
        let (old, new) = if pair % 2 == 0 {
            (
                finish_edit_sample(true, ITERATIONS),
                finish_edit_sample(false, ITERATIONS),
            )
        } else {
            let new = finish_edit_sample(false, ITERATIONS);
            let old = finish_edit_sample(true, ITERATIONS);
            (old, new)
        };
        if pair >= WARMUPS {
            legacy_samples.push(old);
            optimized_samples.push(new);
        }
    }
    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let p95_index = (SAMPLES * 95).div_ceil(100) - 1;
    let legacy_p95_ns = legacy_samples[p95_index];
    let optimized_p95_ns = optimized_samples[p95_index];
    println!(
        "{MARKER} iterations={ITERATIONS} samples={SAMPLES} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio_pct={}",
        optimized_p95_ns * 100 / legacy_p95_ns
    );
    assert!(optimized_p95_ns * 100 <= legacy_p95_ns * 90);
}
