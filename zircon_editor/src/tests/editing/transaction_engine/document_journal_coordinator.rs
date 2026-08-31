use std::any::Any;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

use crate::core::editing::engine::{
    CommandExecutionError, CommandJournalPayload, EditCommand, EditContext,
    EditorTransactionEngine, HistoryContextId, JournalDocumentKey, MergeOutcome,
};
use crate::core::editor_message::DocumentId;
use crate::core::recovery::{DocumentJournalCoordinator, DocumentJournalCoordinatorError};

use super::fixture::FixtureContext;

#[test]
fn document_journal_coordinator_appends_and_compacts_one_bound_document() {
    let directory = TestDirectory::new();
    let coordinator = DocumentJournalCoordinator::new(directory.path());
    let document = DocumentId::new(91);
    coordinator
        .bind_document(document, Path::new("assets/scenes/main.zscene"))
        .unwrap();
    let engine = EditorTransactionEngine::new(FixtureContext::default());

    let first = committed_transaction(&engine, document, "first durable edit");
    let second = committed_transaction(&engine, document, "second durable edit");
    assert_eq!(
        coordinator
            .append_for_test(&engine, document, first)
            .unwrap()
            .sequence(),
        1
    );
    assert_eq!(
        coordinator
            .append_for_test(&engine, document, second)
            .unwrap()
            .sequence(),
        2
    );

    let compacted = coordinator.compact_covered_prefix(document, 1).unwrap();
    assert_eq!(compacted.covered_through(), 1);
    assert_eq!(compacted.discarded_entries(), 1);
    let report = coordinator.read_document(document).unwrap();
    assert_eq!(report.base_sequence(), 1);
    assert_eq!(report.entries().len(), 1);
    assert_eq!(report.entries()[0].sequence(), 2);

    let third = committed_transaction(&engine, document, "post-compaction durable edit");
    assert_eq!(
        coordinator
            .append_for_test(&engine, document, third)
            .unwrap()
            .sequence(),
        3
    );
    let report = coordinator.read_document(document).unwrap();
    assert_eq!(
        report
            .entries()
            .iter()
            .map(|entry| entry.sequence())
            .collect::<Vec<_>>(),
        vec![2, 3]
    );

    let key =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/scenes/main.zscene"))
            .unwrap();
    assert!(coordinator.journal_path(document).unwrap().ends_with(
        Path::new(".zircon/journal")
            .join(key.as_str())
            .join("transactions.zjr")
    ));
}

#[test]
fn document_journal_coordinator_rejects_rebinding_and_append_after_unbind() {
    let directory = TestDirectory::new();
    let coordinator = DocumentJournalCoordinator::new(directory.path());
    let document = DocumentId::new(92);
    coordinator
        .bind_document(document, Path::new("assets/scenes/main.zscene"))
        .unwrap();
    assert!(matches!(
        coordinator.bind_document(document, Path::new("assets/scenes/other.zscene")),
        Err(DocumentJournalCoordinatorError::BindingConflict { .. })
    ));
    assert!(coordinator.unbind_document(document));

    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let transaction = committed_transaction(&engine, document, "unbound durable edit");
    assert!(matches!(
        coordinator.append_for_test(&engine, document, transaction),
        Err(DocumentJournalCoordinatorError::DocumentNotBound { .. })
    ));
}

#[test]
fn document_journal_coordinator_derives_document_keys_only_from_its_own_project_root() {
    let directory = TestDirectory::new();
    let coordinator = DocumentJournalCoordinator::new(directory.path());
    let document = DocumentId::new(93);
    let source = directory.path().join("assets/scenes/main.zscene");

    coordinator
        .bind_project_document(document, &source)
        .unwrap();
    let expected =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/scenes/main.zscene"))
            .unwrap();
    assert!(coordinator
        .journal_path(document)
        .unwrap()
        .ends_with(expected.as_str()));

    let foreign_source = directory
        .path()
        .parent()
        .expect("test directory has an artifact parent")
        .join("foreign.scene.toml");
    assert!(matches!(
        coordinator.bind_project_document(DocumentId::new(94), &foreign_source),
        Err(DocumentJournalCoordinatorError::SourceOutsideProject { .. })
    ));
}

#[test]
fn document_journal_coordinator_admits_materialization_through_the_document_append_gate() {
    let source = include_str!("../../../core/recovery/document_journal/coordinator.rs");
    let append = source
        .split("pub(crate) fn append_for_test")
        .nth(1)
        .and_then(|remaining| remaining.split("/// Drops the active file handle").next())
        .expect("test append owner body");
    let gate = append
        .find("let _append_guard = lock(&bound.append_gate);")
        .expect("append must acquire its document gate");
    let materialization = append
        .find(".journal_transaction(HistoryContextId::Document(document), transaction)")
        .expect("append must materialize the committed engine transaction");

    assert!(
        gate < materialization,
        "the ordered document gate must admit engine materialization before any later append can overtake it"
    );
    assert!(
        !source.contains("append_from_commit_callback")
            && !source.contains("pub fn append_committed"),
        "the unusable production append API must not survive the hard cut"
    );

    let unbind = source
        .split("pub fn unbind_document")
        .nth(1)
        .and_then(|remaining| remaining.split("pub fn journal_path").next())
        .expect("unbind_document owner body");
    let unbind_gate = unbind
        .find("let _append_guard = lock(&bound.append_gate);")
        .expect("unbind must acquire the document gate");
    let deactivate = unbind
        .find("slot.active = false;")
        .expect("unbind must deactivate the document slot");
    let remove_binding = unbind
        .find("bindings.remove(&document);")
        .expect("unbind must remove the document binding");

    assert!(
        unbind_gate < deactivate && deactivate < remove_binding,
        "a DocumentId cannot be rebound until its prior durable writer is inactive"
    );
}

fn committed_transaction(
    engine: &EditorTransactionEngine,
    document: DocumentId,
    label: &str,
) -> crate::core::editing::engine::TransactionId {
    let mut scope = engine
        .begin(label, HistoryContextId::Document(document))
        .unwrap();
    scope.push(PersistedJournalCommand).unwrap();
    scope.commit().unwrap()
}

struct PersistedJournalCommand;

impl EditCommand for PersistedJournalCommand {
    fn label(&self) -> &str {
        "persisted document journal command"
    }

    fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn try_merge(&mut self, _next: &dyn EditCommand) -> MergeOutcome {
        MergeOutcome::Reject
    }

    fn journal_payload(
        &self,
    ) -> Result<CommandJournalPayload, crate::core::editing::engine::CommandJournalUnavailable>
    {
        Ok(CommandJournalPayload::new(
            "test.document_journal",
            1,
            json!({}),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

        let ordinal = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let executable = std::env::current_exe().unwrap();
        let artifact_root = executable
            .parent()
            .expect("test executable has a target-directory parent")
            .join("zircon-editor-test-artifacts");
        let path = artifact_root.join(format!(
            "document-journal-coordinator-{}-{ordinal}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
