use super::{super::BlobStore, GC_BATCH, GC_BATCH_BYTES};
use crate::service::error::ServiceError;
use rusqlite::{params, Connection, Transaction, TransactionBehavior};

pub(super) fn stage(
    transaction: &Transaction<'_>,
    organization: &str,
    project: &str,
    now: u64,
) -> Result<(), ServiceError> {
    let candidates: Vec<(String, u64)> = transaction.prepare(
        "SELECT b.digest,b.bytes FROM cloud_blobs b WHERE b.organization_id=?1 AND b.project_id=?2
         AND NOT EXISTS(SELECT 1 FROM snapshot_blobs s WHERE s.organization_id=b.organization_id AND s.project_id=b.project_id AND s.digest=b.digest)
         AND NOT EXISTS(SELECT 1 FROM cloud_upload_leases l WHERE l.organization_id=b.organization_id AND l.project_id=b.project_id AND l.digest=b.digest AND l.expires_at>?3)
         ORDER BY b.digest LIMIT ?4",
    )?.query_map(params![organization, project, now, GC_BATCH], |row| Ok((row.get(0)?, row.get(1)?)))?.collect::<Result<_, _>>()?;
    for (digest, bytes) in candidates {
        transaction.execute(
            "DELETE FROM cloud_upload_leases WHERE organization_id=?1 AND project_id=?2 AND digest=?3",
            params![organization, project, digest],
        )?;
        transaction.execute(
            "DELETE FROM cloud_blobs WHERE organization_id=?1 AND project_id=?2 AND digest=?3",
            params![organization, project, digest],
        )?;
        transaction.execute(
            "INSERT OR IGNORE INTO cloud_gc_pending SELECT ?1,?2,?3,?4
             WHERE NOT EXISTS(SELECT 1 FROM cloud_blobs WHERE digest=?1)",
            params![digest, bytes, organization, project],
        )?;
    }
    Ok(())
}

pub(in crate::service::cloud) fn collect(
    transaction: &Transaction<'_>,
    store: &BlobStore,
    scope: Option<(&str, &str)>,
) -> Result<u64, ServiceError> {
    let (organization, project) =
        scope.map_or((None, None), |(org, project)| (Some(org), Some(project)));
    let candidates: Vec<(String, u64)> = transaction.prepare(
        "SELECT digest,bytes FROM cloud_gc_pending WHERE (?1 IS NULL OR (organization_id=?1 AND project_id=?2))
         ORDER BY digest LIMIT ?3",
    )?.query_map(params![organization, project, GC_BATCH], |row| Ok((row.get(0)?, row.get(1)?)))?.collect::<Result<_, _>>()?;
    let mut collected = 0;
    let mut collected_bytes = 0;
    for (digest, bytes) in candidates {
        // The immediate writer transaction excludes upload/commit until file removal and ack finish.
        let referenced: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM cloud_blobs WHERE digest=?1)
             OR EXISTS(SELECT 1 FROM snapshot_blobs WHERE digest=?1)
             OR EXISTS(SELECT 1 FROM cloud_upload_leases WHERE digest=?1)",
            [&digest],
            |row| row.get(0),
        )?;
        if !referenced {
            if bytes > super::super::MAX_BLOB_BYTES as u64 {
                return Err(ServiceError::Storage);
            }
            let stored_bytes = bytes + super::super::store::ENCRYPTION_OVERHEAD;
            if collected_bytes + stored_bytes > GC_BATCH_BYTES {
                break;
            }
            store.remove_unreferenced(&digest, bytes)?;
            collected += 1;
            collected_bytes += stored_bytes;
        }
        transaction.execute("DELETE FROM cloud_gc_pending WHERE digest=?1", [&digest])?;
    }
    Ok(collected)
}

pub(in crate::service::cloud) fn recover(
    connection: &mut Connection,
    store: &BlobStore,
) -> Result<(), ServiceError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    collect(&transaction, store, None)?;
    transaction.commit()?;
    Ok(())
}
