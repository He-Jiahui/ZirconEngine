use crate::service::{
    error::ServiceError,
    identity::{now_seconds, Principal},
};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum OperationStatus {
    Committed { result: serde_json::Value },
    // Absence is not proof of cancellation: a worker may be queued or durably prepared.
    Unknown,
}

pub fn validate_id(operation_id: &str) -> Result<(), ServiceError> {
    let id = uuid::Uuid::parse_str(operation_id).map_err(|_| ServiceError::InvalidRequest)?;
    if id.to_string() != operation_id {
        return Err(ServiceError::InvalidRequest);
    }
    Ok(())
}

pub fn fingerprint(payload: &impl Serialize) -> Result<String, ServiceError> {
    let bytes = serde_json::to_vec(payload).map_err(|_| ServiceError::InvalidRequest)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn replay<T: DeserializeOwned>(
    connection: &Connection,
    principal: &Principal,
    operation_id: &str,
    fingerprint: &str,
) -> Result<Option<T>, ServiceError> {
    validate_id(operation_id)?;
    let reserved: Option<String> = connection.query_row(
        "SELECT fingerprint FROM cloud_maintenance_runs WHERE issuer=?1 AND subject=?2 AND operation_id=?3",
        params![principal.issuer, principal.subject, operation_id], |row| row.get(0),
    ).optional()?;
    if reserved.is_some_and(|stored| stored != fingerprint) {
        return Err(ServiceError::OperationConflict);
    }
    let row: Option<(String, String)> = connection.query_row(
        "SELECT fingerprint,result_json FROM operation_receipts WHERE issuer=?1 AND subject=?2 AND operation_id=?3",
        params![principal.issuer, principal.subject, operation_id], |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional()?;
    row.map(|(stored, result)| {
        if stored != fingerprint {
            return Err(ServiceError::OperationConflict);
        }
        serde_json::from_str(&result).map_err(|_| ServiceError::Storage)
    })
    .transpose()
}

pub fn commit<T: Serialize>(
    transaction: &Transaction<'_>,
    principal: &Principal,
    operation_id: &str,
    fingerprint: &str,
    result: &T,
) -> Result<(), ServiceError> {
    let result_json = serde_json::to_string(result).map_err(|_| ServiceError::Storage)?;
    transaction.execute(
        "INSERT INTO operation_receipts VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            principal.issuer,
            principal.subject,
            operation_id,
            fingerprint,
            result_json,
            now_seconds()
        ],
    )?;
    Ok(())
}

pub fn lookup(
    connection: &Connection,
    principal: &Principal,
    operation_id: &str,
) -> Result<OperationStatus, ServiceError> {
    validate_id(operation_id)?;
    let result: Option<String> = connection.query_row("SELECT result_json FROM operation_receipts WHERE issuer=?1 AND subject=?2 AND operation_id=?3", params![principal.issuer, principal.subject, operation_id], |row| row.get(0)).optional()?;
    match result {
        Some(result) => Ok(OperationStatus::Committed {
            result: serde_json::from_str(&result).map_err(|_| ServiceError::Storage)?,
        }),
        None => Ok(OperationStatus::Unknown),
    }
}
