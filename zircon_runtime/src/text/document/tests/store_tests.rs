use super::{
    TextDocumentAdmissionFailure, TextDocumentEditOutcome, TextDocumentStore,
    TextDocumentStoreEditCommit, TextDocumentStoreError, TextDocumentStoreLimits,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    text::{UiTextByteSelection, UiTextDocumentRevision, UiTextEditKind, UiTextEditSource},
};

fn limits() -> TextDocumentStoreLimits {
    TextDocumentStoreLimits {
        max_documents: 4,
        max_document_bytes: 64,
        max_total_document_bytes: 128,
        max_replacement_bytes: 32,
        max_retained_source_bytes_per_document: 128,
        max_total_retained_source_bytes: 256,
        max_addition_sources_per_document: 1,
        max_pieces_per_document: 16,
        max_current_snapshot_bytes: 128,
        max_active_snapshot_leases: 4,
        max_active_snapshot_lease_bytes: 128,
    }
}

#[test]
fn store_requires_explicit_capacity_and_keeps_rejections_content_free() {
    let mut policy = limits();
    policy.max_documents = 1;
    let mut store = TextDocumentStore::with_limits(policy);
    let first = store.open("alpha").expect("first document is admitted");

    let error = store
        .open("private rejected source")
        .expect_err("the document-count limit rejects another owner");

    assert_eq!(
        error,
        TextDocumentStoreError::AdmissionDenied(TextDocumentAdmissionFailure::DocumentCount)
    );
    assert_eq!(store.report().document_count, 1);
    assert_eq!(store.report().current_document_bytes, 5);
    assert!(!format!("{store:?}").contains("alpha"));
    assert_eq!(first.revision, UiTextDocumentRevision::new(0));
}

#[test]
fn rejected_edit_and_no_op_at_the_retained_source_limit_do_not_mutate() {
    let mut policy = limits();
    policy.max_retained_source_bytes_per_document = 5;
    policy.max_total_retained_source_bytes = 5;
    let mut store = TextDocumentStore::with_limits(policy);
    let opened = store.open("alpha").expect("document is admitted");

    let unchanged = store
        .replace(opened.document_id, opened.revision, 0..5, "alpha")
        .expect("a no-op remains legal at the capacity limit");
    assert!(matches!(
        unchanged,
        TextDocumentEditOutcome::Unchanged { .. }
    ));

    let error = store
        .replace(opened.document_id, opened.revision, 0..5, "ALPHA")
        .expect_err("a changed edit cannot exceed retained source capacity");
    assert_eq!(
        error,
        TextDocumentStoreError::AdmissionDenied(
            TextDocumentAdmissionFailure::DocumentRetainedSourceBytes
        )
    );
    let lease = store
        .snapshot(opened.document_id, opened.revision)
        .expect("the rejected edit leaves the original revision intact");
    assert_eq!(lease.as_str(), "alpha");
}

#[test]
fn addition_source_limit_rejects_before_publishing_the_first_added_source() {
    let mut policy = limits();
    policy.max_addition_sources_per_document = 0;
    let mut store = TextDocumentStore::with_limits(policy);
    let opened = store.open("").expect("empty document is admitted");
    let before = store.report();

    let error = store
        .replace(opened.document_id, opened.revision, 0..0, "a")
        .expect_err("a policy can deny creating the addition source");
    assert_eq!(
        error,
        TextDocumentStoreError::AdmissionDenied(TextDocumentAdmissionFailure::AdditionSources)
    );
    assert_eq!(store.report(), before);
    assert_eq!(
        store
            .snapshot(opened.document_id, opened.revision)
            .expect("the original document remains readable")
            .as_str(),
        ""
    );
}

#[test]
fn piece_limit_rejects_a_fragmenting_edit_without_publishing_it() {
    let mut policy = limits();
    policy.max_pieces_per_document = 3;
    let mut store = TextDocumentStore::with_limits(policy);
    let opened = store.open("abcd").expect("document is admitted");
    let first = store
        .replace(opened.document_id, opened.revision, 2..2, "x")
        .expect("three-piece middle insertion is admitted");
    let first_revision = first.revision();
    let before = store.report();

    let error = store
        .replace(opened.document_id, first_revision, 1..1, "y")
        .expect_err("the fragmenting edit exceeds the piece limit");
    assert_eq!(
        error,
        TextDocumentStoreError::AdmissionDenied(TextDocumentAdmissionFailure::Pieces)
    );
    assert_eq!(store.report(), before);
    assert_eq!(
        store
            .snapshot(opened.document_id, first_revision)
            .expect("the admitted revision remains readable")
            .as_str(),
        "abxcd"
    );
}

#[test]
fn snapshot_leases_are_revision_checked_and_release_budget_on_drop() {
    let mut policy = limits();
    policy.max_active_snapshot_leases = 1;
    policy.max_active_snapshot_lease_bytes = 5;
    let mut store = TextDocumentStore::with_limits(policy);
    let opened = store.open("alpha").expect("document is admitted");
    let first = store
        .snapshot(opened.document_id, opened.revision)
        .expect("first lease is admitted");

    assert_eq!(first.document_id(), opened.document_id);
    assert_eq!(first.revision(), opened.revision);
    assert_eq!(store.report().active_snapshot_lease_count, 1);
    assert_eq!(store.report().active_snapshot_lease_bytes, 5);
    assert!(matches!(
        store.snapshot(opened.document_id, opened.revision),
        Err(TextDocumentStoreError::AdmissionDenied(
            TextDocumentAdmissionFailure::ActiveSnapshotLeaseCount
        ))
    ));

    drop(first);
    assert_eq!(store.report().active_snapshot_lease_count, 0);
    let second = store
        .snapshot(opened.document_id, opened.revision)
        .expect("dropping the first lease releases its budget");
    assert_eq!(second.as_str(), "alpha");
}

#[test]
fn changed_snapshot_is_denied_before_flattening_when_current_snapshot_budget_is_full() {
    let mut policy = limits();
    policy.max_current_snapshot_bytes = 4;
    let mut store = TextDocumentStore::with_limits(policy);
    let opened = store.open("alpha").expect("document is admitted");
    let changed = store
        .replace(opened.document_id, opened.revision, 0..1, "A")
        .expect("same-length edit is admitted");
    let revision = changed.revision();

    let error = store
        .snapshot(opened.document_id, revision)
        .expect_err("flattening five current bytes exceeds the four-byte budget");

    assert_eq!(
        error,
        TextDocumentStoreError::AdmissionDenied(TextDocumentAdmissionFailure::CurrentSnapshotBytes)
    );
    assert_eq!(store.report().current_snapshot_bytes, 0);
}

#[test]
fn grapheme_query_is_revision_checked_and_accounts_for_its_flattened_snapshot() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("a\u{0301}bc").expect("document is admitted");
    let changed = store
        .replace(opened.document_id, opened.revision, 3..4, "B")
        .expect("same-length edit is admitted");

    assert!(matches!(
        store.retained_grapheme_count(opened.document_id, opened.revision, 0..3),
        Err(TextDocumentStoreError::StaleRevision { .. })
    ));
    assert_eq!(store.report().current_snapshot_bytes, 0);
    assert_eq!(
        store
            .retained_grapheme_count(opened.document_id, changed.revision(), 0..3)
            .expect("the current revision index is admitted"),
        2
    );
    assert_eq!(store.report().current_snapshot_bytes, "a\u{0301}Bc".len());
}

#[test]
fn grapheme_query_is_denied_before_indexing_when_snapshot_budget_is_full() {
    let mut policy = limits();
    policy.max_current_snapshot_bytes = 4;
    let mut store = TextDocumentStore::with_limits(policy);
    let opened = store.open("alpha").expect("document is admitted");
    let changed = store
        .replace(opened.document_id, opened.revision, 0..1, "A")
        .expect("same-length edit is admitted");

    assert_eq!(
        store.retained_grapheme_count(opened.document_id, changed.revision(), 0..1),
        Err(TextDocumentStoreError::AdmissionDenied(
            TextDocumentAdmissionFailure::CurrentSnapshotBytes
        ))
    );
    assert_eq!(store.report().current_snapshot_bytes, 0);
}

#[test]
fn removing_a_document_releases_document_and_retained_source_totals() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("alpha").expect("document is admitted");
    store
        .replace(opened.document_id, opened.revision, 0..1, "A")
        .expect("replacement is admitted");
    assert_eq!(store.report().document_count, 1);
    assert_eq!(store.report().retained_source_bytes, 6);

    assert!(store.close(opened.document_id));
    assert_eq!(store.report().document_count, 0);
    assert_eq!(store.report().current_document_bytes, 0);
    assert_eq!(store.report().retained_source_bytes, 0);
    assert!(!store.close(opened.document_id));
}

#[test]
fn incremental_residency_tracks_open_edit_snapshot_replacement_and_close() {
    let mut store = TextDocumentStore::with_limits(limits());
    let alpha = store.open("alpha").expect("first document is admitted");
    let beta = store.open("beta").expect("second document is admitted");
    assert_eq!(
        store.report(),
        super::TextDocumentStoreReport {
            document_count: 2,
            current_document_bytes: 9,
            retained_source_bytes: 9,
            ..super::TextDocumentStoreReport::default()
        }
    );

    let first = store
        .replace(alpha.document_id, alpha.revision, 0..1, "A")
        .expect("first replacement is admitted");
    assert_eq!(store.report().current_document_bytes, 9);
    assert_eq!(store.report().retained_source_bytes, 10);
    assert_eq!(store.report().current_snapshot_bytes, 0);

    let first_lease = store
        .snapshot(alpha.document_id, first.revision())
        .expect("changed revision snapshot is admitted");
    assert_eq!(first_lease.as_str(), "Alpha");
    assert_eq!(store.report().current_snapshot_bytes, 5);
    drop(first_lease);

    let second = store
        .replace(alpha.document_id, first.revision(), 1..2, "L")
        .expect("second replacement is admitted");
    assert_eq!(second.revision(), UiTextDocumentRevision::new(2));
    assert_eq!(store.report().current_document_bytes, 9);
    assert_eq!(store.report().retained_source_bytes, 11);
    assert_eq!(store.report().current_snapshot_bytes, 0);

    assert!(store.close(beta.document_id));
    assert_eq!(store.report().document_count, 1);
    assert_eq!(store.report().current_document_bytes, 5);
    assert_eq!(store.report().retained_source_bytes, 7);
    assert!(store.close(alpha.document_id));
    assert_eq!(store.report(), super::TextDocumentStoreReport::default());
}

#[test]
fn changed_store_commit_validates_and_returns_the_public_receipt_atomically() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("alpha").expect("document is admitted");

    let commit = store
        .replace_with_receipt(
            opened.document_id,
            opened.revision,
            0..1,
            "A",
            UiNodeId::new(901),
            UiTextEditSource::Keyboard,
            UiTextEditKind::Replace,
            UiTextByteSelection::collapsed(1),
        )
        .expect("edit and public receipt commit together");

    let TextDocumentStoreEditCommit::Changed {
        internal_receipt,
        public_receipt,
    } = commit
    else {
        panic!("expected a changed store commit");
    };
    assert_eq!(internal_receipt.document_id, opened.document_id);
    assert_eq!(public_receipt.document_id, opened.document_id);
    assert_eq!(public_receipt.previous_revision, opened.revision);
    assert_eq!(public_receipt.revision, UiTextDocumentRevision::new(1));
    assert_eq!(public_receipt.changed.old.start_byte, 0);
    assert_eq!(public_receipt.changed.old.end_byte, 1);
    assert_eq!(public_receipt.changed.new.start_byte, 0);
    assert_eq!(public_receipt.changed.new.end_byte, 1);
}

#[test]
fn invalid_public_receipt_projection_rejects_before_document_commit() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("alpha").expect("document is admitted");

    let error = store
        .replace_with_receipt(
            opened.document_id,
            opened.revision,
            0..1,
            "A",
            UiNodeId::new(902),
            UiTextEditSource::Keyboard,
            UiTextEditKind::Replace,
            UiTextByteSelection::collapsed(6),
        )
        .expect_err("out-of-bounds final selection rejects before commit");

    assert!(matches!(
        error,
        TextDocumentStoreError::ReceiptProjection(
            super::TextDocumentReceiptProjectionError::SelectionOutOfBounds
        )
    ));
    let lease = store
        .snapshot(opened.document_id, opened.revision)
        .expect("the old revision remains current");
    assert_eq!(lease.as_str(), "alpha");
}

#[test]
fn dropping_a_prepared_store_edit_publishes_no_revision() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("alpha").expect("document is admitted");
    let report_before = store.report();

    let prepared = store
        .prepare_replace_with_receipt(
            opened.document_id,
            opened.revision,
            0..1,
            "A",
            UiNodeId::new(903),
            UiTextEditSource::Keyboard,
            UiTextEditKind::Replace,
            UiTextByteSelection::collapsed(1),
        )
        .expect("edit and public projection prepare together");
    assert_eq!(
        prepared
            .public_receipt()
            .expect("changed edit has a prospective receipt")
            .revision,
        UiTextDocumentRevision::new(1)
    );
    drop(prepared);

    assert_eq!(store.report(), report_before);

    let lease = store
        .snapshot(opened.document_id, opened.revision)
        .expect("discarding prepare leaves the old revision current");
    assert_eq!(lease.as_str(), "alpha");
}

#[test]
fn prepared_store_edit_commit_is_infallible_after_exclusive_preflight() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("alpha").expect("document is admitted");

    let prepared = store
        .prepare_replace_with_receipt(
            opened.document_id,
            opened.revision,
            0..1,
            "A",
            UiNodeId::new(904),
            UiTextEditSource::Keyboard,
            UiTextEditKind::Replace,
            UiTextByteSelection::collapsed(1),
        )
        .expect("edit and public projection prepare together");
    let prospective = prepared
        .public_receipt()
        .cloned()
        .expect("changed edit has a prospective receipt");
    let TextDocumentStoreEditCommit::Changed { public_receipt, .. } = prepared.commit() else {
        panic!("expected changed commit");
    };

    assert_eq!(public_receipt, prospective);
    assert_eq!(public_receipt.revision, UiTextDocumentRevision::new(1));
}

#[test]
fn source_range_reads_across_pieces_and_rejects_stale_revisions() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("abcdef").expect("document is admitted");
    let changed = store
        .replace(opened.document_id, opened.revision, 2..4, "WXYZ")
        .expect("replacement is admitted");

    assert_eq!(
        store.source_range(opened.document_id, changed.revision(), 1..8),
        Ok("bWXYZef".to_string())
    );
    assert_eq!(
        store.source_range(opened.document_id, opened.revision, 0..1),
        Err(TextDocumentStoreError::StaleRevision {
            expected: opened.revision,
            actual: changed.revision(),
        })
    );
}
