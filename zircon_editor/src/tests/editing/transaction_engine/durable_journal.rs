use std::any::Any;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use zircon_runtime_interface::serialization::{LoadError, VersionedSchema};

use crate::core::editing::engine::{
    CommandExecutionError, CommandJournalPayload, DurableJournal, DurableJournalError, EditCommand,
    EditContext, EditorTransactionEngine, HistoryContextId, JournalDiscoveryIssue,
    JournalDocumentKey, JournalTailFault, MergeOutcome, PreparedJournalRecord, TransactionJournal,
};

use super::fixture::FixtureContext;

#[test]
fn durable_journal_assigns_monotonic_sequences_and_restores_committed_entries() {
    let directory = TestDirectory::new();
    let store = DurableJournal::new(directory.path());
    let document =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/ui/panel.zui")).unwrap();
    let first = journal_for(1);
    let second = journal_for(2);

    let mut writer = store.open(&document).unwrap();
    assert_eq!(
        writer
            .append_prepared(PreparedJournalRecord::prepare(&first).unwrap())
            .unwrap(),
        1
    );
    assert_eq!(
        writer
            .append_prepared(PreparedJournalRecord::prepare(&second).unwrap())
            .unwrap(),
        2
    );
    assert_eq!(writer.next_sequence(), 3);
    drop(writer);

    let report = store.read(&document).unwrap();
    assert_eq!(report.entries().len(), 2);
    assert_eq!(report.entries()[0].sequence(), 1);
    assert_eq!(report.entries()[0].transaction(), &first);
    assert_eq!(report.entries()[1].sequence(), 2);
    assert_eq!(report.entries()[1].transaction(), &second);
    assert_eq!(report.tail_fault(), None);
}

#[test]
fn durable_journal_stops_at_a_truncated_tail_without_losing_the_valid_prefix() {
    let directory = TestDirectory::new();
    let store = DurableJournal::new(directory.path());
    let document =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/ui/panel.zui")).unwrap();
    let transaction = journal_for(1);

    let mut writer = store.open(&document).unwrap();
    writer
        .append_prepared(PreparedJournalRecord::prepare(&transaction).unwrap())
        .unwrap();
    drop(writer);
    let mut file = OpenOptions::new()
        .append(true)
        .open(store.path_for(&document))
        .unwrap();
    file.write_all(&[0x01]).unwrap();
    file.sync_data().unwrap();

    let report = store.read(&document).unwrap();
    assert_eq!(report.entries().len(), 1);
    assert_eq!(report.entries()[0].transaction(), &transaction);
    assert!(matches!(
        report.tail_fault(),
        Some(JournalTailFault::TruncatedFrame)
    ));
    assert!(matches!(
        store.open(&document),
        Err(
            crate::core::editing::engine::DurableJournalError::UnreadableTail {
                tail: JournalTailFault::TruncatedFrame,
                ..
            }
        )
    ));
}

#[test]
fn durable_journal_compaction_replaces_an_existing_target_on_every_platform() {
    let directory = TestDirectory::new();
    let store = DurableJournal::new(directory.path());
    let document =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/ui/panel.zui")).unwrap();
    let first = journal_for(1);
    let second = journal_for(2);
    let third = journal_for(3);

    let mut writer = store.open(&document).unwrap();
    writer
        .append_prepared(PreparedJournalRecord::prepare(&first).unwrap())
        .unwrap();
    writer
        .append_prepared(PreparedJournalRecord::prepare(&second).unwrap())
        .unwrap();
    writer
        .append_prepared(PreparedJournalRecord::prepare(&third).unwrap())
        .unwrap();
    drop(writer);

    let compacted = store.compact_covered_prefix(&document, 2).unwrap();
    assert_eq!(compacted.covered_through(), 2);
    assert_eq!(compacted.discarded_entries(), 2);
    assert_eq!(compacted.retained_entries(), 1);

    let report = store.read(&document).unwrap();
    assert_eq!(report.base_sequence(), 2);
    assert_eq!(report.entries().len(), 1);
    assert_eq!(report.entries()[0].sequence(), 3);
    assert_eq!(report.entries()[0].transaction(), &third);
    assert_eq!(report.tail_fault(), None);
}

#[test]
fn journal_document_key_rejects_absolute_and_escaping_source_paths() {
    assert!(JournalDocumentKey::from_project_relative_path(Path::new("../outside.zui")).is_err());
    assert!(JournalDocumentKey::from_project_relative_path(Path::new(r"C:\\outside.zui")).is_err());
    assert!(JournalDocumentKey::from_project_relative_path(Path::new("/outside.zui")).is_err());
}

#[test]
fn journal_document_key_normalizes_path_separators_independently_of_the_host() {
    let forward =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/ui/panel.zui")).unwrap();
    let backward =
        JournalDocumentKey::from_project_relative_path(Path::new(r"assets\ui\panel.zui")).unwrap();

    assert_eq!(forward, backward);
    assert_eq!(forward.source_path(), Path::new("assets/ui/panel.zui"));
}

#[test]
fn durable_journal_discovery_isolates_bad_directories_and_reports_recoverable_tails() {
    let directory = TestDirectory::new();
    let store = DurableJournal::new(directory.path());
    let healthy =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/scenes/healthy.zscene"))
            .unwrap();
    let truncated =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/scenes/truncated.zscene"))
            .unwrap();
    let transaction = journal_for(1);

    let mut healthy_writer = store.open(&healthy).unwrap();
    healthy_writer
        .append_prepared(PreparedJournalRecord::prepare(&transaction).unwrap())
        .unwrap();
    drop(healthy_writer);
    let mut truncated_writer = store.open(&truncated).unwrap();
    truncated_writer
        .append_prepared(PreparedJournalRecord::prepare(&transaction).unwrap())
        .unwrap();
    drop(truncated_writer);
    let mut truncated_file = OpenOptions::new()
        .append(true)
        .open(store.path_for(&truncated))
        .unwrap();
    truncated_file.write_all(&[0x01]).unwrap();
    truncated_file.sync_data().unwrap();
    fs::create_dir_all(directory.path().join(".zircon/journal/orphaned-entry")).unwrap();
    fs::create_dir_all(
        directory
            .path()
            .join(".zircon/journal/not-a-file/transactions.zjr"),
    )
    .unwrap();

    let discovery = store.discover().unwrap();
    assert_eq!(discovery.entries().len(), 2);
    let healthy_entry = discovery
        .entries()
        .iter()
        .find(|entry| entry.document() == &healthy)
        .unwrap();
    let truncated_entry = discovery
        .entries()
        .iter()
        .find(|entry| entry.document() == &truncated)
        .unwrap();
    assert_eq!(healthy_entry.report().tail_fault(), None);
    assert!(matches!(
        truncated_entry.report().tail_fault(),
        Some(JournalTailFault::TruncatedFrame)
    ));
    assert_eq!(discovery.issues().len(), 2);
    assert!(discovery
        .issues()
        .iter()
        .all(|issue| matches!(issue, JournalDiscoveryIssue::Journal { .. })));
    assert!(discovery.issues().iter().any(|issue| {
        matches!(
            issue.error(),
            Some(DurableJournalError::UnexpectedFileType { .. })
        )
    }));
}

#[test]
fn prepared_journal_record_owns_the_exact_validated_payload_and_digest() {
    let transaction = journal_for(9);

    let prepared = PreparedJournalRecord::prepare(&transaction).unwrap();

    assert_eq!(prepared.transaction(), transaction.transaction());
    assert_eq!(
        TransactionJournal::decode(prepared.payload()).unwrap(),
        transaction
    );
    assert_eq!(
        prepared.digest(),
        blake3::hash(prepared.payload()).as_bytes()
    );
}

#[test]
fn durable_journal_preserves_future_transaction_schema_as_the_tail_source() {
    let directory = TestDirectory::new();
    let store = DurableJournal::new(directory.path());
    let document =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/ui/future.zui")).unwrap();
    let mut future: serde_json::Value =
        serde_json::from_slice(&journal_for(1).encode().unwrap()).unwrap();
    future["$zircon"]["header"]["schema_version"] = json!(TransactionJournal::VERSION + 1);

    append_raw_record(&store, &document, &serde_json::to_vec(&future).unwrap());

    let report = store.read(&document).unwrap();
    assert!(matches!(
        report.tail_fault(),
        Some(JournalTailFault::InvalidTransaction {
            sequence: 1,
            source: crate::core::editing::engine::TransactionJournalReadError::Decode(
                LoadError::FutureVersion { .. }
            ),
        })
    ));
    assert!(matches!(
        store.open(&document),
        Err(DurableJournalError::UnreadableTail {
            tail: JournalTailFault::InvalidTransaction {
                sequence: 1,
                source: crate::core::editing::engine::TransactionJournalReadError::Decode(
                    LoadError::FutureVersion { .. }
                ),
            },
            ..
        })
    ));
    assert!(matches!(
        store.compact_covered_prefix(&document, 0),
        Err(DurableJournalError::UnreadableTail {
            tail: JournalTailFault::InvalidTransaction {
                sequence: 1,
                source: crate::core::editing::engine::TransactionJournalReadError::Decode(
                    LoadError::FutureVersion { .. }
                ),
            },
            ..
        })
    ));
}

#[test]
fn durable_journal_preserves_corrupt_transaction_payload_as_the_tail_source() {
    let directory = TestDirectory::new();
    let store = DurableJournal::new(directory.path());
    let document =
        JournalDocumentKey::from_project_relative_path(Path::new("assets/ui/corrupt.zui")).unwrap();
    let mut corrupt: serde_json::Value =
        serde_json::from_slice(&journal_for(1).encode().unwrap()).unwrap();
    corrupt["$zircon"]["payload"]["significant"] = serde_json::Value::Null;

    append_raw_record(&store, &document, &serde_json::to_vec(&corrupt).unwrap());

    let report = store.read(&document).unwrap();
    assert!(matches!(
        report.tail_fault(),
        Some(JournalTailFault::InvalidTransaction {
            sequence: 1,
            source: crate::core::editing::engine::TransactionJournalReadError::Decode(
                LoadError::PayloadDecode { .. }
            ),
        })
    ));
}

fn append_raw_record(store: &DurableJournal, document: &JournalDocumentKey, payload: &[u8]) {
    drop(store.open(document).unwrap());
    let mut file = OpenOptions::new()
        .append(true)
        .open(store.path_for(document))
        .unwrap();
    file.write_all(&1_u64.to_le_bytes()).unwrap();
    file.write_all(&(payload.len() as u32).to_le_bytes())
        .unwrap();
    file.write_all(blake3::hash(payload).as_bytes()).unwrap();
    file.write_all(payload).unwrap();
    file.sync_data().unwrap();
}

fn journal_for(delta: i32) -> TransactionJournal {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("durable journal edit", HistoryContextId::Global)
        .unwrap();
    scope.push(PersistedCommand { delta }).unwrap();
    let transaction = scope.commit().unwrap();
    engine
        .journal_transaction(HistoryContextId::Global, transaction)
        .unwrap()
}

struct PersistedCommand {
    delta: i32,
}

impl EditCommand for PersistedCommand {
    fn label(&self) -> &str {
        "persisted journal command"
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let fixture = context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .expect("durable journal fixture context");
        fixture.value += self.delta;
        Ok(())
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let fixture = context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .expect("durable journal fixture context");
        fixture.value -= self.delta;
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
            "test.persisted_journal",
            1,
            json!({ "delta": self.delta }),
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
        let path = artifact_root.join(format!("durable-journal-{}-{ordinal}", std::process::id()));
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
