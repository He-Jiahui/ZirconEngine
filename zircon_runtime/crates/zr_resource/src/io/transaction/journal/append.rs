//! Append-only transition records and durable commit-point publication.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::super::error::{DurableTransactionError, TransactionPhase};
use super::super::schema::{JournalPhase, JournalState, JournalTransition};
use super::super::stage::StagedFile;
use super::frame_codec::{MAX_JOURNAL_BYTES, transition_frame};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::io::transaction) enum CommitPointRecord {
    Durable,
    /// The complete commit frame is visible, but its durability barrier failed.
    PublishedWithoutSync,
}

pub(in crate::io::transaction) fn record_prepared(
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
            retired_digests: staged.retired_digests.clone(),
        },
        TransactionPhase::Stage,
    )
}

pub(in crate::io::transaction) fn record_state(
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
            retired_digests: Vec::new(),
        },
        if state == JournalState::RollingBack {
            TransactionPhase::Rollback
        } else {
            TransactionPhase::Commit
        },
    )
}

pub(in crate::io::transaction) fn record_phase(
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
            retired_digests: Vec::new(),
        },
        if phase == JournalPhase::CleanupIntent {
            TransactionPhase::Stage
        } else if matches!(
            phase,
            JournalPhase::RollbackCompleted | JournalPhase::CleanupRollback
        ) {
            TransactionPhase::Rollback
        } else {
            TransactionPhase::Commit
        },
    )
}

pub(in crate::io::transaction) fn record_commit_point(
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
            retired_digests: Vec::new(),
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
