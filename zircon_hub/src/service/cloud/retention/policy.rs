use super::super::{authorization::authorize, quota::MAX_REVISIONS};
use crate::service::{
    error::ServiceError,
    identity::{now_seconds, Principal},
    organization,
    storage::receipt,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

const MAX_TRASH_SECONDS: u64 = 30 * 86_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetentionPolicy {
    pub keep_latest: Option<u64>,
    pub trash_seconds: u64,
    pub legal_hold: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            keep_latest: None,
            trash_seconds: 86_400,
            legal_hold: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionState {
    pub organization_id: String,
    pub project_id: String,
    pub revision: String,
    pub policy: RetentionPolicy,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetentionRequest {
    pub operation_id: String,
    pub expected_revision: String,
    pub policy: RetentionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RetentionOutcome {
    RetentionUpdated {
        retention: RetentionState,
    },
    RetentionConflict {
        organization_id: String,
        project_id: String,
        expected_revision: String,
        current_revision: String,
    },
}

pub fn get(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    project: &str,
) -> Result<RetentionState, ServiceError> {
    authorize(connection, principal, organization, project, false)?;
    load(connection, organization, project)
}

pub(in crate::service::cloud) fn load(
    connection: &Connection,
    organization: &str,
    project: &str,
) -> Result<RetentionState, ServiceError> {
    let row = connection
        .query_row(
            "SELECT revision,keep_latest,trash_seconds,legal_hold FROM cloud_retention_policies WHERE organization_id=?1 AND project_id=?2",
            params![organization, project],
            |row| Ok((row.get::<_, i64>(0)?, RetentionPolicy {
                keep_latest: row.get(1)?, trash_seconds: row.get(2)?, legal_hold: row.get(3)?,
            })),
        )
        .optional()?;
    let (revision, policy) = row.unwrap_or((0, RetentionPolicy::default()));
    Ok(RetentionState {
        organization_id: organization.into(),
        project_id: project.into(),
        revision: revision.to_string(),
        policy,
    })
}

pub(in crate::service::cloud) fn authorize_admin(
    connection: &Connection,
    principal: &Principal,
    organization: &str,
    project: &str,
) -> Result<i64, ServiceError> {
    let revision = authorize(connection, principal, organization, project, true)?;
    let allowed: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM memberships WHERE organization_id=?1 AND issuer=?2 AND subject=?3 AND active=1 AND role IN ('owner','admin'))",
        params![organization, principal.issuer, principal.subject],
        |row| row.get(0),
    )?;
    if !allowed {
        return Err(ServiceError::Forbidden);
    }
    Ok(revision)
}

pub(in crate::service::cloud) fn authorize_receipt(
    connection: &Connection,
    principal: &Principal,
    result: &serde_json::Value,
) -> Result<(), ServiceError> {
    if !matches!(
        result.get("status").and_then(serde_json::Value::as_str),
        Some("retentionUpdated" | "retentionConflict")
    ) {
        return super::maintenance::authorize_receipt(connection, principal, result);
    }
    let outcome: RetentionOutcome =
        serde_json::from_value(result.clone()).map_err(|_| ServiceError::Storage)?;
    let (organization, project) = match &outcome {
        RetentionOutcome::RetentionUpdated { retention } => {
            (&retention.organization_id, &retention.project_id)
        }
        RetentionOutcome::RetentionConflict {
            organization_id,
            project_id,
            ..
        } => (organization_id, project_id),
    };
    authorize(connection, principal, organization, project, false)?;
    Ok(())
}

pub fn update(
    connection: &mut Connection,
    principal: &Principal,
    organization: &str,
    project: &str,
    request: RetentionRequest,
) -> Result<RetentionOutcome, ServiceError> {
    receipt::validate_id(&request.operation_id)?;
    let expected = request
        .expected_revision
        .parse::<i64>()
        .map_err(|_| ServiceError::InvalidRequest)?;
    if expected < 0
        || expected.to_string() != request.expected_revision
        || request
            .policy
            .keep_latest
            .is_some_and(|keep| keep == 0 || keep > MAX_REVISIONS)
        || request.policy.trash_seconds > MAX_TRASH_SECONDS
    {
        return Err(ServiceError::InvalidRequest);
    }
    let fingerprint = receipt::fingerprint(&(
        "cloud.retention",
        organization,
        project,
        &request.expected_revision,
        &request.policy,
    ))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let organization_revision = authorize_admin(&transaction, principal, organization, project)?;
    if let Some(outcome) =
        receipt::replay(&transaction, principal, &request.operation_id, &fingerprint)?
    {
        return Ok(outcome);
    }
    let current = load(&transaction, organization, project)?;
    let outcome = if current.revision == request.expected_revision {
        let revision = expected.checked_add(1).ok_or(ServiceError::Capacity)?;
        transaction.execute(
            "INSERT INTO cloud_retention_policies VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(organization_id,project_id) DO UPDATE SET revision=excluded.revision,keep_latest=excluded.keep_latest,trash_seconds=excluded.trash_seconds,legal_hold=excluded.legal_hold",
            params![organization, project, revision, request.policy.keep_latest, request.policy.trash_seconds, request.policy.legal_hold],
        )?;
        super::apply(&transaction, organization, project, now_seconds())?;
        RetentionOutcome::RetentionUpdated {
            retention: load(&transaction, organization, project)?,
        }
    } else {
        RetentionOutcome::RetentionConflict {
            organization_id: organization.into(),
            project_id: project.into(),
            expected_revision: request.expected_revision,
            current_revision: current.revision,
        }
    };
    organization::audit(
        &transaction,
        organization,
        principal,
        "cloud.retention",
        if matches!(&outcome, RetentionOutcome::RetentionUpdated { .. }) {
            "allowed"
        } else {
            "conflict"
        },
        organization_revision,
    )?;
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
