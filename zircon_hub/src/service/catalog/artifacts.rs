use super::{CatalogPolicy, Release};
use crate::service::{error::ServiceError, identity::Principal, storage::receipt};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};

pub const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;
const MAX_CATALOG_BYTES: u64 = 256 * 1024 * 1024;

fn revision(value: &str) -> Result<i64, ServiceError> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| ServiceError::InvalidRequest)?;
    if parsed <= 0 || parsed.to_string() != value {
        return Err(ServiceError::InvalidRequest);
    }
    Ok(parsed)
}

fn publisher_release(
    connection: &Connection,
    principal: &Principal,
    policy: &CatalogPolicy,
    package: &str,
    revision_text: &str,
) -> Result<Release, ServiceError> {
    receipt::validate_id(package)?;
    let revision = revision(revision_text)?;
    let envelope: Option<String> = connection.query_row(
        "SELECT envelope FROM catalog_releases WHERE package_id=?1 AND revision=?2 AND publisher_issuer=?3 AND publisher_subject=?4",
        params![package, revision, principal.issuer, principal.subject], |row| row.get(0),
    ).optional()?;
    let (kid, release) = policy.verify(&envelope.ok_or(ServiceError::Forbidden)?)?;
    let publisher = &policy.publishers[&kid];
    if publisher.issuer != principal.issuer || publisher.subject != principal.subject {
        return Err(ServiceError::Forbidden);
    }
    Ok(release)
}

fn validate_capacity(stored: u64, added: u64) -> Result<(), ServiceError> {
    match stored.checked_add(added) {
        Some(total) if total <= MAX_CATALOG_BYTES => Ok(()),
        _ => Err(ServiceError::Capacity),
    }
}

fn matches(bytes: &[u8], release: &Release) -> bool {
    bytes.len() as u64 == release.artifact_size
        && format!("{:x}", Sha256::digest(bytes)) == release.artifact_digest
}

pub fn upload(
    connection: &mut Connection,
    principal: &Principal,
    policy: &CatalogPolicy,
    package: &str,
    revision: &str,
    bytes: &[u8],
) -> Result<(), ServiceError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let release = publisher_release(&transaction, principal, policy, package, revision)?;
    if release.artifact_size > MAX_ARTIFACT_BYTES as u64 || bytes.len() > MAX_ARTIFACT_BYTES {
        return Err(ServiceError::Capacity);
    }
    if !matches(bytes, &release) {
        return Err(ServiceError::InvalidRequest);
    }
    let existing: Option<Vec<u8>> = transaction
        .query_row(
            "SELECT payload FROM catalog_artifacts WHERE digest=?1 AND length(payload)<=?2",
            params![release.artifact_digest, MAX_ARTIFACT_BYTES as u64],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(existing) = existing {
        if existing != bytes {
            return Err(ServiceError::Storage);
        }
    } else {
        let stored: u64 = transaction.query_row(
            "SELECT COALESCE(SUM(length(payload)),0) FROM catalog_artifacts",
            [],
            |row| row.get(0),
        )?;
        validate_capacity(stored, bytes.len() as u64)?;
        transaction.execute(
            "INSERT INTO catalog_artifacts(digest,payload) VALUES (?1,?2)",
            params![release.artifact_digest, bytes],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

pub fn download(
    connection: &Connection,
    principal: &Principal,
    policy: &CatalogPolicy,
    organization: &str,
    package: &str,
    revision_text: &str,
) -> Result<Vec<u8>, ServiceError> {
    revision(revision_text)?;
    let release = super::authorized_artifact(
        connection,
        principal,
        policy,
        organization,
        package,
        revision_text,
    )?;
    if release.artifact_size > MAX_ARTIFACT_BYTES as u64 {
        return Err(ServiceError::Capacity);
    }
    let bytes: Option<Vec<u8>> = connection
        .query_row(
            "SELECT payload FROM catalog_artifacts WHERE digest=?1 AND length(payload)<=?2",
            params![release.artifact_digest, MAX_ARTIFACT_BYTES as u64],
            |row| row.get(0),
        )
        .optional()?;
    let bytes = bytes.ok_or(ServiceError::Storage)?;
    if !matches(&bytes, &release) {
        return Err(ServiceError::Storage);
    }
    Ok(bytes)
}

pub fn envelope(
    connection: &Connection,
    principal: &Principal,
    policy: &CatalogPolicy,
    organization: &str,
    package: &str,
    revision_text: &str,
) -> Result<String, ServiceError> {
    let revision = revision(revision_text)?;
    super::authorized_artifact(
        connection,
        principal,
        policy,
        organization,
        package,
        revision_text,
    )?;
    connection
        .query_row(
            "SELECT envelope FROM catalog_releases WHERE package_id=?1 AND revision=?2",
            params![package, revision],
            |row| row.get(0),
        )
        .map_err(ServiceError::from)
}

#[cfg(test)]
#[path = "artifacts/tests/cases.rs"]
mod tests;
