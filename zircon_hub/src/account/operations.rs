//! Non-secret mutation recovery, isolated by the authenticated service identity.

mod store;
#[cfg(test)]
#[path = "operations/tests/cases.rs"]
mod tests;
#[cfg(windows)]
mod windows;

use super::{cloud::manifest::Manifest, AccountConfig, AccountError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

pub(super) struct CredentialLock {
    _guard: store::StoreLock,
}

pub(super) fn credential_lock() -> Result<CredentialLock, AccountError> {
    store::credential_lock()
        .map(|guard| CredentialLock { _guard: guard })
        .map_err(|_| AccountError::CredentialStore)
}

pub(super) const MAX_REQUEST_BYTES: usize = 65536;
// Matches the service cloud route's 8 MiB manifest plus request envelope cap.
pub(super) const MAX_CLOUD_COMMIT_REQUEST_BYTES: usize = 8 * 1024 * 1024 + 65536;
const MAX_RECORDS: usize = 128;
// A maximum service-valid cloud manifest fits; additional pending records
// fail admission instead of evicting an unknown or terminal operation.
const MAX_JOURNAL_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct OperationIdentity {
    issuer: String,
    subject: String,
    client_id: String,
    service_url: String,
}

impl OperationIdentity {
    pub(super) fn new(config: &AccountConfig, subject: &str) -> Self {
        Self {
            issuer: config.issuer.clone(),
            subject: subject.to_owned(),
            client_id: config.client_id.clone(),
            service_url: config.service_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationStatus {
    Unknown,
    Committed,
    Failed,
    Conflict,
    OperationIdConflict,
}

impl OperationStatus {
    pub(crate) fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Committed | Self::Failed | Self::Conflict | Self::OperationIdConflict
        )
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationSummary {
    pub operation_id: String,
    pub status: OperationStatus,
    pub action: String,
    pub organization_id: Option<String>,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_mode: Option<super::package::PackageRuntimeMode>,
}

/// Broker-owned cleanup authority for one journaled CloudCommit.
///
/// This internal projection keeps the exact project and uncollapsed terminal status without
/// changing the public recovery DTO or the on-disk journal schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CloudCommitCleanupRecord {
    pub(crate) operation_id: String,
    pub(crate) organization_id: String,
    pub(crate) project_id: String,
    pub(crate) status: OperationStatus,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum Mutation {
    CreateProject {
        name: String,
    },
    SetMember {
        issuer: String,
        subject: String,
        role: String,
        active: bool,
    },
    TransferOwnership {
        issuer: String,
        subject: String,
    },
    Invite {
        issuer: String,
        subject: String,
        role: String,
        expires_at: u64,
    },
    AcceptInvite {
        invitation_id: String,
    },
    RevokeInvite {
        invitation_id: String,
    },
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "action",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(super) enum OperationPayload {
    CreateOrganization {
        name: String,
    },
    Mutate {
        organization: String,
        expected_policy_revision: String,
        mutation: Mutation,
    },
    CatalogLicense {
        organization: String,
        expected_policy_revision: String,
        package_id: String,
        revision: String,
        license_id: String,
    },
    InstallPackage {
        organization: String,
        owner: String,
        request: super::package::InstallRequest,
    },
    CloudCommit {
        organization: String,
        project: String,
        base_revision: String,
        manifest: Manifest,
    },
}

impl OperationPayload {
    pub(super) fn validate(&self, operation_id: &str) -> Result<(), AccountError> {
        if let Self::InstallPackage {
            organization,
            owner,
            request,
        } = self
        {
            validate_id(organization)?;
            request.validate()?;
            if request.operation_id != operation_id || !super::package::is_digest(owner) {
                return Err(AccountError::ServiceFailure);
            }
            Ok(())
        } else {
            self.request(operation_id).map(|_| ())
        }
    }

    pub(super) fn request(&self, operation_id: &str) -> Result<(String, Vec<u8>), AccountError> {
        validate_id(operation_id)?;
        let (path, value) = match self {
            Self::InstallPackage { .. } => return Err(AccountError::ServiceFailure),
            Self::CloudCommit {
                organization,
                project,
                base_revision,
                manifest,
            } => {
                validate_id(organization)?;
                validate_id(project)?;
                validate_revision(base_revision, true)?;
                manifest.require_present_package_lock()?;
                manifest.canonical_digest()?;
                (
                    format!("/v1/organizations/{organization}/projects/{project}/cloud/commit"),
                    serde_json::json!({
                        "operationId": operation_id,
                        "baseRevision": base_revision,
                        "manifest": manifest
                    }),
                )
            }
            Self::CatalogLicense {
                organization,
                expected_policy_revision,
                package_id,
                revision,
                license_id,
            } => {
                validate_id(organization)?;
                validate_id(package_id)?;
                validate_revision(expected_policy_revision, false)?;
                validate_revision(revision, false)?;
                if license_id.is_empty() || license_id.len() > 128 {
                    return Err(AccountError::ServiceFailure);
                }
                (
                    format!("/v1/organizations/{organization}/licenses"),
                    serde_json::json!({"operationId":operation_id,"expectedPolicyRevision":expected_policy_revision,"packageId":package_id,"revision":revision,"licenseId":license_id}),
                )
            }
            Self::CreateOrganization { name } => (
                "/v1/organizations".into(),
                serde_json::json!({"operationId":operation_id,"name":name}),
            ),
            Self::Mutate {
                organization,
                expected_policy_revision,
                mutation,
            } => {
                validate_id(organization)?;
                let revision = expected_policy_revision
                    .parse::<i64>()
                    .map_err(|_| AccountError::ServiceFailure)?;
                if revision <= 0 || revision.to_string() != *expected_policy_revision {
                    return Err(AccountError::ServiceFailure);
                }
                (
                    format!("/v1/organizations/{organization}/mutations"),
                    serde_json::json!({"operationId":operation_id,"expectedPolicyRevision":expected_policy_revision,"mutation":mutation}),
                )
            }
        };
        let bytes =
            serde_json::to_vec(&canonicalize(value)).map_err(|_| AccountError::ServiceFailure)?;
        let limit = if matches!(self, Self::CloudCommit { .. }) {
            MAX_CLOUD_COMMIT_REQUEST_BYTES
        } else {
            MAX_REQUEST_BYTES
        };
        if bytes.len() > limit {
            return Err(AccountError::ServiceFailure);
        }
        Ok((path, bytes))
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct OperationRecord {
    identity: OperationIdentity,
    pub(super) operation_id: String,
    pub(super) payload: OperationPayload,
    generation: String,
    attempts: u64,
    status: OperationStatus,
}

impl OperationRecord {
    fn summary(&self) -> OperationSummary {
        let (action, organization_id) = match &self.payload {
            OperationPayload::CreateOrganization { .. } => ("create-organization", None),
            OperationPayload::Mutate { organization, .. } => ("mutate", Some(organization.clone())),
            OperationPayload::CatalogLicense { organization, .. } => {
                ("catalog-license", Some(organization.clone()))
            }
            OperationPayload::InstallPackage { organization, .. } => {
                ("package-install", Some(organization.clone()))
            }
            OperationPayload::CloudCommit { organization, .. } => {
                ("cloud-commit", Some(organization.clone()))
            }
        };
        let target_mode = match &self.payload {
            OperationPayload::InstallPackage { request, .. } => request.target_mode(),
            _ => None,
        };
        OperationSummary {
            operation_id: self.operation_id.clone(),
            // The legacy account snapshot exposes a terminal conflict as a
            // failed operation until the cloud UI has its own typed outcome.
            status: if matches!(
                self.status,
                OperationStatus::Conflict | OperationStatus::OperationIdConflict
            ) {
                OperationStatus::Failed
            } else {
                self.status
            },
            action: action.into(),
            organization_id,
            target_mode,
            error: match self.status {
                OperationStatus::Unknown => Some(AccountError::OutcomeUnknown.to_string()),
                OperationStatus::Failed => Some(AccountError::ServiceFailure.to_string()),
                OperationStatus::Committed => None,
                OperationStatus::Conflict => Some("account_cloud_conflict".into()),
                OperationStatus::OperationIdConflict => {
                    Some(AccountError::OperationConflict.to_string())
                }
            },
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalFile {
    version: u32,
    #[serde(default)]
    revisions: Vec<IdentityRevision>,
    #[serde(default, rename = "revision", skip_serializing)]
    legacy_revision: Option<String>,
    operations: Vec<OperationRecord>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityRevision {
    identity: OperationIdentity,
    revision: String,
}

impl JournalFile {
    fn revision(&self, identity: &OperationIdentity) -> &str {
        self.revisions
            .iter()
            .find(|entry| entry.identity == *identity)
            .map_or("0", |entry| entry.revision.as_str())
    }

    fn advance_revision(&mut self, identity: &OperationIdentity) -> Result<(), AccountError> {
        if let Some(entry) = self
            .revisions
            .iter_mut()
            .find(|entry| entry.identity == *identity)
        {
            entry.revision = entry
                .revision
                .parse::<u64>()
                .ok()
                .and_then(|revision| revision.checked_add(1))
                .ok_or(AccountError::OperationStore)?
                .to_string();
        } else {
            self.revisions.push(IdentityRevision {
                identity: identity.clone(),
                revision: "1".into(),
            });
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(super) struct OperationJournal {
    path: PathBuf,
}

impl OperationJournal {
    pub(super) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[cfg(test)]
    pub(super) fn list(
        &self,
        identity: &OperationIdentity,
    ) -> Result<Vec<OperationSummary>, AccountError> {
        self.access(false, |file| {
            Ok(file
                .operations
                .iter()
                .filter(|record| record.identity == *identity)
                .map(OperationRecord::summary)
                .collect())
        })
    }

    pub(super) fn snapshot(
        &self,
        identity: &OperationIdentity,
    ) -> Result<(Vec<OperationSummary>, String), AccountError> {
        self.access(false, |file| {
            Ok((
                file.operations
                    .iter()
                    .filter(|record| record.identity == *identity)
                    .map(OperationRecord::summary)
                    .collect(),
                file.revision(identity).to_owned(),
            ))
        })
    }

    pub(super) fn cloud_cleanup_snapshot(
        &self,
        identity: &OperationIdentity,
    ) -> Result<(Vec<OperationSummary>, Vec<CloudCommitCleanupRecord>, String), AccountError> {
        self.access(false, |file| {
            let records = file
                .operations
                .iter()
                .filter(|record| record.identity == *identity);
            let mut operations = Vec::new();
            let mut cloud_commits = Vec::new();
            for record in records {
                operations.push(record.summary());
                if let OperationPayload::CloudCommit {
                    organization,
                    project,
                    ..
                } = &record.payload
                {
                    cloud_commits.push(CloudCommitCleanupRecord {
                        operation_id: record.operation_id.clone(),
                        organization_id: organization.clone(),
                        project_id: project.clone(),
                        status: record.status,
                    });
                }
            }
            Ok((
                operations,
                cloud_commits,
                file.revision(identity).to_owned(),
            ))
        })
    }

    pub(super) fn get(
        &self,
        identity: &OperationIdentity,
        operation_id: &str,
    ) -> Result<OperationRecord, AccountError> {
        validate_id(operation_id)?;
        self.access(false, |file| {
            file.operations
                .iter()
                .find(|record| record.identity == *identity && record.operation_id == operation_id)
                .cloned()
                .ok_or(AccountError::ServiceFailure)
        })
    }

    /// The unknown record is durable before the caller is allowed to send HTTP.
    pub(super) fn admit(
        &self,
        identity: &OperationIdentity,
        operation_id: &str,
        generation: &str,
        payload: OperationPayload,
    ) -> Result<bool, AccountError> {
        self.admit_inner(identity, operation_id, generation, payload, false)
    }

    pub(super) fn admit_retry(
        &self,
        identity: &OperationIdentity,
        operation_id: &str,
        generation: &str,
        payload: OperationPayload,
    ) -> Result<bool, AccountError> {
        self.admit_inner(identity, operation_id, generation, payload, true)
    }

    fn admit_inner(
        &self,
        identity: &OperationIdentity,
        operation_id: &str,
        generation: &str,
        payload: OperationPayload,
        require_existing: bool,
    ) -> Result<bool, AccountError> {
        payload.validate(operation_id)?;
        self.access(true, |file| {
            if let Some(existing) = file
                .operations
                .iter_mut()
                .find(|record| record.identity == *identity && record.operation_id == operation_id)
            {
                if existing.payload != payload
                    || matches!(
                        existing.status,
                        OperationStatus::Conflict | OperationStatus::OperationIdConflict
                    )
                    || (matches!(&existing.payload, OperationPayload::CloudCommit { .. })
                        && existing.status == OperationStatus::Committed)
                    || (require_existing && existing.status == OperationStatus::Committed)
                {
                    return Err(AccountError::OperationConflict);
                }
                existing.attempts = existing
                    .attempts
                    .checked_add(1)
                    .ok_or(AccountError::OperationStore)?;
                if existing.status != OperationStatus::Committed {
                    existing.status = OperationStatus::Unknown;
                }
                if matches!(&existing.payload, OperationPayload::CloudCommit { .. }) {
                    file.version = 4;
                }
                file.advance_revision(identity)?;
                return Ok(false);
            }
            // Another process may acknowledge after recovery lookup but before admission.
            if require_existing {
                return Err(AccountError::ServiceFailure);
            }
            if file.operations.iter().any(|record| {
                record.identity == *identity && record.status == OperationStatus::Unknown
            }) {
                return Err(AccountError::Busy);
            }
            if file.operations.len() >= MAX_RECORDS {
                return Err(AccountError::OperationStore);
            }
            file.operations.push(OperationRecord {
                identity: identity.clone(),
                operation_id: operation_id.into(),
                payload,
                generation: generation.into(),
                attempts: 1,
                status: OperationStatus::Unknown,
            });
            if matches!(
                file.operations.last().map(|record| &record.payload),
                Some(OperationPayload::CloudCommit { .. })
            ) {
                file.version = 4;
            }
            file.advance_revision(identity)?;
            Ok(true)
        })
    }

    pub(super) fn finish(
        &self,
        identity: &OperationIdentity,
        operation_id: &str,
        status: OperationStatus,
    ) -> Result<(), AccountError> {
        self.access(true, |file| {
            let record = file
                .operations
                .iter_mut()
                .find(|record| record.identity == *identity && record.operation_id == operation_id)
                .ok_or(AccountError::ServiceFailure)?;
            if matches!(
                status,
                OperationStatus::Conflict | OperationStatus::OperationIdConflict
            ) && !matches!(&record.payload, OperationPayload::CloudCommit { .. })
            {
                return Err(AccountError::ServiceFailure);
            }
            // A delayed failure or absent receipt cannot erase a terminal result.
            if matches!(
                record.status,
                OperationStatus::Committed
                    | OperationStatus::Conflict
                    | OperationStatus::OperationIdConflict
            ) {
                if matches!(
                    status,
                    OperationStatus::Committed
                        | OperationStatus::Conflict
                        | OperationStatus::OperationIdConflict
                ) && status != record.status
                {
                    return Err(AccountError::OperationConflict);
                }
                return Ok(());
            }
            record.status = if status == OperationStatus::Failed && record.attempts > 1 {
                OperationStatus::Unknown
            } else {
                status
            };
            if status == OperationStatus::OperationIdConflict {
                file.version = 4;
            }
            file.advance_revision(identity)?;
            Ok(())
        })
    }

    pub(super) fn acknowledge(
        &self,
        identity: &OperationIdentity,
        operation_id: &str,
    ) -> Result<(), AccountError> {
        validate_id(operation_id)?;
        self.access(true, |file| {
            let Some(index) = file.operations.iter().position(|record| {
                record.identity == *identity && record.operation_id == operation_id
            }) else {
                return Err(AccountError::ServiceFailure);
            };
            if file.operations[index].status == OperationStatus::Unknown {
                return Err(AccountError::OutcomeUnknown);
            }
            file.operations.remove(index);
            file.advance_revision(identity)?;
            Ok(())
        })
    }
}

pub(super) fn validate_id(value: &str) -> Result<(), AccountError> {
    if value.len() != 36
        || value.bytes().enumerate().any(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte != b'-'
            } else {
                !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte)
            }
        })
    {
        return Err(AccountError::ServiceFailure);
    }
    Ok(())
}

pub(super) fn validate_revision(value: &str, zero: bool) -> Result<(), AccountError> {
    if !value
        .parse::<i64>()
        .is_ok_and(|number| (number > 0 || (zero && number == 0)) && number.to_string() == value)
    {
        return Err(AccountError::ServiceFailure);
    }
    Ok(())
}

fn canonicalize(value: Value) -> Value {
    match value {
        Value::Object(values) => {
            let values: std::collections::BTreeMap<_, _> = values
                .into_iter()
                .map(|(key, value)| (key, canonicalize(value)))
                .collect();
            Value::Object(values.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize).collect()),
        value => value,
    }
}
