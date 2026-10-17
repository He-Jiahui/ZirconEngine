use super::{apply, authorize_admin, gc, pending};
use crate::service::{
    cloud::BlobStore,
    error::ServiceError,
    identity::{now_seconds, Principal},
    organization,
    storage::receipt,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaintenanceRequest {
    pub operation_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum MaintenanceStatus {
    MaintenanceCompleted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceReport {
    status: MaintenanceStatus,
    pub organization_id: String,
    pub project_id: String,
    pub pruned_revisions: u64,
    pub collected_objects: u64,
    pub pending_objects: u64,
}

pub fn maintain(
    connection: &mut Connection,
    principal: &Principal,
    store: &BlobStore,
    organization: &str,
    project: &str,
    request: MaintenanceRequest,
) -> Result<MaintenanceReport, ServiceError> {
    receipt::validate_id(&request.operation_id)?;
    run(connection, principal, store, organization, project, request).map_err(|error| match error {
        // Storage failure cannot prove which durable phase ran before a lost reply.
        ServiceError::Storage | ServiceError::Capacity => ServiceError::OutcomeUnknown,
        other => other,
    })
}

fn run(
    connection: &mut Connection,
    principal: &Principal,
    store: &BlobStore,
    organization: &str,
    project: &str,
    request: MaintenanceRequest,
) -> Result<MaintenanceReport, ServiceError> {
    let fingerprint = receipt::fingerprint(&("cloud.maintenance", organization, project))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let prepared: Option<(String, u64)> = transaction.query_row(
        "SELECT fingerprint,pruned_revisions FROM cloud_maintenance_runs WHERE issuer=?1 AND subject=?2 AND operation_id=?3",
        params![principal.issuer, principal.subject, request.operation_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional()?;
    let revision =
        authorize_admin(&transaction, principal, organization, project).map_err(|error| {
            if prepared.is_some() {
                ServiceError::OutcomeUnknown
            } else {
                error
            }
        })?;
    if let Some(result) =
        receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?
    {
        return Ok(result);
    }
    match prepared {
        Some((stored, _)) if stored != fingerprint => return Err(ServiceError::OperationConflict),
        Some(_) => {}
        None => {
            let pruned = apply(&transaction, organization, project, now_seconds())?;
            transaction.execute(
                "INSERT INTO cloud_maintenance_runs VALUES (?1,?2,?3,?4,?5)",
                params![
                    principal.issuer,
                    principal.subject,
                    request.operation_id,
                    fingerprint,
                    pruned
                ],
            )?;
            organization::audit(
                &transaction,
                organization,
                principal,
                "cloud.maintenance.prepare",
                "allowed",
                revision,
            )?;
        }
    }
    transaction.commit()?;

    // A prepared row survives response loss and filesystem/ack faults. Only this
    // second transaction emits a terminal receipt, after the bounded collection.
    finish(
        connection,
        principal,
        store,
        organization,
        project,
        &request.operation_id,
        &fingerprint,
    )
    .map_err(|_| ServiceError::OutcomeUnknown)
}

fn finish(
    connection: &mut Connection,
    principal: &Principal,
    store: &BlobStore,
    organization: &str,
    project: &str,
    operation_id: &str,
    fingerprint: &str,
) -> Result<MaintenanceReport, ServiceError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let revision = authorize_admin(&transaction, principal, organization, project)?;
    if let Some(result) = receipt::replay(&transaction, principal, operation_id, fingerprint)? {
        return Ok(result);
    }
    let pruned_revisions = transaction.query_row(
        "SELECT pruned_revisions FROM cloud_maintenance_runs WHERE issuer=?1 AND subject=?2 AND operation_id=?3 AND fingerprint=?4",
        params![principal.issuer, principal.subject, operation_id, fingerprint],
        |row| row.get(0),
    )?;
    let collected_objects = gc::collect(&transaction, store, Some((organization, project)))?;
    let report = MaintenanceReport {
        status: MaintenanceStatus::MaintenanceCompleted,
        organization_id: organization.into(),
        project_id: project.into(),
        pruned_revisions,
        collected_objects,
        pending_objects: pending(&transaction, organization, project)?,
    };
    organization::audit(
        &transaction,
        organization,
        principal,
        "cloud.maintenance.collect",
        "allowed",
        revision,
    )?;
    receipt::commit(&transaction, principal, operation_id, fingerprint, &report)?;
    transaction.execute(
        "DELETE FROM cloud_maintenance_runs WHERE issuer=?1 AND subject=?2 AND operation_id=?3",
        params![principal.issuer, principal.subject, operation_id],
    )?;
    transaction.commit()?;
    Ok(report)
}

pub(super) fn authorize_receipt(
    connection: &Connection,
    principal: &Principal,
    result: &serde_json::Value,
) -> Result<(), ServiceError> {
    if result.get("status").and_then(serde_json::Value::as_str) != Some("maintenanceCompleted") {
        return Ok(());
    }
    let report: MaintenanceReport =
        serde_json::from_value(result.clone()).map_err(|_| ServiceError::Storage)?;
    super::super::authorization::authorize(
        connection,
        principal,
        &report.organization_id,
        &report.project_id,
        false,
    )?;
    Ok(())
}
