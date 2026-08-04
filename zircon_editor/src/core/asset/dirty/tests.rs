use std::any::Any;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext, EditorTransactionEngine,
    HistoryContextId, HistoryDirtyBatch, HistoryDirtyCursor, SelectionSnapshot,
};
use crate::core::editor_message::DocumentId;
use crate::core::gateway::EditorRuntimeGatewayHandle;

use super::registry::DirtyTransactionStateSource;
use super::{DirtyExternalEffectId, DirtyRegistry, DirtyRegistryError};

struct FixtureContext {
    gateway: EditorRuntimeGatewayHandle,
}

impl Default for FixtureContext {
    fn default() -> Self {
        Self {
            gateway: EditorRuntimeGatewayHandle::detached(),
        }
    }
}

impl EditContext for FixtureContext {
    fn runtime_gateway(&self) -> &EditorRuntimeGatewayHandle {
        &self.gateway
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn restore_selection(&mut self, _snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct AppliedCommand(&'static str);

impl EditCommand for AppliedCommand {
    fn label(&self) -> &str {
        self.0
    }

    fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn document(value: u64) -> DocumentId {
    DocumentId::new(value)
}

fn effect(value: &str) -> DirtyExternalEffectId {
    DirtyExternalEffectId::parse(value).unwrap()
}

fn registry() -> (Arc<EditorTransactionEngine>, DirtyRegistry) {
    let transactions = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let registry = DirtyRegistry::new(Arc::clone(&transactions));
    (transactions, registry)
}

fn commit(transactions: &EditorTransactionEngine, document: DocumentId, label: &'static str) {
    let mut scope = transactions
        .begin(label, HistoryContextId::Document(document))
        .unwrap();
    scope.push(AppliedCommand(label)).unwrap();
    scope.commit().unwrap();
}

#[test]
fn saved_top_is_queried_live_without_registry_dirty_mutations() {
    let (transactions, registry) = registry();
    let document = document(7);
    assert!(registry.register_document(document).unwrap());
    assert!(!registry.snapshot(document).unwrap().is_dirty());

    commit(&transactions, document, "edit");
    assert!(registry.snapshot(document).unwrap().transaction_dirty());
    let token = registry.capture_save_token(document).unwrap();
    registry.mark_saved_if_unchanged(document, token).unwrap();
    assert!(!registry.snapshot(document).unwrap().is_dirty());

    transactions
        .undo(HistoryContextId::Document(document))
        .unwrap();
    assert!(registry.snapshot(document).unwrap().is_dirty());
    transactions
        .redo(HistoryContextId::Document(document))
        .unwrap();
    assert!(!registry.snapshot(document).unwrap().is_dirty());
}

#[test]
fn save_token_bridge_rejects_a_transaction_committed_during_save() {
    let (transactions, registry) = registry();
    let document = document(8);
    registry.register_document(document).unwrap();
    commit(&transactions, document, "before save");
    let token = registry.capture_save_token(document).unwrap();

    commit(&transactions, document, "during save");
    let error = registry
        .mark_saved_if_unchanged(document, token)
        .unwrap_err();

    assert!(matches!(
        error,
        DirtyRegistryError::Transaction(EditCommandError::HistoryChangedDuringSave { .. })
    ));
    assert!(registry.snapshot(document).unwrap().transaction_dirty());
}

#[test]
fn external_effects_are_typed_sorted_and_independently_clearable() {
    let (_transactions, registry) = registry();
    let document = document(3);
    registry.register_document(document).unwrap();
    let source_revision = registry
        .mark_external_effect(document, effect("ui.source_buffer"))
        .unwrap();
    let settings_revision = registry
        .mark_external_effect(document, effect("asset.import_settings"))
        .unwrap();

    let snapshot = registry.snapshot(document).unwrap();
    assert!(!snapshot.transaction_dirty());
    assert_eq!(
        snapshot.external_effects(),
        &[effect("asset.import_settings"), effect("ui.source_buffer")]
    );
    assert!(snapshot.is_dirty());

    assert!(
        registry
            .clear_external_effect(
                document,
                &effect("asset.import_settings"),
                settings_revision,
            )
            .unwrap()
    );
    assert!(registry.snapshot(document).unwrap().is_dirty());
    assert!(
        registry
            .clear_external_effect(document, &effect("ui.source_buffer"), source_revision)
            .unwrap()
    );
    assert!(!registry.snapshot(document).unwrap().is_dirty());
}

#[test]
fn remarking_an_effect_advances_its_revision() {
    let (_transactions, registry) = registry();
    let document = document(5);
    registry.register_document(document).unwrap();
    let effect = effect("ui.source_buffer");

    let first = registry
        .mark_external_effect(document, effect.clone())
        .unwrap();
    let second = registry
        .mark_external_effect(document, effect.clone())
        .unwrap();

    assert!(second.value() > first.value());
    assert_eq!(
        registry
            .snapshot(document)
            .unwrap()
            .external_revision(&effect),
        Some(second)
    );
    assert!(
        !registry
            .clear_external_effect(document, &effect, first)
            .unwrap()
    );
    assert_eq!(
        registry
            .snapshot(document)
            .unwrap()
            .external_revision(&effect),
        Some(second)
    );
    assert!(
        registry
            .clear_external_effect(document, &effect, second)
            .unwrap()
    );
}

#[test]
fn unknown_documents_are_rejected_without_implicit_registration() {
    let (_transactions, registry) = registry();
    let missing = document(99);

    let snapshot_error = registry.snapshot(missing).unwrap_err();
    assert!(matches!(
        snapshot_error,
        DirtyRegistryError::DocumentNotRegistered { document } if document == missing
    ));
    let mark_error = registry
        .mark_external_effect(missing, effect("ui.source_buffer"))
        .unwrap_err();
    assert!(matches!(
        mark_error,
        DirtyRegistryError::DocumentNotRegistered { document } if document == missing
    ));
    assert!(registry.changes_since(None).unwrap().snapshots().is_empty());
}

#[test]
fn unregister_discards_only_that_documents_external_effects() {
    let (_transactions, registry) = registry();
    let first = document(1);
    let second = document(2);
    registry.register_document(first).unwrap();
    registry.register_document(second).unwrap();
    registry
        .mark_external_effect(first, effect("ui.source_buffer"))
        .unwrap();
    registry
        .mark_external_effect(second, effect("ui.source_buffer"))
        .unwrap();

    assert!(registry.unregister_document(first).unwrap());
    assert!(matches!(
        registry.snapshot(first),
        Err(DirtyRegistryError::DocumentNotRegistered { .. })
    ));
    assert!(registry.snapshot(second).unwrap().is_dirty());
}

#[test]
fn initial_dirty_delta_is_sorted_and_combines_both_sources() {
    let (transactions, registry) = registry();
    let later = document(9);
    let earlier = document(4);
    registry.register_document(later).unwrap();
    registry.register_document(earlier).unwrap();
    commit(&transactions, earlier, "transaction dirty");
    registry
        .mark_external_effect(later, effect("ui.source_buffer"))
        .unwrap();

    let delta = registry.changes_since(None).unwrap();
    assert!(delta.is_reset());
    let snapshots = delta.snapshots();
    assert_eq!(
        snapshots
            .iter()
            .map(|snapshot| snapshot.document())
            .collect::<Vec<_>>(),
        vec![earlier, later]
    );
    assert!(snapshots[0].transaction_dirty());
    assert!(snapshots[0].external_effects().is_empty());
    assert!(!snapshots[1].transaction_dirty());
    assert_eq!(
        snapshots[1].external_effects(),
        &[effect("ui.source_buffer")]
    );
}

#[test]
fn stable_cursor_returns_no_snapshots_or_removals() {
    let (_transactions, registry) = registry();
    registry.register_document(document(1)).unwrap();
    let baseline = registry.changes_since(None).unwrap();

    let stable = registry.changes_since(Some(baseline.cursor())).unwrap();

    assert!(!stable.is_reset());
    assert!(stable.snapshots().is_empty());
    assert!(stable.removed_documents().is_empty());
}

#[test]
fn dirty_delta_updates_only_changed_documents() {
    let (transactions, registry) = registry();
    let first = document(31);
    let second = document(32);
    registry.register_document(first).unwrap();
    registry.register_document(second).unwrap();
    let baseline = registry.changes_since(None).unwrap();

    registry
        .mark_external_effect(second, effect("ui.source_buffer"))
        .unwrap();
    let external_delta = registry.changes_since(Some(baseline.cursor())).unwrap();
    assert_eq!(external_delta.snapshots().len(), 1);
    assert_eq!(external_delta.snapshots()[0].document(), second);

    commit(&transactions, first, "transaction changed");
    let transaction_delta = registry
        .changes_since(Some(external_delta.cursor()))
        .unwrap();
    assert_eq!(transaction_delta.snapshots().len(), 1);
    assert_eq!(transaction_delta.snapshots()[0].document(), first);
    assert!(transaction_delta.snapshots()[0].transaction_dirty());
}

#[test]
fn unregister_is_published_as_a_typed_removal() {
    let (_transactions, registry) = registry();
    let removed = document(41);
    let retained = document(42);
    registry.register_document(removed).unwrap();
    registry.register_document(retained).unwrap();
    let baseline = registry.changes_since(None).unwrap();

    registry.unregister_document(removed).unwrap();
    let delta = registry.changes_since(Some(baseline.cursor())).unwrap();

    assert_eq!(delta.removed_documents(), &[removed]);
    assert!(delta.snapshots().is_empty());
}

#[test]
fn ten_thousand_document_stable_and_single_change_work_is_delta_bounded() {
    let (_transactions, registry) = registry();
    for value in 1..=10_000 {
        registry.register_document(document(value)).unwrap();
    }
    let baseline = registry.changes_since(None).unwrap();
    assert!(baseline.is_reset());
    assert_eq!(baseline.snapshots().len(), 10_000);

    let stable = registry.changes_since(Some(baseline.cursor())).unwrap();
    assert!(!stable.is_reset());
    assert!(stable.snapshots().is_empty());
    assert_eq!(registry.take_journal_visits_for_test(), 0);

    let changed = document(9_999);
    registry
        .mark_external_effect(changed, effect("ui.source_buffer"))
        .unwrap();
    let delta = registry.changes_since(Some(stable.cursor())).unwrap();
    assert!(!delta.is_reset());
    assert_eq!(delta.snapshots().len(), 1);
    assert_eq!(delta.snapshots()[0].document(), changed);
    assert_eq!(registry.take_journal_visits_for_test(), 1);
}

#[test]
fn dirty_delta_dispatches_optional_cursors_without_expect_invariants() {
    let source = include_str!("registry.rs");
    let changes_since = source
        .split("pub fn changes_since")
        .nth(1)
        .and_then(|body| body.split("fn snapshot_with_effects").next())
        .expect("dirty delta implementation should remain available");

    assert!(changes_since.contains("let cursor_generation = cursor.map"));
    assert!(changes_since.contains("let transaction_cursor = cursor.map"));
    assert!(!changes_since.contains("cursor checked above"));
    assert!(!changes_since.contains(".expect("));
}

#[test]
fn cursor_from_another_registry_is_rejected() {
    let (_first_transactions, first) = registry();
    let (_second_transactions, second) = registry();
    let foreign = first.changes_since(None).unwrap();

    assert!(matches!(
        second.changes_since(Some(foreign.cursor())),
        Err(DirtyRegistryError::CursorRegistryMismatch)
    ));
}

struct BlockingDirtySource {
    transactions: Arc<EditorTransactionEngine>,
    first_started: Mutex<Option<mpsc::Sender<()>>>,
    first_release: Mutex<Option<mpsc::Receiver<()>>>,
}

impl BlockingDirtySource {
    fn arm(&self, started: mpsc::Sender<()>, release: mpsc::Receiver<()>) {
        *self
            .first_started
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(started);
        *self
            .first_release
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(release);
    }

    fn block_once(&self) {
        let release = self
            .first_release
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(started) = self
            .first_started
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = started.send(());
        }
        if let Some(release) = release {
            release.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    }
}

impl DirtyTransactionStateSource for BlockingDirtySource {
    fn is_dirty(&self, document: DocumentId) -> Result<bool, EditCommandError> {
        self.block_once();
        self.transactions
            .is_dirty(HistoryContextId::Document(document))
    }

    fn dirty_states_since(
        &self,
        cursor: Option<&HistoryDirtyCursor>,
    ) -> Result<HistoryDirtyBatch, EditCommandError> {
        self.block_once();
        self.transactions.dirty_states_since(cursor)
    }

    fn capture_save_token(
        &self,
        document: DocumentId,
    ) -> Result<crate::core::editing::engine::HistorySaveToken, EditCommandError> {
        self.transactions
            .capture_save_token(HistoryContextId::Document(document))
    }

    fn mark_saved_if_unchanged(
        &self,
        document: DocumentId,
        token: crate::core::editing::engine::HistorySaveToken,
    ) -> Result<(), EditCommandError> {
        self.transactions
            .mark_saved_if_unchanged(HistoryContextId::Document(document), token)
            .map(drop)
    }
}

#[test]
fn snapshot_retries_external_generation_change_instead_of_returning_false_clean() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let transactions = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let source = Arc::new(BlockingDirtySource {
        transactions,
        first_started: Mutex::new(None),
        first_release: Mutex::new(None),
    });
    let registry = DirtyRegistry::with_transaction_source(source.clone());
    let document = document(14);
    registry.register_document(document).unwrap();
    source.arm(started_tx, release_rx);
    let worker_registry = registry.clone();
    let worker = thread::spawn(move || worker_registry.snapshot(document).unwrap());
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();

    registry
        .mark_external_effect(document, effect("ui.source_buffer"))
        .unwrap();
    release_tx.send(()).unwrap();

    let snapshot = worker.join().unwrap();
    assert!(!snapshot.transaction_dirty());
    assert_eq!(snapshot.external_effects(), &[effect("ui.source_buffer")]);
    assert!(snapshot.is_dirty());
}

#[test]
fn dirty_delta_retries_only_changed_documents_when_external_generation_moves() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let transactions = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let source = Arc::new(BlockingDirtySource {
        transactions,
        first_started: Mutex::new(None),
        first_release: Mutex::new(None),
    });
    let registry = DirtyRegistry::with_transaction_source(source.clone());
    let first = document(21);
    let second = document(22);
    registry.register_document(first).unwrap();
    registry.register_document(second).unwrap();
    let baseline = registry.changes_since(None).unwrap();
    source.arm(started_tx, release_rx);
    let worker_registry = registry.clone();
    let worker = thread::spawn(move || {
        worker_registry
            .changes_since(Some(baseline.cursor()))
            .unwrap()
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();

    registry
        .mark_external_effect(second, effect("ui.source_buffer"))
        .unwrap();
    release_tx.send(()).unwrap();

    let delta = worker.join().unwrap();
    assert!(!delta.is_reset());
    assert_eq!(delta.snapshots().len(), 1);
    assert_eq!(delta.snapshots()[0].document(), second);
    assert_eq!(
        delta.snapshots()[0].external_effects(),
        &[effect("ui.source_buffer")]
    );
    assert!(delta.snapshots()[0].is_dirty());
}

#[test]
fn dirty_snapshot_stores_each_external_effect_id_once() {
    let source = include_str!("registry.rs");
    let snapshot_start = source.find("pub struct DirtyDocumentSnapshot").unwrap();
    let snapshot_end = source[snapshot_start..]
        .find("impl DirtyDocumentSnapshot")
        .map(|offset| snapshot_start + offset)
        .unwrap();
    let snapshot_source = &source[snapshot_start..snapshot_end];

    assert!(snapshot_source.contains("external_revisions: Vec<DirtyExternalEffectRevision>"));
    assert!(!snapshot_source.contains(
        "external_revisions: BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision>"
    ));
    assert!(source.contains("binary_search(effect)"));
}
