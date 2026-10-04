use super::{
    cloud, oidc,
    operations::{validate_id, Mutation, OperationIdentity, OperationPayload, OperationStatus},
    AccountBroker, AccountError,
};
use openidconnect::{reqwest, OAuth2TokenResponse};
use serde::Deserialize;
use serde_json::Value;

const SERVICE_RESPONSE_LIMIT: usize = 262144;

#[derive(Clone, Deserialize)]
#[serde(
    tag = "action",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ServiceRequest {
    CloudHead {
        organization: String,
        project: String,
    },
    CloudCommit {
        organization: String,
        project: String,
        operation_id: String,
        base_revision: String,
        manifest: Value,
    },
    Catalog {
        after: Option<String>,
        query: Option<String>,
    },
    CatalogEntitlements {
        organization: String,
        after: Option<String>,
    },
    CatalogArtifact {
        organization: String,
        package_id: String,
        revision: String,
    },
    CatalogLicense {
        organization: String,
        operation_id: String,
        expected_policy_revision: String,
        package_id: String,
        revision: String,
        license_id: String,
    },
    Organizations {
        after: Option<String>,
    },
    Invitations {
        after: Option<String>,
    },
    IssuedInvitations {
        organization: String,
        after: Option<String>,
    },
    Members {
        organization: String,
        after: Option<String>,
    },
    Projects {
        organization: String,
        after: Option<String>,
    },
    CreateOrganization {
        operation_id: String,
        name: String,
    },
    Mutate {
        organization: String,
        operation_id: String,
        expected_policy_revision: String,
        mutation: Value,
    },
    Receipt {
        operation_id: String,
    },
}

struct PreparedRequest {
    path: String,
    after: Option<String>,
    query: Option<String>,
    body: Option<Vec<u8>>,
    operation: Option<(String, OperationPayload)>,
    cloud_scope: Option<(String, String)>,
}

impl ServiceRequest {
    fn parts(self) -> Result<PreparedRequest, AccountError> {
        let mut search = None;
        let cloud_scope = match &self {
            Self::CloudHead {
                organization,
                project,
            } => Some((organization.clone(), project.clone())),
            _ => None,
        };
        let (path, after, operation) = match self {
            Self::CloudHead {
                organization,
                project,
            } => {
                validate_id(&organization)?;
                validate_id(&project)?;
                (
                    format!("/v1/organizations/{organization}/projects/{project}/cloud/head"),
                    None,
                    None,
                )
            }
            Self::CloudCommit {
                organization,
                project,
                operation_id,
                base_revision,
                manifest,
            } => {
                let manifest: cloud::manifest::Manifest =
                    serde_json::from_value(manifest).map_err(|_| AccountError::ServiceFailure)?;
                (
                    String::new(),
                    None,
                    Some((
                        operation_id,
                        OperationPayload::CloudCommit {
                            organization,
                            project,
                            base_revision,
                            manifest: manifest.normalized()?,
                        },
                    )),
                )
            }
            Self::Catalog { after, query } => {
                if let Some(after) = &after {
                    validate_id(after)?;
                }
                if query.as_ref().is_some_and(|query| query.len() > 256) {
                    return Err(AccountError::ServiceFailure);
                }
                search = query;
                ("/v1/catalog".into(), after, None)
            }
            Self::CatalogEntitlements {
                organization,
                after,
            } => {
                validate_id(&organization)?;
                if let Some(after) = &after {
                    super::operations::validate_revision(after, true)?;
                }
                (
                    format!("/v1/organizations/{organization}/licenses"),
                    after,
                    None,
                )
            }
            Self::CatalogArtifact {
                organization,
                package_id,
                revision,
            } => {
                let path = super::catalog::artifact_path(&organization, &package_id, &revision)?;
                (path, None, None)
            }
            Self::CatalogLicense {
                organization,
                operation_id,
                expected_policy_revision,
                package_id,
                revision,
                license_id,
            } => (
                String::new(),
                None,
                Some((
                    operation_id,
                    OperationPayload::CatalogLicense {
                        organization,
                        expected_policy_revision,
                        package_id,
                        revision,
                        license_id,
                    },
                )),
            ),
            Self::Organizations { after } => ("/v1/organizations".into(), after, None),
            Self::Invitations { after } => ("/v1/invitations".into(), after, None),
            Self::IssuedInvitations {
                organization,
                after,
            } => {
                validate_id(&organization)?;
                (
                    format!("/v1/organizations/{organization}/invitations"),
                    after,
                    None,
                )
            }
            Self::Members {
                organization,
                after,
            } => {
                validate_id(&organization)?;
                (
                    format!("/v1/organizations/{organization}/members"),
                    after,
                    None,
                )
            }
            Self::Projects {
                organization,
                after,
            } => {
                validate_id(&organization)?;
                (
                    format!("/v1/organizations/{organization}/projects"),
                    after,
                    None,
                )
            }
            Self::CreateOrganization { operation_id, name } => (
                String::new(),
                None,
                Some((operation_id, OperationPayload::CreateOrganization { name })),
            ),
            Self::Mutate {
                organization,
                operation_id,
                expected_policy_revision,
                mutation,
            } => {
                let mutation: Mutation =
                    serde_json::from_value(mutation).map_err(|_| AccountError::ServiceFailure)?;
                (
                    String::new(),
                    None,
                    Some((
                        operation_id,
                        OperationPayload::Mutate {
                            organization,
                            expected_policy_revision,
                            mutation,
                        },
                    )),
                )
            }
            Self::Receipt { operation_id } => {
                validate_id(&operation_id)?;
                (format!("/v1/operations/{operation_id}"), None, None)
            }
        };
        let (path, body) = match &operation {
            Some((operation_id, payload)) => {
                let (path, bytes) = payload.request(operation_id)?;
                (path, Some(bytes))
            }
            None => (path, None),
        };
        Ok(PreparedRequest {
            path,
            after,
            query: search,
            body,
            operation,
            cloud_scope,
        })
    }
}

impl AccountBroker {
    pub async fn service_request(
        &self,
        expected_generation: &str,
        request: ServiceRequest,
    ) -> Result<Value, AccountError> {
        let prepared = request.parts()?;
        let mutation = prepared.operation.is_some();
        let _mutation = if mutation {
            Some(self.mutation.try_lock().map_err(|_| AccountError::Busy)?)
        } else {
            None
        };
        self.perform_service_request(expected_generation, prepared, false)
            .await
    }

    pub(super) async fn service_request_with_permit(
        &self,
        expected_generation: &str,
        request: ServiceRequest,
        _permit: &tokio::sync::MutexGuard<'_, ()>,
    ) -> Result<Value, AccountError> {
        self.perform_service_request(expected_generation, request.parts()?, true)
            .await
    }

    async fn perform_service_request(
        &self,
        expected_generation: &str,
        prepared: PreparedRequest,
        recovery: bool,
    ) -> Result<Value, AccountError> {
        let mutation = prepared.operation.is_some();
        let cloud_commit = prepared
            .operation
            .as_ref()
            .is_some_and(|(_, payload)| matches!(payload, OperationPayload::CloudCommit { .. }));
        let cloud_scope = prepared.cloud_scope.clone();
        let (token, generation, identity) = {
            let state = self.state.lock().await;
            if state.view.generation != expected_generation {
                return Err(AccountError::Cancelled);
            }
            let authenticated = state
                .authenticated
                .as_ref()
                .ok_or(AccountError::SessionExpired)?;
            (
                authenticated.tokens.access_token().clone(),
                state.generation,
                OperationIdentity::new(&self.config, &authenticated.subject),
            )
        };
        let mut url = self.config.endpoint(&format!(
            "{}{}",
            self.config.service_url.trim_end_matches('/'),
            prepared.path
        ))?;
        if let Some(after) = prepared.after {
            url.query_pairs_mut().append_pair("after", &after);
        }
        if let Some(query) = prepared.query {
            url.query_pairs_mut().append_pair("query", &query);
        }
        let http = oidc::http()?;
        let builder = match prepared.body {
            Some(bytes) => http
                .post(url)
                .header("Content-Type", "application/json")
                .body(bytes),
            None => http.get(url),
        };
        let request = builder
            .bearer_auth(token.secret())
            .build()
            .map_err(|_| AccountError::ServiceFailure)?;
        let journal = prepared
            .operation
            .as_ref()
            .map(|_| self.operation_journal())
            .transpose()?;
        let first_attempt = if let Some((operation_id, payload)) = &prepared.operation {
            let journal = journal
                .as_ref()
                .ok_or(AccountError::OperationStore)?
                .clone();
            let identity = identity.clone();
            let operation_id = operation_id.clone();
            let payload = payload.clone();
            let generation = generation.to_string();
            tokio::task::spawn_blocking(move || {
                if recovery {
                    journal.admit_retry(&identity, &operation_id, &generation, payload)
                } else {
                    journal.admit(&identity, &operation_id, &generation, payload)
                }
            })
            .await
            .map_err(|_| AccountError::OperationStore)??
        } else {
            false
        };
        {
            let state = self.state.lock().await;
            if state.generation != generation || state.authenticated.is_none() {
                return Err(if mutation {
                    AccountError::OutcomeUnknown
                } else {
                    AccountError::Cancelled
                });
            }
        }
        let limit = if cloud_scope.is_some() {
            cloud::head::RESPONSE_LIMIT
        } else {
            SERVICE_RESPONSE_LIMIT
        };
        let mut result = send_service_request(&http, request, mutation, cloud_commit, limit).await;
        if let Some((organization, project)) = &cloud_scope {
            result = result.and_then(|value| cloud::head::admit(value, organization, project));
        }
        if let Some((operation_id, payload)) = &prepared.operation {
            result = result.and_then(|value| {
                project_mutation_result(payload, &value).ok_or(AccountError::OutcomeUnknown)
            });
            // A generic rejection on retry cannot disprove a previous commit.
            // An exact operation-ID conflict identifies a different stored payload.
            if !first_attempt && matches!(&result, Err(AccountError::ServiceFailure)) {
                result = Err(AccountError::OutcomeUnknown);
            }
            let status = mutation_status(&result, cloud_commit, first_attempt);
            let journal = journal
                .as_ref()
                .ok_or(AccountError::OutcomeUnknown)?
                .clone();
            let identity = identity.clone();
            let operation_id = operation_id.clone();
            if tokio::task::spawn_blocking(move || journal.finish(&identity, &operation_id, status))
                .await
                .map_err(|_| AccountError::OutcomeUnknown)?
                .is_err()
            {
                return Err(AccountError::OutcomeUnknown);
            }
        }
        let state = self.state.lock().await;
        if state.generation != generation || state.authenticated.is_none() {
            return Err(if mutation {
                AccountError::OutcomeUnknown
            } else {
                AccountError::Cancelled
            });
        }
        result
    }
}

fn mutation_status(
    result: &Result<Value, AccountError>,
    cloud_commit: bool,
    first_attempt: bool,
) -> OperationStatus {
    match result {
        Ok(value) if cloud_commit && cloud::commit::is_conflict(value) => OperationStatus::Conflict,
        Ok(_) => OperationStatus::Committed,
        Err(AccountError::OperationConflict) if cloud_commit => {
            OperationStatus::OperationIdConflict
        }
        Err(AccountError::ServiceFailure) if first_attempt => OperationStatus::Failed,
        _ => OperationStatus::Unknown,
    }
}

async fn send_service_request(
    http: &reqwest::Client,
    request: reqwest::Request,
    mutation: bool,
    cloud_commit: bool,
    response_limit: usize,
) -> Result<Value, AccountError> {
    let response = http.execute(request).await.map_err(|_| {
        if mutation {
            AccountError::OutcomeUnknown
        } else {
            AccountError::ProviderUnavailable
        }
    })?;
    let status = response.status();
    if cloud_commit {
        if status == reqwest::StatusCode::OK || status == reqwest::StatusCode::CONFLICT {
            let bytes = read_service_body(response, true, response_limit).await?;
            return cloud::commit::decode_http(status, &bytes);
        }
        if status.is_success() {
            return Err(AccountError::OutcomeUnknown);
        }
    }
    if !status.is_success() {
        if !mutation {
            return Err(AccountError::ServiceFailure);
        }
        let bytes = read_service_body(response, true, SERVICE_RESPONSE_LIMIT).await?;
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Rejection {
            error: String,
        }
        let rejected = serde_json::from_slice::<Rejection>(&bytes).is_ok_and(|body| {
            matches!(
                (status.as_u16(), body.error.as_str()),
                (400, "invalid_request")
                    | (401, "unauthorized")
                    | (403, "forbidden")
                    | (409, "policy_conflict")
                    | (429, "capacity_exceeded")
            )
        });
        return Err(if rejected {
            AccountError::ServiceFailure
        } else {
            AccountError::OutcomeUnknown
        });
    }
    let bytes = read_service_body(response, mutation, response_limit).await?;
    serde_json::from_slice(&bytes).map_err(|_| {
        if mutation {
            AccountError::OutcomeUnknown
        } else {
            AccountError::ServiceFailure
        }
    })
}

pub(super) fn valid_mutation_result(payload: &OperationPayload, result: &Value) -> bool {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Organization {
        id: String,
        name: String,
        policy_revision: String,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Receipt {
        organization_id: String,
        resource_id: String,
        policy_revision: String,
        audit_sequence: i64,
    }
    let revision = |value: &str| {
        value
            .parse::<i64>()
            .is_ok_and(|number| number > 0 && number.to_string() == value)
    };
    match payload {
        OperationPayload::CloudCommit {
            organization,
            project,
            base_revision,
            manifest,
        } => manifest.canonical_digest().ok().is_some_and(|digest| {
            cloud::commit::project_receipt(result, organization, project, base_revision, &digest)
                .is_some()
        }),
        OperationPayload::InstallPackage { request, .. } => {
            super::package::project_receipt(request, result).is_some()
        }
        OperationPayload::CatalogLicense {
            package_id,
            revision,
            ..
        } => {
            result.get("packageId").and_then(Value::as_str) == Some(package_id.as_str())
                && result.get("revision").and_then(Value::as_str) == Some(revision.as_str())
        }
        OperationPayload::CreateOrganization { name } => {
            serde_json::from_value::<Organization>(result.clone()).is_ok_and(|result| {
                validate_id(&result.id).is_ok()
                    && result.name == *name
                    && revision(&result.policy_revision)
            })
        }
        OperationPayload::Mutate {
            organization,
            mutation,
            ..
        } => serde_json::from_value::<Receipt>(result.clone()).is_ok_and(|result| {
            let resource_matches = match mutation {
                Mutation::SetMember { subject, .. } => result.resource_id == *subject,
                Mutation::TransferOwnership { issuer, subject } => {
                    result.resource_id == format!("{issuer}:{subject}")
                }
                Mutation::AcceptInvite { invitation_id }
                | Mutation::RevokeInvite { invitation_id } => result.resource_id == *invitation_id,
                _ => validate_id(&result.resource_id).is_ok(),
            };
            result.organization_id == *organization
                && resource_matches
                && revision(&result.policy_revision)
                && result.audit_sequence > 0
        }),
    }
}

pub(super) fn project_mutation_result(payload: &OperationPayload, result: &Value) -> Option<Value> {
    if let OperationPayload::CloudCommit {
        organization,
        project,
        base_revision,
        manifest,
    } = payload
    {
        let digest = manifest.canonical_digest().ok()?;
        return cloud::commit::project_receipt(
            result,
            organization,
            project,
            base_revision,
            &digest,
        );
    }
    if !valid_mutation_result(payload, result) {
        return None;
    }
    Some(match payload {
        OperationPayload::InstallPackage { request, .. } => {
            return super::package::project_receipt(request, result)
        }
        OperationPayload::CatalogLicense { .. } => {
            serde_json::json!({"packageId":result["packageId"],"revision":result["revision"]})
        }
        OperationPayload::CreateOrganization { .. } => {
            serde_json::json!({"id":result["id"],"name":result["name"],"policyRevision":result["policyRevision"]})
        }
        OperationPayload::Mutate { .. } => {
            serde_json::json!({"organizationId":result["organizationId"],"resourceId":result["resourceId"],"policyRevision":result["policyRevision"],"auditSequence":result["auditSequence"]})
        }
        OperationPayload::CloudCommit { .. } => unreachable!("handled above"),
    })
}

pub(super) async fn read_service_body(
    mut response: reqwest::Response,
    mutation: bool,
    limit: usize,
) -> Result<Vec<u8>, AccountError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| {
        if mutation {
            AccountError::OutcomeUnknown
        } else {
            AccountError::ProviderUnavailable
        }
    })? {
        if chunk.len() > limit - bytes.len() {
            return Err(if mutation {
                AccountError::OutcomeUnknown
            } else {
                AccountError::ServiceFailure
            });
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "tests/service.rs"]
mod tests;
