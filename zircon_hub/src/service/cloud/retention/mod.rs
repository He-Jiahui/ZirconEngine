pub(super) mod gc;
mod maintenance;
mod policy;

pub use maintenance::{maintain, MaintenanceReport, MaintenanceRequest};
pub(super) use policy::{authorize_admin, authorize_receipt, load};
pub use policy::{
    get, update, RetentionOutcome, RetentionPolicy, RetentionRequest, RetentionState,
};

use super::{authorization::authorize, BlobStore};
use crate::service::{
    error::ServiceError,
    identity::{now_seconds, Principal},
};
use rusqlite::{params, Connection, Transaction, TransactionBehavior};

pub(super) const UPLOAD_LEASE_SECONDS: u64 = 86_400;
pub(super) const GC_BATCH: u64 = 64;
pub(super) const GC_BATCH_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn pending(
    connection: &Connection,
    organization: &str,
    project: &str,
) -> Result<u64, ServiceError> {
    Ok(connection.query_row(
        "SELECT (SELECT COUNT(*) FROM cloud_gc_pending WHERE organization_id=?1 AND project_id=?2)
         + (SELECT COUNT(*) FROM cloud_blobs b JOIN cloud_retention_policies p
            ON p.organization_id=b.organization_id AND p.project_id=b.project_id
            WHERE b.organization_id=?1 AND b.project_id=?2 AND p.keep_latest IS NOT NULL AND p.legal_hold=0
            AND NOT EXISTS(SELECT 1 FROM snapshot_blobs s WHERE s.organization_id=b.organization_id AND s.project_id=b.project_id AND s.digest=b.digest)
            AND NOT EXISTS(SELECT 1 FROM cloud_upload_leases l WHERE l.organization_id=b.organization_id AND l.project_id=b.project_id AND l.digest=b.digest AND l.expires_at>?3))",
        params![organization, project, now_seconds()],
        |row| row.get(0),
    )?)
}

pub(super) fn apply(
    transaction: &Transaction<'_>,
    organization: &str,
    project: &str,
    now: u64,
) -> Result<u64, ServiceError> {
    let policy = load(transaction, organization, project)?.policy;
    let Some(keep_latest) = policy.keep_latest.filter(|_| !policy.legal_hold) else {
        transaction.execute(
            "DELETE FROM cloud_snapshot_trash WHERE organization_id=?1 AND project_id=?2",
            params![organization, project],
        )?;
        return Ok(0);
    };
    transaction.execute(
        "DELETE FROM cloud_snapshot_trash WHERE organization_id=?1 AND project_id=?2 AND
         (revision IN (SELECT revision FROM project_snapshots WHERE organization_id=?1 AND project_id=?2 ORDER BY revision DESC LIMIT ?3)
         OR revision IN (SELECT revision FROM project_heads WHERE organization_id=?1 AND project_id=?2))",
        params![organization, project, keep_latest],
    )?;
    transaction.execute(
        "INSERT OR IGNORE INTO cloud_snapshot_trash
         SELECT organization_id,project_id,revision,?4 FROM project_snapshots
         WHERE organization_id=?1 AND project_id=?2
         AND revision NOT IN (SELECT revision FROM project_snapshots WHERE organization_id=?1 AND project_id=?2 ORDER BY revision DESC LIMIT ?3)
         AND revision NOT IN (SELECT revision FROM project_heads WHERE organization_id=?1 AND project_id=?2)",
        params![organization, project, keep_latest, now + policy.trash_seconds],
    )?;
    let expired: Vec<i64> = transaction
        .prepare(
            "SELECT revision FROM cloud_snapshot_trash WHERE organization_id=?1 AND project_id=?2
             AND purge_after<=?3
             AND revision NOT IN (SELECT revision FROM project_heads WHERE organization_id=?1 AND project_id=?2)
             ORDER BY revision LIMIT ?4",
        )?
        .query_map(params![organization, project, now, super::quota::MAX_REVISIONS], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    for revision in &expired {
        transaction.execute(
            "DELETE FROM snapshot_blobs WHERE organization_id=?1 AND project_id=?2 AND revision=?3",
            params![organization, project, revision],
        )?;
        transaction.execute(
            "DELETE FROM cloud_snapshot_trash WHERE organization_id=?1 AND project_id=?2 AND revision=?3",
            params![organization, project, revision],
        )?;
        transaction.execute(
            "DELETE FROM project_snapshots WHERE organization_id=?1 AND project_id=?2 AND revision=?3",
            params![organization, project, revision],
        )?;
    }
    gc::stage(transaction, organization, project, now)?;
    Ok(expired.len() as u64)
}

pub(super) fn collect_before_write(
    connection: &mut Connection,
    principal: &Principal,
    store: &BlobStore,
    organization: &str,
    project: &str,
) -> Result<(), ServiceError> {
    // Commit physical deletes in their own writer transaction before a caller can
    // begin a different mutation. A later mutation rollback can therefore never
    // resurrect database rows for a file already removed by this pass.
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    authorize(&transaction, principal, organization, project, true)?;
    gc::collect(&transaction, store, Some((organization, project)))?;
    transaction.commit()?;
    Ok(())
}
