use super::{authorization::authorize, quota::MAX_REVISIONS, BlobStore, Manifest};
use crate::service::{
    error::ServiceError,
    identity::{now_seconds, Principal},
    organization,
    storage::receipt,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommitRequest {
    pub operation_id: String,
    pub base_revision: String,
    pub manifest: Manifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub organization_id: String,
    pub project_id: String,
    pub base_revision: String,
    pub revision: String,
    pub manifest_digest: String,
    pub created_at: u64,
    pub created_by: String,
    pub manifest: Manifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotReceipt {
    pub organization_id: String,
    pub project_id: String,
    pub base_revision: String,
    pub revision: String,
    pub manifest_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CommitOutcome {
    Committed {
        snapshot: SnapshotReceipt,
    },
    Conflict {
        organization_id: String,
        project_id: String,
        base_revision: String,
        current_revision: String,
        manifest_digest: String,
    },
}

pub fn authorize_receipt(
    connection: &Connection,
    principal: &Principal,
    result: &serde_json::Value,
) -> Result<(), ServiceError> {
    if !matches!(
        result.get("status").and_then(serde_json::Value::as_str),
        Some("committed" | "conflict")
    ) {
        return super::retention::authorize_receipt(connection, principal, result);
    }
    let outcome: CommitOutcome =
        serde_json::from_value(result.clone()).map_err(|_| ServiceError::Storage)?;
    let (organization, project) = match &outcome {
        CommitOutcome::Committed { snapshot } => (&snapshot.organization_id, &snapshot.project_id),
        CommitOutcome::Conflict {
            organization_id,
            project_id,
            ..
        } => (organization_id, project_id),
    };
    authorize(connection, principal, organization, project, false)?;
    Ok(())
}

pub fn head(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    project: &str,
) -> Result<Option<Snapshot>, ServiceError> {
    authorize(connection, principal, organization, project, false)?;
    let row: Option<String> = connection.query_row(
        "SELECT s.manifest_json FROM project_heads h JOIN project_snapshots s
         ON s.organization_id=h.organization_id AND s.project_id=h.project_id AND s.revision=h.revision
         WHERE h.organization_id=?1 AND h.project_id=?2",
        params![organization, project], |row| row.get(0),
    ).optional()?;
    row.map(|json| serde_json::from_str(&json).map_err(|_| ServiceError::Storage))
        .transpose()
}

pub fn commit(
    connection: &mut Connection,
    principal: &Principal,
    store: &BlobStore,
    organization: &str,
    project: &str,
    mut request: CommitRequest,
) -> Result<CommitOutcome, ServiceError> {
    receipt::validate_id(&request.operation_id)?;
    let base = request
        .base_revision
        .parse::<i64>()
        .map_err(|_| ServiceError::InvalidRequest)?;
    if base < 0 || base.to_string() != request.base_revision {
        return Err(ServiceError::InvalidRequest);
    }
    let manifest_digest = request.manifest.canonicalize()?;
    let fingerprint = receipt::fingerprint(&(
        "cloud.commit",
        organization,
        project,
        &request.base_revision,
        &manifest_digest,
    ))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let policy_revision = authorize(&transaction, principal, organization, project, true)?;
    if let Some(result) =
        receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?
    {
        return Ok(result);
    }
    let current: Option<i64> = transaction
        .query_row(
            "SELECT revision FROM project_heads WHERE organization_id=?1 AND project_id=?2",
            params![organization, project],
            |row| row.get(0),
        )
        .optional()?;
    let current = current.unwrap_or(0);
    if current != base {
        let outcome = CommitOutcome::Conflict {
            organization_id: organization.into(),
            project_id: project.into(),
            base_revision: request.base_revision,
            current_revision: current.to_string(),
            manifest_digest,
        };
        // The conflict is a durable terminal operation, even when the HTTP reply is lost.
        organization::audit(
            &transaction,
            organization,
            principal,
            "cloud.commit",
            "conflict",
            policy_revision,
        )?;
        receipt::commit(
            &transaction,
            principal,
            &request.operation_id,
            &fingerprint,
            &outcome,
        )?;
        transaction.commit()?;
        return Ok(outcome);
    }
    let mut verified = std::collections::BTreeSet::new();
    for file in &request.manifest.files {
        let size: Option<u64> = transaction.query_row("SELECT bytes FROM cloud_blobs WHERE organization_id=?1 AND project_id=?2 AND digest=?3", params![organization, project, file.digest], |row| row.get(0)).optional()?;
        if size != Some(file.bytes) {
            return Err(ServiceError::InvalidRequest);
        }
        if verified.insert(&file.digest) && store.read(&file.digest)?.len() as u64 != file.bytes {
            return Err(ServiceError::Storage);
        }
    }
    let next = base.checked_add(1).ok_or(ServiceError::Capacity)?;
    let snapshot = Snapshot {
        organization_id: organization.into(),
        project_id: project.into(),
        base_revision: request.base_revision,
        revision: next.to_string(),
        manifest_digest,
        created_at: now_seconds(),
        created_by: receipt::fingerprint(&(&principal.issuer, &principal.subject))?,
        manifest: request.manifest,
    };
    let json = serde_json::to_string(&snapshot).map_err(|_| ServiceError::Storage)?;
    transaction.execute(
        "INSERT INTO project_snapshots VALUES (?1,?2,?3,?4)",
        params![organization, project, next, json],
    )?;
    for file in &snapshot.manifest.files {
        transaction.execute(
            "INSERT OR IGNORE INTO snapshot_blobs VALUES (?1,?2,?3,?4)",
            params![organization, project, next, file.digest],
        )?;
    }
    transaction.execute("INSERT INTO project_heads VALUES (?1,?2,?3) ON CONFLICT(organization_id,project_id) DO UPDATE SET revision=excluded.revision", params![organization, project, next])?;
    super::retention::apply(&transaction, organization, project, snapshot.created_at)?;
    let revisions: u64 = transaction.query_row(
        "SELECT COUNT(*) FROM project_snapshots WHERE organization_id=?1 AND project_id=?2",
        params![organization, project],
        |row| row.get(0),
    )?;
    if revisions > MAX_REVISIONS {
        return Err(ServiceError::Capacity);
    }
    organization::audit(
        &transaction,
        organization,
        principal,
        "cloud.commit",
        "allowed",
        policy_revision,
    )?;
    let outcome = CommitOutcome::Committed {
        snapshot: SnapshotReceipt {
            organization_id: snapshot.organization_id,
            project_id: snapshot.project_id,
            base_revision: snapshot.base_revision,
            revision: snapshot.revision,
            manifest_digest: snapshot.manifest_digest,
        },
    };
    receipt::commit(
        &transaction,
        principal,
        &request.operation_id,
        &fingerprint,
        &outcome,
    )?;
    transaction.commit()?;
    Ok(outcome)
}
