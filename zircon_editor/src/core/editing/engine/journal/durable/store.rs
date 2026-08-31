use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::core::resource::io::atomic_write;

use super::{
    DurableJournalError, JournalCompactionReport, JournalDocumentKey, JournalWriter,
    PreparedJournalRecord, MAX_JOURNAL_BYTES,
};

pub(super) const JOURNAL_MAGIC: [u8; 8] = *b"ZRJNL001";
pub(super) const JOURNAL_FORMAT_VERSION: u16 = 2;

pub struct DurableJournal {
    project_root: PathBuf,
}

impl DurableJournal {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub fn path_for(&self, document: &JournalDocumentKey) -> PathBuf {
        self.journal_root()
            .join(document.as_str())
            .join("transactions.zjr")
    }

    pub fn open(
        &self,
        document: &JournalDocumentKey,
    ) -> Result<JournalWriter, DurableJournalError> {
        let path = self.path_for(document);
        let parent = path
            .parent()
            .ok_or_else(|| DurableJournalError::InvalidRoot { path: path.clone() })?;
        fs::create_dir_all(parent).map_err(|source| DurableJournalError::Io {
            operation: "create journal directory",
            path: parent.to_path_buf(),
            source,
        })?;
        if !path.exists() {
            return JournalWriter::create(path, document);
        }

        let mut report = self.read(document)?;
        if let Some(tail) = report.take_tail_fault() {
            return Err(DurableJournalError::UnreadableTail { path, tail });
        }
        let next_sequence = match report.entries().last() {
            Some(entry) => entry
                .sequence()
                .checked_add(1)
                .ok_or_else(|| DurableJournalError::SequenceExhausted { path: path.clone() })?,
            None => report
                .base_sequence()
                .checked_add(1)
                .ok_or_else(|| DurableJournalError::SequenceExhausted { path: path.clone() })?,
        };
        JournalWriter::open_existing(path, next_sequence)
    }

    pub const fn maximum_bytes() -> u64 {
        MAX_JOURNAL_BYTES
    }

    pub(super) fn journal_root(&self) -> PathBuf {
        self.project_root.join(".zircon").join("journal")
    }

    /// Rewrites only records newer than a snapshot that durably covers `covered_through`.
    pub fn compact_covered_prefix(
        &self,
        document: &JournalDocumentKey,
        covered_through: u64,
    ) -> Result<JournalCompactionReport, DurableJournalError> {
        let path = self.path_for(document);
        let mut report = self.read(document)?;
        if let Some(tail) = report.take_tail_fault() {
            return Err(DurableJournalError::UnreadableTail { path, tail });
        }
        let base_sequence = report.base_sequence();
        let last_sequence = report
            .entries()
            .last()
            .map_or(base_sequence, |entry| entry.sequence());
        if covered_through < base_sequence || covered_through > last_sequence {
            return Err(DurableJournalError::CheckpointOutOfRange {
                path,
                requested: covered_through,
                base_sequence,
                last_sequence,
            });
        }
        if covered_through == base_sequence {
            return Ok(JournalCompactionReport {
                covered_through,
                discarded_entries: 0,
                retained_entries: report.entries().len(),
            });
        }

        let retained = report
            .entries()
            .iter()
            .filter(|entry| entry.sequence() > covered_through)
            .collect::<Vec<_>>();
        let temporary = compaction_path(&path)?;
        let result = (|| {
            let mut writer =
                JournalWriter::create_with_base(temporary.clone(), document, covered_through)?;
            for entry in &retained {
                let prepared =
                    PreparedJournalRecord::prepare(entry.transaction()).map_err(|source| {
                        DurableJournalError::Preparation {
                            path: temporary.clone(),
                            source,
                        }
                    })?;
                let sequence = writer.append_prepared(prepared)?;
                if sequence != entry.sequence() {
                    return Err(DurableJournalError::CompactionSequenceMismatch {
                        path: temporary.clone(),
                        expected: entry.sequence(),
                    });
                }
            }
            drop(writer);
            replace_compacted_journal(&temporary, &path)?;
            Ok(JournalCompactionReport {
                covered_through,
                discarded_entries: report.entries().len() - retained.len(),
                retained_entries: retained.len(),
            })
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

/// Publishes the already-synced compaction file through the shared cross-platform file contract.
///
/// Compaction is a bounded maintenance path, while normal transaction appends stay streaming.
/// Reading this writer-owned temporary file lets the runtime own `ReplaceFileW` versus rename
/// semantics without duplicating platform FFI in the editor.
fn replace_compacted_journal(
    temporary: &std::path::Path,
    path: &std::path::Path,
) -> Result<(), DurableJournalError> {
    let bytes = fs::read(temporary).map_err(|source| DurableJournalError::Io {
        operation: "read compacted durable journal for atomic replacement",
        path: temporary.to_path_buf(),
        source,
    })?;
    atomic_write(path, &bytes).map_err(|source| DurableJournalError::Io {
        operation: "atomically replace compacted durable journal",
        path: path.to_path_buf(),
        source,
    })?;
    fs::remove_file(temporary).map_err(|source| DurableJournalError::Io {
        operation: "remove published durable journal compaction staging",
        path: temporary.to_path_buf(),
        source,
    })
}

fn compaction_path(path: &std::path::Path) -> Result<PathBuf, DurableJournalError> {
    static NEXT_COMPACTION: AtomicU64 = AtomicU64::new(0);

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| DurableJournalError::InvalidRoot {
            path: path.to_path_buf(),
        })?;
    let nonce = NEXT_COMPACTION.fetch_add(1, Ordering::Relaxed);
    Ok(path.with_file_name(format!(
        ".{file_name}.compact-{}-{nonce}",
        std::process::id()
    )))
}
