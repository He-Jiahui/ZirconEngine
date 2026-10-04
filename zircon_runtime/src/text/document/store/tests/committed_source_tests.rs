use super::{TextDocumentStore, TextDocumentStoreError, TextDocumentStoreLimits};

#[test]
fn source_equality_compares_piece_text_without_creating_a_snapshot() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("aé界d").expect("open source");
    let edited = store
        .replace(opened.document_id, opened.revision, 1..3, "🙂")
        .expect("piece edit");
    let revision = edited.revision();
    let before = store.report();
    assert_eq!(before.current_snapshot_bytes, 0);

    assert_eq!(
        store.source_equals(opened.document_id, revision, "a🙂界d"),
        Ok(true)
    );
    assert_eq!(
        store.source_equals(opened.document_id, revision, "a🙂界x"),
        Ok(false)
    );
    assert_eq!(
        store.source_equals(opened.document_id, revision, "a🙂界"),
        Ok(false)
    );
    let after = store.report();
    assert_eq!(after.current_snapshot_bytes, 0);
    assert_eq!(after.active_snapshot_lease_count, 0);
    assert_eq!(after.current_document_bytes, before.current_document_bytes);
    assert_eq!(after.retained_source_bytes, before.retained_source_bytes);
}

#[test]
fn source_equality_requires_the_bound_revision_and_live_document() {
    let mut store = TextDocumentStore::with_limits(limits());
    let opened = store.open("abc").expect("open source");
    let edited = store
        .replace(opened.document_id, opened.revision, 2..3, "d")
        .expect("source edit");
    let revision = edited.revision();
    assert_eq!(
        store.source_equals(opened.document_id, opened.revision, "abd"),
        Err(TextDocumentStoreError::StaleRevision {
            expected: opened.revision,
            actual: revision,
        })
    );
    assert!(store.close(opened.document_id));
    assert_eq!(
        store.source_equals(opened.document_id, revision, "abd"),
        Err(TextDocumentStoreError::UnknownDocument)
    );
}

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
