use super::{
    oidc,
    operations::{validate_id, validate_revision},
    AccountBroker, AccountError, ServiceRequest,
};
use openidconnect::OAuth2TokenResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Release {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub exp: u64,
    pub package_id: String,
    pub revision: String,
    pub version: String,
    pub name: String,
    pub kind: String,
    pub description: String,
    pub license_id: String,
    pub license_text: String,
    pub artifact_digest: String,
    pub artifact_size: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Page<T> {
    items: Vec<T>,
    next_cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entitlement {
    package_id: String,
    revision: String,
    license_id: String,
}

pub(super) fn artifact_path(
    organization: &str,
    package: &str,
    revision: &str,
) -> Result<String, AccountError> {
    validate_id(organization)?;
    validate_id(package)?;
    validate_revision(revision, false)?;
    Ok(format!(
        "/v1/organizations/{organization}/catalog/{package}/{revision}"
    ))
}

impl AccountBroker {
    pub async fn catalog(
        &self,
        generation: &str,
        after: Option<String>,
        query: Option<String>,
    ) -> Result<Value, AccountError> {
        let value = self
            .service_request(generation, ServiceRequest::Catalog { after, query })
            .await?;
        let page: Page<Release> =
            serde_json::from_value(value).map_err(|_| AccountError::ServiceFailure)?;
        if page.items.len() > 100 {
            return Err(AccountError::ServiceFailure);
        }
        for release in &page.items {
            validate_release(release)?;
        }
        if let Some(cursor) = &page.next_cursor {
            validate_id(cursor)?;
        }
        serde_json::to_value(page).map_err(|_| AccountError::ServiceFailure)
    }

    pub async fn catalog_entitlements(
        &self,
        generation: &str,
        organization: String,
        after: Option<String>,
    ) -> Result<Value, AccountError> {
        let value = self
            .service_request(
                generation,
                ServiceRequest::CatalogEntitlements {
                    organization,
                    after,
                },
            )
            .await?;
        let page: Page<Entitlement> =
            serde_json::from_value(value).map_err(|_| AccountError::ServiceFailure)?;
        if page.items.len() > 50 {
            return Err(AccountError::ServiceFailure);
        }
        for item in &page.items {
            validate_id(&item.package_id)?;
            validate_revision(&item.revision, false)?;
            if item.license_id.is_empty() || item.license_id.len() > 128 {
                return Err(AccountError::ServiceFailure);
            }
        }
        if let Some(cursor) = &page.next_cursor {
            validate_revision(cursor, false)?;
        }
        serde_json::to_value(page).map_err(|_| AccountError::ServiceFailure)
    }

    pub(super) async fn catalog_release(
        &self,
        generation: &str,
        organization: &str,
        package_id: &str,
        revision: &str,
    ) -> Result<Release, AccountError> {
        let value = self
            .service_request(
                generation,
                ServiceRequest::CatalogArtifact {
                    organization: organization.into(),
                    package_id: package_id.into(),
                    revision: revision.into(),
                },
            )
            .await?;
        let release: Release =
            serde_json::from_value(value).map_err(|_| AccountError::ServiceFailure)?;
        validate_release(&release)?;
        if release.package_id != package_id || release.revision != revision {
            return Err(AccountError::PackageTrust);
        }
        Ok(release)
    }

    pub(super) async fn package_bytes(
        &self,
        generation: &str,
        organization: &str,
        release: &Release,
    ) -> Result<Vec<u8>, AccountError> {
        let token = {
            let state = self.state.lock().await;
            if state.view.generation != generation {
                return Err(AccountError::Cancelled);
            }
            state
                .authenticated
                .as_ref()
                .ok_or(AccountError::SessionExpired)?
                .tokens
                .access_token()
                .clone()
        };
        let path = artifact_path(organization, &release.package_id, &release.revision)?;
        let url = self.config.endpoint(&format!(
            "{}{path}/artifact",
            self.config.service_url.trim_end_matches('/')
        ))?;
        let response = oidc::http()?
            .get(url)
            .bearer_auth(token.secret())
            .send()
            .await
            .map_err(|_| AccountError::ProviderUnavailable)?;
        if !response.status().is_success() {
            return Err(AccountError::ServiceFailure);
        }
        if response
            .content_length()
            .is_some_and(|length| length != release.artifact_size)
        {
            return Err(AccountError::PackageTrust);
        }
        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| AccountError::ProviderUnavailable)?
        {
            if chunk.len() > super::package::MAX_PACKAGE_BYTES - bytes.len() {
                return Err(AccountError::PackageCapacity);
            }
            bytes.extend_from_slice(&chunk);
        }
        self.operation_identity(generation).await?;
        if bytes.len() as u64 != release.artifact_size
            || super::package::digest(&bytes) != release.artifact_digest
        {
            return Err(AccountError::PackageTrust);
        }
        Ok(bytes)
    }
}

fn validate_release(release: &Release) -> Result<(), AccountError> {
    validate_id(&release.package_id)?;
    validate_revision(&release.revision, false)?;
    if release.iss.len() > 2048
        || release.aud.len() > 256
        || release.sub.len() > 256
        || release.name.is_empty()
        || release.name.len() > 256
        || release.description.len() > 8192
        || release.license_text.len() > 8192
        || release.license_id.is_empty()
        || release.license_id.len() > 128
        || release.version.is_empty()
        || release.version.len() > 64
        || !matches!(release.kind.as_str(), "plugin" | "asset")
        || !super::package::is_digest(&release.artifact_digest)
        || release.artifact_size == 0
        || release.artifact_size > 1073741824
    {
        return Err(AccountError::ServiceFailure);
    }
    Ok(())
}
