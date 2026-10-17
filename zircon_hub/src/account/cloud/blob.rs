use super::{
    super::{
        oidc, operations::validate_id, service::read_service_body, AccountBroker, AccountError,
    },
    manifest::{digest, MAX_BLOB_BYTES},
};
use openidconnect::{reqwest, OAuth2TokenResponse};
use sha2::{Digest, Sha256};

impl AccountBroker {
    // Binary project data stays inside the Rust broker, never a Tauri/WebView response.
    pub(crate) async fn download_cloud_blob(
        &self,
        expected_generation: &str,
        organization: &str,
        project: &str,
        expected_digest: &str,
    ) -> Result<Vec<u8>, AccountError> {
        validate_id(organization)?;
        validate_id(project)?;
        if !digest(expected_digest) {
            return Err(AccountError::ServiceFailure);
        }
        let (token, generation) = {
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
            )
        };
        let url = self.config.endpoint(&format!(
            "{}/v1/organizations/{organization}/projects/{project}/cloud/blobs/{expected_digest}",
            self.config.service_url.trim_end_matches('/')
        ))?;
        let http = oidc::http()?;
        {
            let state = self.state.lock().await;
            if state.generation != generation || state.authenticated.is_none() {
                return Err(AccountError::Cancelled);
            }
        }
        let response = http
            .get(url)
            .bearer_auth(token.secret())
            .send()
            .await
            .map_err(|_| AccountError::ProviderUnavailable)?;
        if !response.status().is_success()
            || response
                .content_length()
                .is_some_and(|length| length > MAX_BLOB_BYTES as u64)
        {
            return Err(AccountError::ServiceFailure);
        }
        let bytes = read_service_body(response, false, MAX_BLOB_BYTES).await?;
        if format!("{:x}", Sha256::digest(&bytes)) != expected_digest {
            return Err(AccountError::ServiceFailure);
        }
        let state = self.state.lock().await;
        if state.generation != generation || state.authenticated.is_none() {
            return Err(AccountError::Cancelled);
        }
        Ok(bytes)
    }

    // Content-addressed PUT is repeatable: an uncertain response never commits a project head.
    pub(crate) async fn upload_cloud_blob(
        &self,
        expected_generation: &str,
        organization: &str,
        project: &str,
        expected_digest: &str,
        bytes: Vec<u8>,
    ) -> Result<(), AccountError> {
        validate_id(organization)?;
        validate_id(project)?;
        if !digest(expected_digest)
            || bytes.len() > MAX_BLOB_BYTES
            || format!("{:x}", Sha256::digest(&bytes)) != expected_digest
        {
            return Err(AccountError::ServiceFailure);
        }
        let (token, generation) = {
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
            )
        };
        let url = self.config.endpoint(&format!(
            "{}/v1/organizations/{organization}/projects/{project}/cloud/blobs/{expected_digest}",
            self.config.service_url.trim_end_matches('/')
        ))?;
        let http = oidc::http()?;
        let request = http
            .put(url)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .bearer_auth(token.secret())
            .body(bytes)
            .build()
            .map_err(|_| AccountError::ServiceFailure)?;
        {
            let state = self.state.lock().await;
            if state.generation != generation || state.authenticated.is_none() {
                return Err(AccountError::Cancelled);
            }
        }
        let response = http
            .execute(request)
            .await
            .map_err(|_| AccountError::OutcomeUnknown)?;
        let status = response.status();
        let state = self.state.lock().await;
        if state.generation != generation || state.authenticated.is_none() {
            return Err(AccountError::OutcomeUnknown);
        }
        if status == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else if status.is_client_error() {
            Err(AccountError::ServiceFailure)
        } else {
            Err(AccountError::OutcomeUnknown)
        }
    }
}

#[cfg(test)]
#[path = "blob/tests/cases.rs"]
mod tests;
