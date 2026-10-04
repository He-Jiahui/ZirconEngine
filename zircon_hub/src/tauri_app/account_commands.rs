use std::{env, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::account::{
    operations::OperationSummary, AccountBroker, AccountConfig, AccountError, AccountView,
    PackageActionSchemaV2, PackageRuntimeMode, ServiceRequest,
};
use crate::projects::CloudBindingEnvironment;

use super::commands::HubCommandState;

mod cloud_binding;
use cloud_binding::execute_cloud_binding_action;
mod cloud_sync;
use cloud_sync::execute_cloud_sync_action;

const ACCOUNT_CONFIG_ENV: &str = "ZIRCON_HUB_ACCOUNT_CONFIG";

pub(super) struct AccountCommandState {
    broker: Option<Arc<AccountBroker>>,
    environment: Option<CloudBindingEnvironment>,
    unavailable: AccountView,
}

impl AccountCommandState {
    pub(super) fn load() -> Self {
        let Some(path) = env::var_os(ACCOUNT_CONFIG_ENV).map(PathBuf::from) else {
            return Self {
                broker: None,
                environment: None,
                unavailable: AccountView::default(),
            };
        };

        let loaded = AccountConfig::load(&path).and_then(|config| {
            let environment = CloudBindingEnvironment {
                issuer: config.issuer.clone(),
                client_id: config.client_id.clone(),
                service_url: config.service_url.clone(),
            };
            AccountBroker::new(config).map(|broker| (broker, environment))
        });
        match loaded {
            Ok((broker, environment)) => Self {
                broker: Some(Arc::new(broker)),
                environment: Some(environment),
                unavailable: AccountView::default(),
            },
            Err(error) => Self {
                broker: None,
                environment: None,
                unavailable: AccountView {
                    configured: false,
                    status: "unavailable".into(),
                    error: Some(error.to_string()),
                    ..AccountView::default()
                },
            },
        }
    }

    async fn view(&self) -> AccountView {
        match self.broker.as_ref() {
            Some(broker) => broker.view().await,
            None => self.unavailable.clone(),
        }
    }

    fn broker(&self) -> Option<Arc<AccountBroker>> {
        self.broker.clone()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccountSnapshot {
    pub backend_epoch: String,
    pub account: AccountView,
    pub data: Option<Value>,
    pub error: Option<String>,
    pub operations: Vec<OperationSummary>,
    pub operations_revision: String,
    pub operations_error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(
    tag = "action",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum AccountActionRequest {
    CloudBinding {
        backend_epoch: String,
        generation: String,
    },
    AttachCloudProject {
        backend_epoch: String,
        generation: String,
        organization: String,
        project: String,
        selected_project_id: String,
    },
    CloudHead {
        backend_epoch: String,
        generation: String,
        organization: String,
        project: String,
    },
    CloudPush {
        backend_epoch: String,
        generation: String,
        operation_id: String,
        base_revision: String,
    },
    CloudResumePush {
        backend_epoch: String,
        generation: String,
        operation_id: String,
    },
    CloudStageDownload {
        backend_epoch: String,
        generation: String,
        expected_revision: String,
    },
    CloudApplyDownload {
        backend_epoch: String,
        generation: String,
        stage_id: String,
        expected_revision: String,
    },
    CloudDiscardDownload {
        backend_epoch: String,
        generation: String,
        expected_revision: String,
    },
    Catalog {
        backend_epoch: String,
        generation: String,
        after: Option<String>,
        query: Option<String>,
    },
    CatalogEntitlements {
        backend_epoch: String,
        generation: String,
        organization: String,
        after: Option<String>,
    },
    CatalogLicense {
        backend_epoch: String,
        generation: String,
        organization: String,
        operation_id: String,
        expected_policy_revision: String,
        package_id: String,
        revision: String,
        license_id: String,
    },
    PackageInventory {
        schema_version: PackageActionSchemaV2,
        target_mode: PackageRuntimeMode,
        backend_epoch: String,
        generation: String,
        organization: String,
    },
    PackageInstall {
        schema_version: PackageActionSchemaV2,
        target_mode: PackageRuntimeMode,
        backend_epoch: String,
        generation: String,
        organization: String,
        operation_id: String,
        package_id: String,
        revision: String,
        expected_inventory_revision: String,
    },
    SignIn {
        backend_epoch: String,
    },
    Refresh {
        backend_epoch: String,
    },
    Logout {
        backend_epoch: String,
    },
    Cancel {
        backend_epoch: String,
    },
    Organizations {
        backend_epoch: String,
        generation: String,
        after: Option<String>,
    },
    Invitations {
        backend_epoch: String,
        generation: String,
        after: Option<String>,
    },
    IssuedInvitations {
        backend_epoch: String,
        generation: String,
        organization: String,
        after: Option<String>,
    },
    Members {
        backend_epoch: String,
        generation: String,
        organization: String,
        after: Option<String>,
    },
    Projects {
        backend_epoch: String,
        generation: String,
        organization: String,
        after: Option<String>,
    },
    CreateOrganization {
        backend_epoch: String,
        generation: String,
        operation_id: String,
        name: String,
    },
    Mutate {
        backend_epoch: String,
        generation: String,
        organization: String,
        operation_id: String,
        expected_policy_revision: String,
        mutation: Value,
    },
    Receipt {
        backend_epoch: String,
        generation: String,
        operation_id: String,
    },
    Reconcile {
        backend_epoch: String,
        generation: String,
        operation_id: String,
    },
    Retry {
        backend_epoch: String,
        generation: String,
        operation_id: String,
    },
    Acknowledge {
        backend_epoch: String,
        generation: String,
        operation_id: String,
    },
}

impl AccountActionRequest {
    fn backend_epoch(&self) -> &str {
        match self {
            Self::SignIn { backend_epoch }
            | Self::Refresh { backend_epoch }
            | Self::Logout { backend_epoch }
            | Self::Cancel { backend_epoch }
            | Self::Organizations { backend_epoch, .. }
            | Self::Invitations { backend_epoch, .. }
            | Self::IssuedInvitations { backend_epoch, .. }
            | Self::Members { backend_epoch, .. }
            | Self::Projects { backend_epoch, .. }
            | Self::CloudBinding { backend_epoch, .. }
            | Self::AttachCloudProject { backend_epoch, .. }
            | Self::CloudHead { backend_epoch, .. }
            | Self::CloudPush { backend_epoch, .. }
            | Self::CloudResumePush { backend_epoch, .. }
            | Self::CloudStageDownload { backend_epoch, .. }
            | Self::CloudApplyDownload { backend_epoch, .. }
            | Self::CloudDiscardDownload { backend_epoch, .. }
            | Self::CreateOrganization { backend_epoch, .. }
            | Self::Mutate { backend_epoch, .. }
            | Self::Receipt { backend_epoch, .. } => backend_epoch,
            Self::Catalog { backend_epoch, .. }
            | Self::CatalogEntitlements { backend_epoch, .. }
            | Self::CatalogLicense { backend_epoch, .. }
            | Self::PackageInventory { backend_epoch, .. }
            | Self::PackageInstall { backend_epoch, .. } => backend_epoch,
            Self::Reconcile { backend_epoch, .. }
            | Self::Retry { backend_epoch, .. }
            | Self::Acknowledge { backend_epoch, .. } => backend_epoch,
        }
    }

    fn service_response_context(&self) -> Option<ServiceResponseContext> {
        let (generation, mutation) = match self {
            Self::Catalog { generation, .. }
            | Self::CatalogEntitlements { generation, .. }
            | Self::PackageInventory { generation, .. }
            | Self::Organizations { generation, .. }
            | Self::Invitations { generation, .. }
            | Self::IssuedInvitations { generation, .. }
            | Self::Members { generation, .. }
            | Self::Projects { generation, .. }
            | Self::CloudBinding { generation, .. }
            | Self::AttachCloudProject { generation, .. }
            | Self::CloudHead { generation, .. }
            | Self::Receipt { generation, .. } => (generation, false),
            Self::CloudStageDownload { generation, .. } => (generation, false),
            Self::Reconcile { generation, .. } | Self::Acknowledge { generation, .. } => {
                (generation, false)
            }
            Self::CatalogLicense { generation, .. }
            | Self::PackageInstall { generation, .. }
            | Self::CreateOrganization { generation, .. }
            | Self::Mutate { generation, .. } => (generation, true),
            Self::CloudPush { generation, .. }
            | Self::CloudResumePush { generation, .. }
            | Self::CloudApplyDownload { generation, .. }
            | Self::CloudDiscardDownload { generation, .. } => (generation, true),
            Self::Retry { generation, .. } => (generation, true),
            _ => return None,
        };
        Some(ServiceResponseContext {
            generation: generation.clone(),
            mutation,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AccountCommandError {
    #[error("hub_state_epoch_stale")]
    StaleEpoch,
}

pub(crate) async fn account_state(
    hub: tauri::State<'_, HubCommandState>,
    account: tauri::State<'_, AccountCommandState>,
) -> Result<AccountSnapshot, AccountCommandError> {
    let (view, recovery) = match account.broker.as_ref() {
        Some(broker) => broker.recovery_snapshot().await,
        None => (account.view().await, Ok((Vec::new(), "0".into()))),
    };
    let mut snapshot = finish_account_action(hub.backend_epoch().to_owned(), view, Ok(None), None);
    apply_recovery(&mut snapshot, recovery);
    Ok(snapshot)
}

pub(crate) async fn account_action(
    request: AccountActionRequest,
    hub: tauri::State<'_, HubCommandState>,
    account: tauri::State<'_, AccountCommandState>,
) -> Result<AccountSnapshot, AccountCommandError> {
    let backend_epoch = hub.backend_epoch().to_owned();
    match request {
        request @ (AccountActionRequest::CloudBinding { .. }
        | AccountActionRequest::AttachCloudProject { .. }) => {
            execute_cloud_binding_action(request, backend_epoch, &hub, &account).await
        }
        request @ (AccountActionRequest::CloudPush { .. }
        | AccountActionRequest::CloudResumePush { .. }
        | AccountActionRequest::CloudStageDownload { .. }
        | AccountActionRequest::CloudApplyDownload { .. }
        | AccountActionRequest::CloudDiscardDownload { .. }) => {
            execute_cloud_sync_action(request, backend_epoch, &hub, &account).await
        }
        request => {
            let broker = account.broker();
            execute_account_action(request, backend_epoch, broker, account.unavailable.clone())
                .await
        }
    }
}

async fn execute_account_action(
    request: AccountActionRequest,
    backend_epoch: String,
    broker: Option<Arc<AccountBroker>>,
    unavailable: AccountView,
) -> Result<AccountSnapshot, AccountCommandError> {
    let expected_epoch = request.backend_epoch().to_owned();
    ensure_epoch(&backend_epoch, &expected_epoch)?;
    let Some(broker) = broker else {
        let account = unavailable;
        return Ok(AccountSnapshot {
            backend_epoch: expected_epoch,
            error: Some(
                account
                    .error
                    .clone()
                    .unwrap_or_else(|| "account_not_configured".into()),
            ),
            account,
            data: None,
            operations: Vec::new(),
            operations_revision: "0".into(),
            operations_error: None,
        });
    };
    let service_context = request.service_response_context();

    let result = match request {
        AccountActionRequest::CloudBinding { .. }
        | AccountActionRequest::AttachCloudProject { .. }
        | AccountActionRequest::CloudPush { .. }
        | AccountActionRequest::CloudResumePush { .. }
        | AccountActionRequest::CloudStageDownload { .. }
        | AccountActionRequest::CloudApplyDownload { .. }
        | AccountActionRequest::CloudDiscardDownload { .. } => {
            unreachable!("project cloud actions are handled with the Hub session")
        }
        AccountActionRequest::CloudHead {
            generation,
            organization,
            project,
            ..
        } => broker
            .service_request(
                &generation,
                ServiceRequest::CloudHead {
                    organization,
                    project,
                },
            )
            .await
            .map(Some),
        AccountActionRequest::Catalog {
            generation,
            after,
            query,
            ..
        } => broker.catalog(&generation, after, query).await.map(Some),
        AccountActionRequest::CatalogEntitlements {
            generation,
            organization,
            after,
            ..
        } => broker
            .catalog_entitlements(&generation, organization, after)
            .await
            .map(Some),
        AccountActionRequest::CatalogLicense {
            generation,
            organization,
            operation_id,
            expected_policy_revision,
            package_id,
            revision,
            license_id,
            ..
        } => broker
            .service_request(
                &generation,
                ServiceRequest::CatalogLicense {
                    organization,
                    operation_id,
                    expected_policy_revision,
                    package_id,
                    revision,
                    license_id,
                },
            )
            .await
            .map(Some),
        AccountActionRequest::PackageInventory {
            generation,
            organization,
            target_mode,
            ..
        } => broker
            .package_inventory(&generation, organization, target_mode)
            .await
            .map(Some),
        AccountActionRequest::PackageInstall {
            generation,
            organization,
            operation_id,
            package_id,
            revision,
            expected_inventory_revision,
            target_mode,
            ..
        } => broker
            .install_package(
                &generation,
                organization,
                operation_id,
                package_id,
                revision,
                expected_inventory_revision,
                target_mode,
            )
            .await
            .map(Some),
        AccountActionRequest::SignIn { .. } => broker.authenticate(false).await.map(|_| None),
        AccountActionRequest::Refresh { .. } => broker.authenticate(true).await.map(|_| None),
        AccountActionRequest::Logout { .. } | AccountActionRequest::Cancel { .. } => {
            broker.logout().await.map(|_| None)
        }
        AccountActionRequest::Organizations {
            generation, after, ..
        } => broker
            .service_request(&generation, ServiceRequest::Organizations { after })
            .await
            .map(Some),
        AccountActionRequest::Invitations {
            generation, after, ..
        } => broker
            .service_request(&generation, ServiceRequest::Invitations { after })
            .await
            .map(Some),
        AccountActionRequest::IssuedInvitations {
            generation,
            organization,
            after,
            ..
        } => broker
            .service_request(
                &generation,
                ServiceRequest::IssuedInvitations {
                    organization,
                    after,
                },
            )
            .await
            .map(Some),
        AccountActionRequest::Members {
            generation,
            organization,
            after,
            ..
        } => broker
            .service_request(
                &generation,
                ServiceRequest::Members {
                    organization,
                    after,
                },
            )
            .await
            .map(Some),
        AccountActionRequest::Projects {
            generation,
            organization,
            after,
            ..
        } => broker
            .service_request(
                &generation,
                ServiceRequest::Projects {
                    organization,
                    after,
                },
            )
            .await
            .map(Some),
        AccountActionRequest::CreateOrganization {
            generation,
            operation_id,
            name,
            ..
        } => broker
            .service_request(
                &generation,
                ServiceRequest::CreateOrganization { operation_id, name },
            )
            .await
            .map(Some),
        AccountActionRequest::Mutate {
            generation,
            organization,
            operation_id,
            expected_policy_revision,
            mutation,
            ..
        } => broker
            .service_request(
                &generation,
                ServiceRequest::Mutate {
                    organization,
                    operation_id,
                    expected_policy_revision,
                    mutation,
                },
            )
            .await
            .map(Some),
        AccountActionRequest::Receipt {
            generation,
            operation_id,
            ..
        } => broker
            .service_request(&generation, ServiceRequest::Receipt { operation_id })
            .await
            .map(Some),
        AccountActionRequest::Reconcile {
            generation,
            operation_id,
            ..
        } => broker
            .reconcile_operation(&generation, &operation_id)
            .await
            .map(Some),
        AccountActionRequest::Retry {
            generation,
            operation_id,
            ..
        } => broker
            .retry_operation(&generation, &operation_id)
            .await
            .map(Some),
        AccountActionRequest::Acknowledge {
            generation,
            operation_id,
            ..
        } => broker
            .acknowledge_operation(&generation, &operation_id)
            .await
            .map(|_| None),
    };
    let (account, recovery) = broker.recovery_snapshot().await;
    let mut snapshot =
        finish_account_action(expected_epoch, account, result, service_context.as_ref());
    apply_recovery(&mut snapshot, recovery);
    Ok(snapshot)
}

struct ServiceResponseContext {
    generation: String,
    mutation: bool,
}

fn finish_account_action(
    backend_epoch: String,
    account: AccountView,
    result: Result<Option<Value>, AccountError>,
    service_context: Option<&ServiceResponseContext>,
) -> AccountSnapshot {
    // A logout or account switch may finish after the HTTP response but before this view read.
    let result = match service_context {
        Some(context)
            if context.generation != account.generation || account.status != "signed-in" =>
        {
            Err(if context.mutation {
                AccountError::OutcomeUnknown
            } else {
                AccountError::Cancelled
            })
        }
        _ => result,
    };
    let (data, error) = match result {
        Ok(data) => (data, None),
        Err(error) => (None, Some(error.to_string())),
    };
    AccountSnapshot {
        backend_epoch,
        account,
        data,
        error,
        operations: Vec::new(),
        operations_revision: "0".into(),
        operations_error: None,
    }
}

fn apply_recovery(
    snapshot: &mut AccountSnapshot,
    recovery: Result<(Vec<OperationSummary>, String), AccountError>,
) {
    match recovery {
        Ok((operations, revision)) => {
            snapshot.operations = operations;
            snapshot.operations_revision = revision;
            snapshot.operations_error = None;
        }
        Err(error) => {
            snapshot.operations = Vec::new();
            snapshot.operations_error = Some(error.to_string());
        }
    }
}

fn ensure_epoch(current: &str, expected: &str) -> Result<(), AccountCommandError> {
    if current != expected {
        return Err(AccountCommandError::StaleEpoch);
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/account_commands.rs"]
mod tests;
