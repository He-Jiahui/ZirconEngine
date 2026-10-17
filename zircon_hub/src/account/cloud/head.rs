use super::super::{operations::validate_id, AccountError};
use super::manifest::{digest, Manifest};
use serde::Deserialize;
use serde_json::Value;

// The service caps the canonical manifest at 8 MiB. The snapshot envelope
// contains only fixed-size IDs, digests, revisions and an author/timestamp.
pub(in crate::account) const RESPONSE_LIMIT: usize = 8 * 1024 * 1024 + 4096;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Snapshot {
    organization_id: String,
    project_id: String,
    base_revision: String,
    revision: String,
    manifest_digest: String,
    created_at: u64,
    created_by: String,
    manifest: Manifest,
}

pub(in crate::account) fn admit(
    value: Value,
    organization: &str,
    project: &str,
) -> Result<Value, AccountError> {
    if value.is_null() {
        return Ok(value);
    }
    let head: Snapshot =
        serde_json::from_value(value.clone()).map_err(|_| AccountError::ServiceFailure)?;
    validate_id(&head.organization_id)?;
    validate_id(&head.project_id)?;
    let base = revision(&head.base_revision)?;
    let current = revision(&head.revision)?;
    if head.organization_id != organization
        || head.project_id != project
        || base.checked_add(1) != Some(current)
        || !digest(&head.manifest_digest)
        || !digest(&head.created_by)
        || head.created_at == 0
        || head.manifest.require_present_package_lock().is_err()
        || head.manifest.canonical_digest()? != head.manifest_digest
    {
        return Err(AccountError::ServiceFailure);
    }
    Ok(value)
}

fn revision(value: &str) -> Result<i64, AccountError> {
    let number = value
        .parse::<i64>()
        .map_err(|_| AccountError::ServiceFailure)?;
    if number < 0 || number.to_string() != value {
        return Err(AccountError::ServiceFailure);
    }
    Ok(number)
}

#[cfg(test)]
#[path = "tests/head.rs"]
mod tests;
