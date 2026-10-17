//! Manifest and pending apply receipt share a project-owned durable transaction.
//! External writers do not participate in this process/OS lock; this is not external CAS.
use super::{lock_project_generation, ProjectManifest, ProjectPaths};
use crate::core::resource::io::transaction::{
    commit_prepared_files_with_revisions, DurableCommitDisposition, DurableCommitReport,
    PreparedFileWrite, TransactionFault,
};
use crate::core::resource::io::{SourceRevision, SourceSnapshot};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::Path};
use zircon_runtime_interface::project::MAX_PROJECT_MANIFEST_BYTES;
#[path = "plugin_apply/recovery.rs"]
mod recovery;
pub(crate) use recovery::recover_plugin_apply_files_locked;
pub use recovery::{ensure_plugin_apply_ready, recover_plugin_apply_files};
const TAG: &str = "plugin-enablement";
const MAX_RECEIPT_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginApplyErrorKind {
    Io,
    Manifest,
    StaleRevision,
    PendingOperation,
    RecoveryRequired,
    InvalidReceipt,
    DurableFailure,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginApplyError {
    pub kind: PluginApplyErrorKind,
    pub cause: String,
}
impl PluginApplyError {
    fn new(kind: PluginApplyErrorKind, cause: impl ToString) -> Self {
        Self {
            kind,
            cause: cause.to_string(),
        }
    }
}
impl fmt::Display for PluginApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.cause)
    }
}
impl std::error::Error for PluginApplyError {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginApplyDurability {
    Durable,
    CleanupDeferred,
    RecoveryDeferred,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingPluginApply {
    pub version: u32,
    pub operation_id: String,
    pub plugin_id: String,
    pub enabled: bool,
    pub previous_enabled: bool,
    pub project_guid: String,
    pub manifest_digest: String,
    pub session_instance: String,
    pub session_generation: u64,
    pub catalog_generation: u64,
    pub acknowledged: bool,
    pub resolved_before_activation: bool,
}
impl PendingPluginApply {
    fn validate(&self) -> Result<(), PluginApplyError> {
        if (self.acknowledged && self.resolved_before_activation)
            || self.version != 1
            || self.operation_id.is_empty()
            || self.operation_id.len() > 128
            || self.plugin_id.is_empty()
            || self.plugin_id.len() > 512
            || self.project_guid.is_empty()
            || self.project_guid.len() > 128
            || self.manifest_digest.len() != 64
            || !self.manifest_digest.bytes().all(|c| c.is_ascii_hexdigit())
            || self.session_instance.len() > 128
        {
            return Err(PluginApplyError::new(
                PluginApplyErrorKind::InvalidReceipt,
                "invalid pending receipt",
            ));
        }
        Ok(())
    }
}
pub struct PluginApplyDocument {
    pub manifest: ProjectManifest,
    pub revision: SourceRevision,
    pub manifest_digest: String,
}
impl PluginApplyDocument {
    pub fn read(paths: &ProjectPaths) -> Result<Self, PluginApplyError> {
        ensure_plugin_apply_ready(paths)?;
        let snapshot =
            SourceSnapshot::read_bounded(paths.manifest_path(), MAX_PROJECT_MANIFEST_BYTES)
                .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
        let text = std::str::from_utf8(snapshot.bytes())
            .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Manifest, e))?;
        let manifest = ProjectManifest::from_toml_str(text)
            .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Manifest, e))?
            .value;
        let manifest_digest = blake3::hash(snapshot.bytes()).to_hex().to_string();
        Ok(Self {
            manifest,
            revision: snapshot.into_parts().1,
            manifest_digest,
        })
    }
}
pub struct PluginApplyCommit {
    pub receipt: PendingPluginApply,
    pub committed_revision: Option<SourceRevision>,
    pub durability: PluginApplyDurability,
}
fn lane(paths: &ProjectPaths) -> std::path::PathBuf {
    paths.derived_root().join("plugin-enablement")
}
fn journal(paths: &ProjectPaths) -> std::path::PathBuf {
    lane(paths).join("journal")
}
fn pending_path(paths: &ProjectPaths) -> std::path::PathBuf {
    lane(paths).join("pending.json")
}
fn check_paths(paths: &ProjectPaths) -> Result<(), PluginApplyError> {
    let actual = ProjectPaths::from_root(paths.root())
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    if actual != *paths {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::StaleRevision,
            "physical project changed",
        ));
    }
    Ok(())
}
pub fn pending_plugin_apply(
    paths: &ProjectPaths,
) -> Result<Option<PendingPluginApply>, PluginApplyError> {
    ensure_plugin_apply_ready(paths)?;
    Ok(read_receipt(paths)?.filter(|r| !r.acknowledged && !r.resolved_before_activation))
}

/// This check is data-only and precedes manifest parse or any project activation.
pub fn ensure_plugin_apply_preflight_ready(paths: &ProjectPaths) -> Result<(), PluginApplyError> {
    ensure_plugin_apply_ready(paths)?;
    if let Some(receipt) = pending_plugin_apply(paths)? {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::PendingOperation,
            format!(
                "pending plugin operation {} requires explicit retry or pre-open revert",
                receipt.operation_id
            ),
        ));
    }
    Ok(())
}
/// Caller owns the exclusive, non-Ready project session admission. No runtime effects occur.
pub fn revert_plugin_apply_before_activation(
    paths: &ProjectPaths,
    operation_id: &str,
    expected: &SourceRevision,
    new_operation_id: String,
) -> Result<PluginApplyCommit, PluginApplyError> {
    let previous = pending_plugin_apply(paths)?
        .filter(|p| p.operation_id == operation_id)
        .ok_or_else(|| {
            PluginApplyError::new(
                PluginApplyErrorKind::StaleRevision,
                "pending operation changed",
            )
        })?;
    let document = verify_pending_plugin_apply(paths, &previous)?;
    if &document.revision != expected {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::StaleRevision,
            "expected committed revision changed",
        ));
    }
    let mut candidate = document.manifest.clone();
    let selection = candidate
        .plugins
        .selections
        .iter_mut()
        .find(|p| p.id == previous.plugin_id)
        .ok_or_else(|| {
            PluginApplyError::new(
                PluginApplyErrorKind::InvalidReceipt,
                "pending selection is absent",
            )
        })?;
    selection.enabled = previous.previous_enabled;
    let receipt = PendingPluginApply {
        operation_id: new_operation_id,
        enabled: previous.previous_enabled,
        previous_enabled: previous.enabled,
        acknowledged: false,
        resolved_before_activation: true,
        ..previous
    };
    commit_with_fault(
        paths,
        &document,
        &candidate,
        receipt,
        Some(operation_id),
        TransactionFault::None,
    )
}

fn read_receipt(paths: &ProjectPaths) -> Result<Option<PendingPluginApply>, PluginApplyError> {
    check_paths(paths)?;
    let snapshot = match SourceSnapshot::read_bounded(&pending_path(paths), MAX_RECEIPT_BYTES) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(PluginApplyError::new(PluginApplyErrorKind::Io, e)),
    };
    let receipt: PendingPluginApply = serde_json::from_slice(snapshot.bytes())
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::InvalidReceipt, e))?;
    receipt.validate()?;
    Ok(Some(receipt))
}
pub fn commit_plugin_apply(
    paths: &ProjectPaths,
    document: &PluginApplyDocument,
    candidate: &ProjectManifest,
    mut receipt: PendingPluginApply,
    supersedes: Option<&str>,
) -> Result<PluginApplyCommit, PluginApplyError> {
    receipt.resolved_before_activation = false;
    commit_with_fault(
        paths,
        document,
        candidate,
        receipt,
        supersedes,
        TransactionFault::None,
    )
}
fn commit_with_fault(
    paths: &ProjectPaths,
    document: &PluginApplyDocument,
    candidate: &ProjectManifest,
    mut receipt: PendingPluginApply,
    supersedes: Option<&str>,
    fault: TransactionFault,
) -> Result<PluginApplyCommit, PluginApplyError> {
    let _generation = lock_project_generation(paths.root())
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    check_paths(paths)?;
    ensure_plugin_apply_ready(paths)?;
    if let Some(previous) = pending_plugin_apply(paths)? {
        if supersedes != Some(previous.operation_id.as_str())
            || previous.manifest_digest != document.manifest_digest
        {
            return Err(PluginApplyError::new(
                PluginApplyErrorKind::PendingOperation,
                previous.operation_id,
            ));
        }
    } else if supersedes.is_some() {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::StaleRevision,
            "superseded operation is no longer pending",
        ));
    }
    if candidate.project_guid != document.manifest.project_guid {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::StaleRevision,
            "candidate belongs to a different project",
        ));
    }
    let bytes = candidate
        .to_current_format_bytes()
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Manifest, e))?;
    receipt.manifest_digest = blake3::hash(&bytes).to_hex().to_string();
    receipt.project_guid = candidate.project_guid.to_string();
    receipt.acknowledged = false;
    receipt.validate()?;
    let marker = serde_json::to_vec(&receipt)
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::InvalidReceipt, e))?;
    if marker.len() > MAX_RECEIPT_BYTES {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::InvalidReceipt,
            "receipt exceeds bound",
        ));
    }
    let marker_revision = SourceRevision::observe(&pending_path(paths))
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    let writes = vec![
        PreparedFileWrite::new(paths.manifest_path().to_path_buf(), bytes)
            .expecting_source(document.revision.clone()),
        PreparedFileWrite::new(pending_path(paths), marker).expecting_source(marker_revision),
    ];
    let mut report = DurableCommitReport::default();
    let committed =
        commit_prepared_files_with_revisions(&journal(paths), TAG, writes, fault, &mut report)
            .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::DurableFailure, e))?;
    let (disposition, revisions) = committed.into_parts();
    let committed_revision = revisions
        .into_iter()
        .find(|(p, _)| p == paths.manifest_path())
        .map(|(_, r)| r);
    Ok(PluginApplyCommit {
        receipt,
        committed_revision,
        durability: durability(disposition),
    })
}
fn durability(value: DurableCommitDisposition) -> PluginApplyDurability {
    match value {
        DurableCommitDisposition::Durable => PluginApplyDurability::Durable,
        DurableCommitDisposition::CleanupDeferred => PluginApplyDurability::CleanupDeferred,
        DurableCommitDisposition::CommitRecoveryDeferred => PluginApplyDurability::RecoveryDeferred,
    }
}

/// File-only acknowledgement never repeats lifecycle callbacks.
pub fn acknowledge_plugin_apply(
    paths: &ProjectPaths,
    operation_id: &str,
    committed_revision: &SourceRevision,
) -> Result<PluginApplyDurability, PluginApplyError> {
    let _generation = lock_project_generation(paths.root())
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    check_paths(paths)?;
    ensure_plugin_apply_ready(paths)?;
    let mut receipt = pending_plugin_apply(paths)?.ok_or_else(|| {
        PluginApplyError::new(
            PluginApplyErrorKind::StaleRevision,
            "operation is no longer pending",
        )
    })?;
    let current = SourceSnapshot::read_bounded(paths.manifest_path(), MAX_PROJECT_MANIFEST_BYTES)
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    if receipt.operation_id != operation_id
        || current.revision() != committed_revision
        || blake3::hash(current.bytes()).to_hex().as_str() != receipt.manifest_digest
    {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::StaleRevision,
            "manifest or operation changed before acknowledgement",
        ));
    }
    let marker = SourceSnapshot::read_bounded(&pending_path(paths), MAX_RECEIPT_BYTES)
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    receipt.acknowledged = true;
    let bytes = serde_json::to_vec(&receipt)
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::InvalidReceipt, e))?;
    let writes =
        vec![PreparedFileWrite::new(pending_path(paths), bytes)
            .expecting_source(marker.into_parts().1)];
    let mut report = DurableCommitReport::default();
    let result = commit_prepared_files_with_revisions(
        &journal(paths),
        TAG,
        writes,
        TransactionFault::None,
        &mut report,
    )
    .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::DurableFailure, e))?;
    Ok(durability(result.into_parts().0))
}
pub fn verify_pending_plugin_apply(
    paths: &ProjectPaths,
    receipt: &PendingPluginApply,
) -> Result<PluginApplyDocument, PluginApplyError> {
    if pending_plugin_apply(paths)?.as_ref() != Some(receipt) {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::StaleRevision,
            "pending operation changed",
        ));
    }
    let document = PluginApplyDocument::read(paths)?;
    if document.manifest.project_guid.to_string() != receipt.project_guid
        || document.manifest_digest != receipt.manifest_digest
    {
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::StaleRevision,
            "pending receipt does not describe current manifest",
        ));
    }
    Ok(document)
}
#[cfg(test)]
#[path = "plugin_apply/tests/cases.rs"]
mod tests;
