mod callback;
mod catalog;
pub(crate) mod cloud;
mod config;
mod credential;
mod oidc;
pub(crate) mod operations;
mod package;
mod recovery;
mod service;
pub(crate) use package::{PackageActionSchemaV2, PackageRuntimeMode};
pub use service::ServiceRequest;

pub use config::AccountConfig;
use credential::{CredentialStore, SavedSession, WindowsCredentials};
use openidconnect::OAuth2TokenResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("account_configuration_invalid")]
    Configuration,
    #[error("identity_provider_unavailable")]
    ProviderUnavailable,
    #[error("account_credential_store_unavailable")]
    CredentialStore,
    #[error("account_callback_invalid")]
    Callback,
    #[error("account_login_cancelled")]
    Cancelled,
    #[error("account_login_timed_out")]
    Timeout,
    #[error("account_browser_unavailable")]
    Browser,
    #[error("account_identity_invalid")]
    InvalidIdentity,
    #[error("account_session_expired")]
    SessionExpired,
    #[error("account_operation_busy")]
    Busy,
    #[error("account_service_operation_failed")]
    ServiceFailure,
    #[error("account_service_outcome_unknown")]
    OutcomeUnknown,
    #[error("account_operation_store_unavailable")]
    OperationStore,
    #[error("account_operation_id_conflict")]
    OperationConflict,
    #[error("account_revocation_pending")]
    RevocationPending,
    #[error("account_package_service_unavailable")]
    PackageUnavailable,
    #[error("account_package_trust_denied")]
    PackageTrust,
    #[error("account_package_capacity_exceeded")]
    PackageCapacity,
    #[error("account_package_inventory_conflict")]
    PackageConflict,
    #[error("account_package_target_migration_required")]
    PackageTargetRequired,
    #[error("account_package_policy_unconfigured")]
    PackagePolicyUnconfigured,
    #[error("account_package_target_unconfigured")]
    PackageTargetUnconfigured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub configured: bool,
    pub status: String,
    pub issuer: Option<String>,
    pub subject: Option<String>,
    pub display_name: Option<String>,
    pub generation: String,
    pub error: Option<String>,
}

impl Default for AccountView {
    fn default() -> Self {
        Self {
            configured: false,
            status: "unavailable".into(),
            issuer: None,
            subject: None,
            display_name: None,
            generation: "0".into(),
            error: None,
        }
    }
}

struct State {
    generation: u64,
    view: AccountView,
    authenticated: Option<oidc::Authenticated>,
}

pub struct AccountBroker {
    config: AccountConfig,
    credentials: Arc<dyn CredentialStore>,
    state: Mutex<State>,
    operation: Mutex<()>,
    mutation: Mutex<()>,
    cancel: watch::Sender<u64>,
}

impl AccountBroker {
    pub fn new(config: AccountConfig) -> Result<Self, AccountError> {
        let credentials = Arc::new(WindowsCredentials::new(&config.issuer, &config.client_id)?);
        let pending_revocation = credentials.pending_revocation()?.is_some();
        let (cancel, _) = watch::channel(0);
        Ok(Self {
            config,
            credentials,
            state: Mutex::new(State {
                generation: 0,
                authenticated: None,
                view: AccountView {
                    configured: true,
                    status: "signed-out".into(),
                    generation: "0".into(),
                    error: pending_revocation.then(|| AccountError::RevocationPending.to_string()),
                    ..Default::default()
                },
            }),
            operation: Mutex::new(()),
            mutation: Mutex::new(()),
            cancel,
        })
    }

    pub async fn view(&self) -> AccountView {
        self.state.lock().await.view.clone()
    }

    pub async fn authenticate(&self, refresh: bool) -> Result<AccountView, AccountError> {
        let mut cancel = self.cancel.subscribe();
        let attempt = *cancel.borrow();
        let _operation = self.operation.try_lock().map_err(|_| AccountError::Busy)?;
        {
            let mut state = self.state.lock().await;
            if *cancel.borrow() != attempt {
                return Err(AccountError::Cancelled);
            }
            state.generation += 1;
            state.authenticated = None;
            state.view = AccountView {
                configured: true,
                status: "signed-out".into(),
                generation: state.generation.to_string(),
                ..Default::default()
            };
        }
        self.retry_revocation().await?;
        let credentials = self.credentials.clone();
        let saved = if refresh {
            tokio::task::spawn_blocking(move || credentials.load())
                .await
                .map_err(|_| AccountError::CredentialStore)??
                .ok_or(AccountError::SessionExpired)
                .map(Some)?
        } else {
            None
        };
        let result = tokio::select! {
            biased;
            _ = cancel.changed() => Err(AccountError::Cancelled),
            result = oidc::authenticate(&self.config, saved.as_ref()) => result,
        };
        match result {
            Ok(authenticated) => {
                if *cancel.borrow() != attempt {
                    return Err(AccountError::Cancelled);
                }
                let refresh_token = authenticated
                    .tokens
                    .refresh_token()
                    .map(|token| token.secret().clone())
                    .or_else(|| saved.as_ref().map(|saved| saved.refresh_token.clone()))
                    .ok_or(AccountError::InvalidIdentity)?;
                let expected_saved = saved;
                let saved = SavedSession {
                    issuer: self.config.issuer.clone(),
                    subject: authenticated.subject.clone(),
                    nonce: authenticated.nonce.clone(),
                    refresh_token,
                    revoke_only: false,
                };
                let credentials = self.credentials.clone();
                tokio::task::spawn_blocking(move || {
                    credentials.save(&saved, expected_saved.as_ref())
                })
                .await
                .map_err(|_| AccountError::CredentialStore)??;
                if *cancel.borrow() != attempt {
                    let credentials = self.credentials.clone();
                    tokio::task::spawn_blocking(move || credentials.begin_logout())
                        .await
                        .map_err(|_| AccountError::CredentialStore)??;
                    return Err(AccountError::Cancelled);
                }
                let mut state = self.state.lock().await;
                let cancelled = *cancel.borrow() != attempt;
                if cancelled {
                    drop(state);
                    let credentials = self.credentials.clone();
                    tokio::task::spawn_blocking(move || credentials.begin_logout())
                        .await
                        .map_err(|_| AccountError::CredentialStore)??;
                    return Err(AccountError::Cancelled);
                }
                state.generation += 1;
                state.view = AccountView {
                    configured: true,
                    status: "signed-in".into(),
                    issuer: Some(self.config.issuer.clone()),
                    subject: Some(authenticated.subject.clone()),
                    display_name: Some(authenticated.name.clone()),
                    generation: state.generation.to_string(),
                    error: None,
                };
                state.authenticated = Some(authenticated);
                Ok(state.view.clone())
            }
            Err(error) => {
                if refresh
                    && matches!(
                        error,
                        AccountError::SessionExpired | AccountError::InvalidIdentity
                    )
                {
                    let credentials = self.credentials.clone();
                    let expected_saved = saved;
                    tokio::task::spawn_blocking(move || credentials.clear(expected_saved.as_ref()))
                        .await
                        .map_err(|_| AccountError::CredentialStore)??;
                }
                let mut state = self.state.lock().await;
                state.generation += 1;
                state.authenticated = None;
                state.view = AccountView {
                    configured: true,
                    status: "signed-out".into(),
                    generation: state.generation.to_string(),
                    error: Some(error.to_string()),
                    ..Default::default()
                };
                Err(error)
            }
        }
    }

    pub async fn logout(&self) -> Result<AccountView, AccountError> {
        {
            // Local package publication uses this state lock as its authorization boundary.
            let _state = self.state.lock().await;
            self.cancel.send_modify(|generation| *generation += 1);
        }
        let _operation = self.operation.lock().await;
        let credentials = self.credentials.clone();
        let mut state = self.state.lock().await;
        state.generation += 1;
        state.authenticated = None;
        state.view = AccountView {
            configured: true,
            status: "signed-out".into(),
            generation: state.generation.to_string(),
            ..Default::default()
        };
        drop(state);
        tokio::task::spawn_blocking(move || credentials.begin_logout())
            .await
            .map_err(|_| AccountError::CredentialStore)??;
        self.retry_revocation().await?;
        Ok(self.view().await)
    }

    async fn retry_revocation(&self) -> Result<(), AccountError> {
        let credentials = self.credentials.clone();
        let saved = tokio::task::spawn_blocking(move || credentials.pending_revocation())
            .await
            .map_err(|_| AccountError::CredentialStore)??;
        if let Some(saved) = saved {
            if saved.issuer != self.config.issuer {
                return Err(AccountError::CredentialStore);
            }
            oidc::revoke(&self.config, &saved.refresh_token)
                .await
                .map_err(|_| AccountError::RevocationPending)?;
            let credentials = self.credentials.clone();
            tokio::task::spawn_blocking(move || credentials.finish_revocation(&saved))
                .await
                .map_err(|_| AccountError::CredentialStore)??;
        }
        Ok(())
    }

    pub async fn organizations(&self) -> Result<serde_json::Value, AccountError> {
        self.service_request(
            &self.view().await.generation,
            ServiceRequest::Organizations { after: None },
        )
        .await
    }
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
