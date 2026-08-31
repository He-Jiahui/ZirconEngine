use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::super::error::DurableTransactionError;
use super::super::schema::{JournalDocument, JournalPhase, JournalState};
use super::super::stage::{FilePresence, file_presence};
use super::RecoveryPolicy;

pub(super) fn validate_document_evidence(
    journal_path: &Path,
    phase: JournalPhase,
    document: &JournalDocument,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    if document.state == JournalState::Intent {
        return Ok(());
    }
    let target_existed = document.target_existed.ok_or_else(|| {
        DurableTransactionError::invalid(journal_path, "missing target origin evidence")
    })?;
    let new_digest = document.new_digest.as_deref().ok_or_else(|| {
        DurableTransactionError::invalid(journal_path, "missing staged digest evidence")
    })?;
    if target_existed != document.original_digest.is_some() {
        return Err(DurableTransactionError::invalid(
            journal_path,
            "target origin and original digest disagree",
        ));
    }
    if document.retirements.len() != document.retired_digests.len() {
        return Err(DurableTransactionError::invalid(
            journal_path,
            "retired file and digest evidence disagree",
        ));
    }

    let cleanup = matches!(
        phase,
        JournalPhase::CleanupIntent | JournalPhase::Cleanup | JournalPhase::CleanupRollback
    );
    match phase {
        JournalPhase::Intent
        | JournalPhase::CleanupIntent
        | JournalPhase::RollbackCompleted
        | JournalPhase::CleanupRollback => {
            require_original_target(journal_path, document, policy, evidence)?;
            require_retired_original(journal_path, document, policy, evidence)?;
        }
        JournalPhase::Active => match document.state {
            JournalState::Prepared => {
                require_original_target(journal_path, document, policy, evidence)?;
                require_retired_original(journal_path, document, policy, evidence)?;
            }
            JournalState::Committing | JournalState::Committed | JournalState::RollingBack => {
                require_original_or_new_target(
                    journal_path,
                    document,
                    new_digest,
                    policy,
                    evidence,
                )?;
                require_retired_original_or_missing(journal_path, document, policy, evidence)?;
            }
            JournalState::Intent => unreachable!("handled before evidence validation"),
        },
        JournalPhase::AllCommitted | JournalPhase::Cleanup => {
            require_digest(
                journal_path,
                &document.target,
                new_digest,
                false,
                "committed target",
                policy,
                evidence,
            )?;
            require_retired_missing(journal_path, document)?;
        }
    }

    let staging_may_be_missing = cleanup
        || phase == JournalPhase::RollbackCompleted
        || matches!(
            document.state,
            JournalState::Committing | JournalState::Committed | JournalState::RollingBack
        );
    require_digest(
        journal_path,
        &document.staging,
        new_digest,
        staging_may_be_missing,
        "staging artifact",
        policy,
        evidence,
    )?;
    if let Some(original) = document.original_digest.as_deref() {
        require_digest(
            journal_path,
            &document.backup,
            original,
            cleanup,
            "backup artifact",
            policy,
            evidence,
        )?;
    } else if checked_file_presence(journal_path, &document.backup)? == FilePresence::Present {
        return Err(DurableTransactionError::invalid(
            journal_path,
            "backup exists for a newly-created target",
        ));
    }
    for (retirement, digest) in document.retirements.iter().zip(&document.retired_digests) {
        require_digest(
            journal_path,
            &retirement.backup,
            digest,
            cleanup,
            "retired backup",
            policy,
            evidence,
        )?;
    }
    Ok(())
}

fn require_original_target(
    journal_path: &Path,
    document: &JournalDocument,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    match document.original_digest.as_deref() {
        Some(digest) => require_digest(
            journal_path,
            &document.target,
            digest,
            false,
            "original target",
            policy,
            evidence,
        ),
        None => match checked_file_presence(journal_path, &document.target)? {
            FilePresence::Missing => Ok(()),
            FilePresence::Present => Err(DurableTransactionError::invalid(
                journal_path,
                "new target exists before transaction publication",
            )),
        },
    }
}

fn require_original_or_new_target(
    journal_path: &Path,
    document: &JournalDocument,
    new_digest: &str,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    if checked_file_presence(journal_path, &document.target)? == FilePresence::Missing {
        // A replace may have moved the old target to its backup before an ambiguous failure.
        // The mandatory backup digest check below decides whether rollback is still authoritative.
        return Ok(());
    }
    let actual = file_evidence(journal_path, &document.target, policy, evidence)?;
    if actual == new_digest
        || document
            .original_digest
            .as_deref()
            .is_some_and(|original| actual == original)
    {
        Ok(())
    } else {
        Err(DurableTransactionError::invalid(
            journal_path,
            "active target matches neither old nor new generation",
        ))
    }
}

fn require_retired_original(
    journal_path: &Path,
    document: &JournalDocument,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    for (retirement, digest) in document.retirements.iter().zip(&document.retired_digests) {
        require_digest(
            journal_path,
            &retirement.path,
            digest,
            false,
            "retired live file",
            policy,
            evidence,
        )?;
    }
    Ok(())
}

fn require_retired_original_or_missing(
    journal_path: &Path,
    document: &JournalDocument,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    for (retirement, digest) in document.retirements.iter().zip(&document.retired_digests) {
        require_digest(
            journal_path,
            &retirement.path,
            digest,
            true,
            "retired live file",
            policy,
            evidence,
        )?;
    }
    Ok(())
}

fn require_retired_missing(
    journal_path: &Path,
    document: &JournalDocument,
) -> Result<(), DurableTransactionError> {
    for retirement in &document.retirements {
        if checked_file_presence(journal_path, &retirement.path)? == FilePresence::Present {
            return Err(DurableTransactionError::invalid(
                journal_path,
                "retired live file remains after committed generation",
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn require_digest(
    journal_path: &Path,
    path: &Path,
    expected: &str,
    allow_missing: bool,
    label: &str,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<(), DurableTransactionError> {
    if checked_file_presence(journal_path, path)? == FilePresence::Missing {
        return if allow_missing {
            Ok(())
        } else {
            Err(DurableTransactionError::invalid(
                journal_path,
                format!("{label} is missing"),
            ))
        };
    }
    if file_evidence(journal_path, path, policy, evidence)? == expected {
        Ok(())
    } else {
        Err(DurableTransactionError::invalid(
            journal_path,
            format!("{label} digest does not match journal evidence"),
        ))
    }
}

fn file_evidence(
    journal_path: &Path,
    path: &Path,
    policy: &mut impl RecoveryPolicy,
    evidence: &mut EvidenceCache,
) -> Result<String, DurableTransactionError> {
    if let Some(digest) = evidence.digests.get(path) {
        return Ok(digest.clone());
    }
    let digest = policy.digest_file(path).map_err(|source| {
        DurableTransactionError::invalid(
            journal_path,
            format!("invalid transaction evidence {}: {source}", path.display()),
        )
    })?;
    evidence.digests.insert(path.to_path_buf(), digest.clone());
    Ok(digest)
}

fn checked_file_presence(
    journal_path: &Path,
    path: &Path,
) -> Result<FilePresence, DurableTransactionError> {
    file_presence(path).map_err(|source| {
        DurableTransactionError::invalid(
            journal_path,
            format!("invalid transaction evidence {}: {source}", path.display()),
        )
    })
}

#[derive(Default)]
pub(super) struct EvidenceCache {
    digests: HashMap<PathBuf, String>,
}
