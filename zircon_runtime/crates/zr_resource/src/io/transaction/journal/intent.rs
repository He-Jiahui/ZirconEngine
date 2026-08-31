//! Immutable WAL intent construction and durable first-frame publication.

use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

use super::super::error::{DurableTransactionError, TransactionPhase};
use super::super::owner_lock::owner_lock_path;
use super::super::pathing::{journal_path, transaction_sibling, PathIdentity};
use super::super::schema::{JournalIntent, JournalRetirement, TransactionJournal, JOURNAL_VERSION};
use super::super::PreparedFileWrite;
use super::frame_codec::encode_frame;
use crate::io::{atomic_write_new, sync_parent_directory};

pub(in crate::io::transaction) fn plan_intent(
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
    validate_artifact_namespace(directory, &path, &intents)?;
    Ok((path, intents))
}

pub(in crate::io::transaction) fn persist_intent(
    path: &Path,
    tag: &str,
    transaction_id: &str,
    intents: &[JournalIntent],
) -> Result<(), DurableTransactionError> {
    persist_journal(
        path,
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
                    retirements: intent.retirements.clone(),
                })
                .collect(),
            transitions: Vec::new(),
        },
    )
}

#[cfg(test)]
pub(in crate::io::transaction) fn create_intent(
    directory: &Path,
    tag: &str,
    transaction_id: &str,
    writes: &[PreparedFileWrite],
) -> Result<(PathBuf, Vec<JournalIntent>), DurableTransactionError> {
    let (path, intents) = plan_intent(directory, tag, transaction_id, writes)?;
    persist_intent(&path, tag, transaction_id, &intents)?;
    Ok((path, intents))
}

fn validate_artifact_namespace(
    directory: &Path,
    journal: &Path,
    intents: &[JournalIntent],
) -> Result<(), DurableTransactionError> {
    let mut identities = BTreeSet::new();
    let owner_lock = owner_lock_path(directory).map_err(|source| {
        DurableTransactionError::operation(TransactionPhase::Stage, directory, source)
    })?;
    reserve_path(&mut identities, &owner_lock)?;
    reserve_path(&mut identities, journal)?;
    for intent in intents {
        reserve_path(&mut identities, &intent.target)?;
        reserve_path(&mut identities, &intent.staging)?;
        reserve_path(&mut identities, &intent.backup)?;
        reserve_path(&mut identities, &intent.rollback_staging)?;
        for retirement in &intent.retirements {
            reserve_path(&mut identities, &retirement.path)?;
            reserve_path(&mut identities, &retirement.backup)?;
            reserve_path(&mut identities, &retirement.rollback_staging)?;
        }
    }
    for identity in &identities {
        let ancestors = identity.strict_ancestor_identities().map_err(|source| {
            DurableTransactionError::operation(
                TransactionPhase::Stage,
                identity.operation_path(),
                source,
            )
        })?;
        if ancestors
            .into_iter()
            .any(|ancestor| identities.contains(&ancestor))
        {
            return Err(invalid_namespace(
                identity.operation_path(),
                "transaction live and artifact paths overlap an ancestor or descendant",
            ));
        }
    }
    Ok(())
}

fn reserve_path(
    identities: &mut BTreeSet<PathIdentity>,
    path: &Path,
) -> Result<(), DurableTransactionError> {
    let identity = PathIdentity::resolve(path).map_err(|source| {
        DurableTransactionError::operation(TransactionPhase::Stage, path, source)
    })?;
    if identities.insert(identity) {
        Ok(())
    } else {
        Err(invalid_namespace(
            path,
            "transaction live and artifact paths alias",
        ))
    }
}

fn invalid_namespace(path: &Path, reason: &str) -> DurableTransactionError {
    DurableTransactionError::operation(
        TransactionPhase::Stage,
        path,
        io::Error::new(io::ErrorKind::InvalidInput, reason),
    )
}

fn intent_for_write(write: &PreparedFileWrite, tag: &str, transaction_id: &str) -> JournalIntent {
    JournalIntent {
        target: write.path.clone(),
        staging: transaction_sibling(&write.path, tag, "stage", transaction_id),
        backup: transaction_sibling(&write.path, tag, "backup", transaction_id),
        rollback_staging: transaction_sibling(&write.path, tag, "rollback-stage", transaction_id),
        retirements: write
            .retirements
            .iter()
            .map(|retirement| JournalRetirement {
                path: retirement.path.clone(),
                backup: transaction_sibling(
                    &retirement.path,
                    tag,
                    "retired-backup",
                    transaction_id,
                ),
                rollback_staging: transaction_sibling(
                    &retirement.path,
                    tag,
                    "retired-rollback-stage",
                    transaction_id,
                ),
            })
            .collect(),
    }
}

fn persist_journal(
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
    atomic_write_new(path, &frame)
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
