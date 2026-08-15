//! Fsync'd immutable intent plus append-only transitions.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::{DurableTransactionError, TransactionPhase};
use super::pathing::{journal_path, transaction_sibling};
use super::schema::{
    JournalIntent, JournalPhase, JournalState, JournalTransition, TransactionJournal,
    JOURNAL_VERSION,
};
use super::stage::StagedFile;
use super::PreparedFileWrite;
use crate::core::resource::io::{atomic_write, sync_parent_directory};

const FRAME_HEADER_BYTES: usize = 8 + blake3::OUT_LEN;
pub(super) const MAX_JOURNAL_BYTES: usize = 128 * 1024 * 1024;
const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CommitPointRecord {
    Durable,
    /// The complete commit frame is visible, but its durability barrier failed.
    PublishedWithoutSync,
}

pub(super) fn create_intent(
    directory: &Path,
    tag: &str,
    transaction_id: &str,
    writes: &[PreparedFileWrite],
) -> Result<(PathBuf, Vec<JournalIntent>), DurableTransactionError> {
    let first = writes.first().ok_or_else(|| {
        DurableTransactionError::operation(
            TransactionPhase::Stage,
            PathBuf::from("<empty>"),
            io::Error::new(io::ErrorKind::InvalidInput, "empty file transaction"),
        )
    })?;
    let intents = writes
        .iter()
        .map(|write| intent_for_write(write, tag, transaction_id))
        .collect::<Vec<_>>();
    let path = journal_path(directory, &first.path, tag, transaction_id);
    persist_intent(
        &path,
        &TransactionJournal {
            version: JOURNAL_VERSION,
            tag: tag.to_owned(),
            transaction_id: transaction_id.to_owned(),
            documents: intents
                .iter()
                .map(|intent| JournalIntent {
                    target: intent.target.clone(),
                    staging: intent.staging.clone(),
                    backup: intent.backup.clone(),
                    rollback_staging: intent.rollback_staging.clone(),
                    retired_path: intent.retired_path.clone(),
                    retired_backup: intent.retired_backup.clone(),
                    retired_rollback_staging: intent.retired_rollback_staging.clone(),
                })
                .collect(),
            transitions: Vec::new(),
        },
    )?;
    Ok((path, intents))
}

fn intent_for_write(write: &PreparedFileWrite, tag: &str, transaction_id: &str) -> JournalIntent {
    JournalIntent {
        target: write.path.clone(),
        staging: transaction_sibling(&write.path, tag, "stage", transaction_id),
        backup: transaction_sibling(&write.path, tag, "backup", transaction_id),
        rollback_staging: transaction_sibling(&write.path, tag, "rollback-stage", transaction_id),
        retired_path: write.retired_path.clone(),
        retired_backup: write
            .retired_path
            .as_ref()
            .map(|path| transaction_sibling(path, tag, "retired-backup", transaction_id)),
        retired_rollback_staging: write
            .retired_path
            .as_ref()
            .map(|path| transaction_sibling(path, tag, "retired-rollback-stage", transaction_id)),
    }
}

pub(super) fn record_prepared(
    path: &Path,
    index: usize,
    staged: &StagedFile,
) -> Result<(), DurableTransactionError> {
    append_transition(
        path,
        JournalTransition {
            phase: JournalPhase::Intent,
            document_index: Some(index),
            state: Some(JournalState::Prepared),
            target_existed: Some(staged.target_existed),
            original_digest: staged.original_digest.clone(),
            new_digest: Some(staged.new_digest.clone()),
            retired_digest: staged.retired_digest.clone(),
        },
        TransactionPhase::Stage,
    )
}

pub(super) fn record_state(
    path: &Path,
    index: usize,
    state: JournalState,
) -> Result<(), DurableTransactionError> {
    append_transition(
        path,
        JournalTransition {
            phase: JournalPhase::Active,
            document_index: Some(index),
            state: Some(state),
            target_existed: None,
            original_digest: None,
            new_digest: None,
            retired_digest: None,
        },
        if state == JournalState::RollingBack {
            TransactionPhase::Rollback
        } else {
            TransactionPhase::Commit
        },
    )
}

pub(super) fn record_phase(
    path: &Path,
    phase: JournalPhase,
) -> Result<(), DurableTransactionError> {
    append_transition(
        path,
        JournalTransition {
            phase,
            document_index: None,
            state: None,
            target_existed: None,
            original_digest: None,
            new_digest: None,
            retired_digest: None,
        },
        if matches!(
            phase,
            JournalPhase::RollbackCompleted | JournalPhase::CleanupRollback
        ) {
            TransactionPhase::Rollback
        } else {
            TransactionPhase::Commit
        },
    )
}

pub(super) fn record_commit_point(
    path: &Path,
) -> Result<CommitPointRecord, DurableTransactionError> {
    let frame = transition_frame(
        path,
        JournalTransition {
            phase: JournalPhase::AllCommitted,
            document_index: None,
            state: None,
            target_existed: None,
            original_digest: None,
            new_digest: None,
            retired_digest: None,
        },
        TransactionPhase::Commit,
    )?;
    let mut file = open_bounded_append(path, frame.len(), TransactionPhase::Commit)?;
    file.write_all(&frame).map_err(|source| {
        DurableTransactionError::operation(TransactionPhase::Commit, path, source)
    })?;
    Ok(match file.sync_all() {
        Ok(()) => CommitPointRecord::Durable,
        Err(_) => CommitPointRecord::PublishedWithoutSync,
    })
}

fn persist_intent(
    path: &Path,
    journal: &TransactionJournal,
) -> Result<(), DurableTransactionError> {
    let bytes = toml::to_string_pretty(journal).map_err(|error| {
        DurableTransactionError::operation(
            TransactionPhase::Stage,
            path,
            io::Error::new(io::ErrorKind::InvalidData, error.to_string()),
        )
    })?;
    let frame = encode_frame(bytes.as_bytes())?;
    atomic_write(path, &frame)
        .and_then(|()| {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)?
                .sync_all()
        })
        .and_then(|()| sync_parent_directory(path))
        .map_err(|source| DurableTransactionError::operation(TransactionPhase::Stage, path, source))
}

fn append_transition(
    path: &Path,
    transition: JournalTransition,
    phase: TransactionPhase,
) -> Result<(), DurableTransactionError> {
    let frame = transition_frame(path, transition, phase)?;
    let mut file = open_bounded_append(path, frame.len(), phase)?;
    file.write_all(&frame)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|source| DurableTransactionError::operation(phase, path, source))
}

fn open_bounded_append(
    path: &Path,
    frame_len: usize,
    phase: TransactionPhase,
) -> Result<fs::File, DurableTransactionError> {
    let file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|source| DurableTransactionError::operation(phase, path, source))?;
    let current_len = file
        .metadata()
        .map_err(|source| DurableTransactionError::operation(phase, path, source))?
        .len();
    let frame_len = u64::try_from(frame_len).map_err(|_| {
        DurableTransactionError::operation(
            phase,
            path,
            io::Error::new(
                io::ErrorKind::InvalidData,
                "journal frame length exceeds this platform",
            ),
        )
    })?;
    let bounded_len = MAX_JOURNAL_BYTES as u64;
    if current_len
        .checked_add(frame_len)
        .is_none_or(|next_len| next_len > bounded_len)
    {
        return Err(DurableTransactionError::operation(
            phase,
            path,
            io::Error::new(
                io::ErrorKind::InvalidData,
                "durable transaction journal would exceed its bounded size",
            ),
        ));
    }
    Ok(file)
}

fn transition_frame(
    path: &Path,
    transition: JournalTransition,
    phase: TransactionPhase,
) -> Result<Vec<u8>, DurableTransactionError> {
    let bytes = toml::to_string_pretty(&TransitionAppend {
        transitions: vec![transition],
    })
    .map_err(|error| {
        DurableTransactionError::operation(
            phase,
            path,
            io::Error::new(io::ErrorKind::InvalidData, error.to_string()),
        )
    })?;
    encode_frame(bytes.as_bytes())
}

#[cfg(test)]
pub(super) fn decode_journal(
    path: &Path,
    bytes: &[u8],
) -> Result<TransactionJournal, DurableTransactionError> {
    decode_journal_with_valid_len(path, bytes).map(|(journal, _)| journal)
}

pub(super) fn decode_journal_with_valid_len(
    path: &Path,
    bytes: &[u8],
) -> Result<(TransactionJournal, usize), DurableTransactionError> {
    if bytes.len() > MAX_JOURNAL_BYTES {
        return Err(DurableTransactionError::invalid(
            path,
            "durable transaction journal exceeds its bounded size",
        ));
    }
    let mut offset = 0;
    let intent = decode_frame(path, bytes, &mut offset, true)?.ok_or_else(|| {
        DurableTransactionError::invalid(path, "transaction journal has no complete intent frame")
    })?;
    let mut journal = parse_toml_frame::<TransactionJournal>(path, intent)?;
    if !journal.transitions.is_empty() {
        return Err(DurableTransactionError::invalid(
            path,
            "immutable intent frame contains state transitions",
        ));
    }
    while let Some(frame) = decode_frame(path, bytes, &mut offset, false)? {
        let append = parse_toml_frame::<TransitionAppend>(path, frame)?;
        if append.transitions.len() != 1 {
            return Err(DurableTransactionError::invalid(
                path,
                "journal frame must contain exactly one transition",
            ));
        }
        journal.transitions.extend(append.transitions);
    }
    Ok((journal, offset))
}

pub(super) fn truncate_torn_tail(
    path: &Path,
    valid_len: usize,
) -> Result<(), DurableTransactionError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Recovery, path, source)
        })?;
    let current_len = file
        .metadata()
        .map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Recovery, path, source)
        })?
        .len();
    let valid_len = u64::try_from(valid_len).map_err(|_| {
        DurableTransactionError::operation(
            TransactionPhase::Recovery,
            path,
            io::Error::new(
                io::ErrorKind::InvalidData,
                "durable journal prefix length exceeds this platform",
            ),
        )
    })?;
    if current_len < valid_len {
        return Err(DurableTransactionError::invalid(
            path,
            "durable journal became shorter after validation",
        ));
    }
    if current_len == valid_len {
        return Ok(());
    }
    file.set_len(valid_len)
        .and_then(|()| file.sync_all())
        .map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Recovery, path, source)
        })
}

fn encode_frame(payload: &[u8]) -> Result<Vec<u8>, DurableTransactionError> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(DurableTransactionError::operation(
            TransactionPhase::Stage,
            PathBuf::from("<journal-frame>"),
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "durable transaction journal frame exceeds its bounded size",
            ),
        ));
    }
    let mut frame = Vec::with_capacity(FRAME_HEADER_BYTES + payload.len());
    frame.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    frame.extend_from_slice(blake3::hash(payload).as_bytes());
    frame.extend_from_slice(payload);
    Ok(frame)
}

fn decode_frame<'a>(
    path: &Path,
    bytes: &'a [u8],
    offset: &mut usize,
    intent: bool,
) -> Result<Option<&'a [u8]>, DurableTransactionError> {
    let remaining = &bytes[*offset..];
    if remaining.is_empty() {
        return Ok(None);
    }
    if remaining.len() < FRAME_HEADER_BYTES {
        return if intent {
            Err(DurableTransactionError::invalid(
                path,
                "immutable intent frame header is incomplete",
            ))
        } else {
            Ok(None)
        };
    }
    let mut length_bytes = [0_u8; 8];
    length_bytes.copy_from_slice(&remaining[..8]);
    let length = u64::from_le_bytes(length_bytes);
    let length = usize::try_from(length).map_err(|_| {
        DurableTransactionError::invalid(path, "journal frame length exceeds this platform")
    })?;
    if length > MAX_FRAME_BYTES {
        return Err(DurableTransactionError::invalid(
            path,
            "journal frame exceeds its bounded size",
        ));
    }
    let end = FRAME_HEADER_BYTES
        .checked_add(length)
        .ok_or_else(|| DurableTransactionError::invalid(path, "journal frame length overflows"))?;
    if remaining.len() < end {
        return if intent {
            Err(DurableTransactionError::invalid(
                path,
                "immutable intent frame payload is incomplete",
            ))
        } else {
            Ok(None)
        };
    }
    let payload = &remaining[FRAME_HEADER_BYTES..end];
    let expected = &remaining[8..FRAME_HEADER_BYTES];
    if blake3::hash(payload).as_bytes() != expected {
        if !intent && remaining.len() == end {
            return Ok(None);
        }
        return Err(DurableTransactionError::invalid(
            path,
            "journal frame checksum is invalid",
        ));
    }
    *offset += end;
    Ok(Some(payload))
}

fn parse_toml_frame<T: for<'de> Deserialize<'de>>(
    path: &Path,
    bytes: &[u8],
) -> Result<T, DurableTransactionError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        DurableTransactionError::invalid(path, format!("journal frame is not UTF-8: {error}"))
    })?;
    toml::from_str(source).map_err(|source| DurableTransactionError::JournalDeserialize {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Deserialize, Serialize)]
struct TransitionAppend {
    transitions: Vec<JournalTransition>,
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn every_torn_transition_tail_folds_to_the_previous_durable_state() {
        let intent = intent_frame();
        let transition = encode_frame(
            toml::to_string_pretty(&prepared_transition())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();

        for cut in 0..transition.len() {
            let mut bytes = intent.clone();
            bytes.extend_from_slice(&transition[..cut]);
            let journal = decode_journal(Path::new("journal.zrjournal"), &bytes).unwrap();
            assert!(
                journal.transitions.is_empty(),
                "a {cut}-byte torn frame must not publish its transition"
            );
        }

        let mut complete = intent;
        complete.extend_from_slice(&transition);
        let journal = decode_journal(Path::new("journal.zrjournal"), &complete).unwrap();
        assert_eq!(journal.transitions.len(), 1);
    }

    #[test]
    fn final_checksum_failure_is_a_torn_transition_but_intent_corruption_is_fatal() {
        let intent = intent_frame();
        let mut transition = encode_frame(
            toml::to_string_pretty(&prepared_transition())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        *transition.last_mut().unwrap() ^= 0xff;
        let mut bytes = intent.clone();
        bytes.extend_from_slice(&transition);

        let journal = decode_journal(Path::new("journal.zrjournal"), &bytes).unwrap();
        assert!(journal.transitions.is_empty());

        let mut corrupt_intent = intent;
        *corrupt_intent.last_mut().unwrap() ^= 0xff;
        assert!(decode_journal(Path::new("journal.zrjournal"), &corrupt_intent).is_err());
    }

    #[test]
    fn torn_transition_tail_is_truncated_before_recovery_appends() {
        let root = std::env::temp_dir().join(format!(
            "zircon-durable-journal-tail-{}-{}",
            std::process::id(),
            crate::core::resource::io::NEXT_ATOMIC_FILE_ID
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let path = root.join("project.zrjournal");
        fs::create_dir_all(&root).unwrap();
        let mut bytes = intent_frame();
        let prepared = encode_frame(
            toml::to_string_pretty(&prepared_transition())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        bytes.extend_from_slice(&prepared);
        let durable_len = bytes.len();
        let active = transition_frame(
            &path,
            JournalTransition {
                phase: JournalPhase::Active,
                document_index: None,
                state: None,
                target_existed: None,
                original_digest: None,
                new_digest: None,
                retired_digest: None,
            },
            TransactionPhase::Recovery,
        )
        .unwrap();
        bytes.extend_from_slice(&active[..FRAME_HEADER_BYTES - 1]);
        fs::write(&path, &bytes).unwrap();

        let (_, decoded_len) = decode_journal_with_valid_len(&path, &bytes).unwrap();
        assert_eq!(decoded_len, durable_len);
        truncate_torn_tail(&path, decoded_len).unwrap();
        record_phase(&path, JournalPhase::Active).unwrap();

        let decoded = decode_journal(&path, &fs::read(&path).unwrap()).unwrap();
        assert_eq!(decoded.transitions.len(), 2);
        assert_eq!(decoded.fold().unwrap().phase, JournalPhase::Active);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn transition_and_commit_point_appends_cannot_exceed_the_journal_bound() {
        let root = std::env::temp_dir().join(format!(
            "zircon-durable-journal-bound-{}-{}",
            std::process::id(),
            crate::core::resource::io::NEXT_ATOMIC_FILE_ID
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let path = root.join("project.zrjournal");
        fs::create_dir_all(&root).unwrap();
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_JOURNAL_BYTES as u64).unwrap();
        drop(file);

        let transition_error = record_phase(&path, JournalPhase::Active)
            .expect_err("a transition must not grow an already-full journal");
        let commit_point_error = record_commit_point(&path)
            .expect_err("the commit point must not grow an already-full journal");

        assert!(transition_error.to_string().contains("bounded size"));
        assert!(commit_point_error.to_string().contains("bounded size"));
        assert_eq!(fs::metadata(&path).unwrap().len(), MAX_JOURNAL_BYTES as u64);
        fs::remove_dir_all(root).unwrap();
    }

    fn intent_frame() -> Vec<u8> {
        let journal = TransactionJournal {
            version: JOURNAL_VERSION,
            tag: "project".to_owned(),
            transaction_id: "1-1".to_owned(),
            documents: vec![JournalIntent {
                target: PathBuf::from("C:/project/.zircon/registry/asset-registry.json"),
                staging: PathBuf::from(
                    "C:/project/.zircon/registry/.registry.zr-project-stage-1-1",
                ),
                backup: PathBuf::from(
                    "C:/project/.zircon/registry/.registry.zr-project-backup-1-1",
                ),
                rollback_staging: PathBuf::from(
                    "C:/project/.zircon/registry/.registry.zr-project-rollback-stage-1-1",
                ),
                retired_path: None,
                retired_backup: None,
                retired_rollback_staging: None,
            }],
            transitions: Vec::new(),
        };
        encode_frame(toml::to_string_pretty(&journal).unwrap().as_bytes()).unwrap()
    }

    fn prepared_transition() -> TransitionAppend {
        TransitionAppend {
            transitions: vec![JournalTransition {
                phase: JournalPhase::Intent,
                document_index: Some(0),
                state: Some(JournalState::Prepared),
                target_existed: Some(true),
                original_digest: Some("old".to_owned()),
                new_digest: Some("new".to_owned()),
                retired_digest: None,
            }],
        }
    }
}
