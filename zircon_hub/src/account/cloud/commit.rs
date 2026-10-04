use super::manifest::digest;
use crate::account::{operations::validate_id, AccountError};
use openidconnect::reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum CommitOutcome {
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SnapshotReceipt {
    organization_id: String,
    project_id: String,
    base_revision: String,
    revision: String,
    manifest_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationIdConflict {
    error: String,
}

// A typed HTTP 409 CAS result is terminal; an exact operation-ID rejection is
// a separate failure, and any other shape leaves the outcome unknown.
pub(in crate::account) fn decode_http(
    status: reqwest::StatusCode,
    bytes: &[u8],
) -> Result<Value, AccountError> {
    let outcome: CommitOutcome = serde_json::from_slice(bytes).map_err(|_| {
        if status == reqwest::StatusCode::CONFLICT
            && serde_json::from_slice::<OperationIdConflict>(bytes)
                .is_ok_and(|rejection| rejection.error == "operation_id_conflict")
        {
            AccountError::OperationConflict
        } else {
            AccountError::OutcomeUnknown
        }
    })?;
    if !matches!(
        (status, &outcome),
        (reqwest::StatusCode::OK, CommitOutcome::Committed { .. })
            | (
                reqwest::StatusCode::CONFLICT,
                CommitOutcome::Conflict { .. }
            )
    ) {
        return Err(AccountError::OutcomeUnknown);
    }
    serde_json::to_value(outcome).map_err(|_| AccountError::OutcomeUnknown)
}

pub(in crate::account) fn project_receipt(
    value: &Value,
    organization: &str,
    project: &str,
    base_revision: &str,
    manifest_digest: &str,
) -> Option<Value> {
    validate_id(organization).ok()?;
    validate_id(project).ok()?;
    if !digest(manifest_digest) {
        return None;
    }
    let base = revision(base_revision)?;
    let outcome: CommitOutcome = serde_json::from_value(value.clone()).ok()?;
    let matches = match &outcome {
        CommitOutcome::Committed { snapshot } => {
            snapshot.organization_id == organization
                && snapshot.project_id == project
                && snapshot.base_revision == base_revision
                && snapshot.manifest_digest == manifest_digest
                && base.checked_add(1) == revision(&snapshot.revision)
        }
        CommitOutcome::Conflict {
            organization_id,
            project_id,
            base_revision: observed_base,
            current_revision,
            manifest_digest: observed_digest,
        } => {
            organization_id == organization
                && project_id == project
                && observed_base == base_revision
                && observed_digest == manifest_digest
                && revision(current_revision).is_some_and(|current| current != base)
        }
    };
    if !matches {
        return None;
    }
    serde_json::to_value(outcome).ok()
}

pub(in crate::account) fn is_conflict(value: &Value) -> bool {
    value.get("status").and_then(Value::as_str) == Some("conflict")
}

fn revision(value: &str) -> Option<i64> {
    let number = value.parse::<i64>().ok()?;
    (number >= 0 && number.to_string() == value).then_some(number)
}

#[cfg(test)]
#[path = "commit/tests/cases.rs"]
mod tests;
