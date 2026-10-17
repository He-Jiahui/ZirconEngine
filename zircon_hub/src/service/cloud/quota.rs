use super::authorization::authorize;
use crate::service::{error::ServiceError, identity::Principal};
use rusqlite::{params, Connection};
use serde::Serialize;

pub(super) const MAX_GLOBAL_BYTES: u64 = 8 * 1024 * 1024 * 1024;
pub(super) const MAX_PROJECT_OBJECTS: u64 = 100_000;
pub(super) const MAX_GLOBAL_OBJECTS: u64 = 500_000;
pub(super) const MAX_REVISIONS: u64 = 128;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub logical_bytes: u64,
    pub physical_bytes: u64,
    pub objects: u64,
    pub revisions: u64,
    pub max_logical_bytes: u64,
    pub max_objects: u64,
    pub max_revisions: u64,
    pub max_blob_bytes: usize,
    pub retention: &'static str,
    pub retention_policy: super::RetentionState,
    pub trash_revisions: u64,
    pub pending_gc_objects: u64,
    pub upload_lease_seconds: u64,
}

pub fn usage(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    project: &str,
) -> Result<Usage, ServiceError> {
    authorize(connection, principal, organization, project, false)?;
    let (logical_bytes, mut physical_bytes, objects): (u64, u64, u64) = connection.query_row(
        "SELECT COALESCE(SUM(bytes),0),COALESCE(SUM(bytes + ?3),0),COUNT(*) FROM cloud_blobs WHERE organization_id=?1 AND project_id=?2",
        params![organization, project, super::store::ENCRYPTION_OVERHEAD], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let retired_bytes: u64 = connection.query_row(
        "SELECT COALESCE(SUM(bytes + ?3),0) FROM cloud_gc_pending WHERE organization_id=?1 AND project_id=?2",
        params![organization, project, super::store::ENCRYPTION_OVERHEAD],
        |row| row.get(0),
    )?;
    physical_bytes += retired_bytes;
    let revisions = connection.query_row(
        "SELECT COUNT(*) FROM project_snapshots WHERE organization_id=?1 AND project_id=?2",
        params![organization, project],
        |row| row.get(0),
    )?;
    let retention_policy = super::retention::load(connection, organization, project)?;
    let trash_revisions = connection.query_row(
        "SELECT COUNT(*) FROM cloud_snapshot_trash WHERE organization_id=?1 AND project_id=?2",
        params![organization, project],
        |row| row.get(0),
    )?;
    Ok(Usage {
        logical_bytes,
        physical_bytes,
        objects,
        revisions,
        max_logical_bytes: super::manifest::MAX_PROJECT_BYTES,
        max_objects: MAX_PROJECT_OBJECTS,
        max_revisions: MAX_REVISIONS,
        max_blob_bytes: super::MAX_BLOB_BYTES,
        retention: if retention_policy.policy.keep_latest.is_some() {
            "keep-latest-with-trash"
        } else {
            "retain-all-reject-at-limit"
        },
        retention_policy,
        trash_revisions,
        pending_gc_objects: super::retention::pending(connection, organization, project)?,
        upload_lease_seconds: super::retention::UPLOAD_LEASE_SECONDS,
    })
}
