use super::{
    apply_recovery, ensure_epoch, finish_account_action, AccountActionRequest, AccountCommandError,
    AccountCommandState, AccountError, AccountSnapshot, AccountView, HubCommandState,
    ServiceRequest, ServiceResponseContext, Value,
};
use crate::projects::{
    project_filesystem_path_key, CloudAccountScope, CloudBindingEnvironment, CloudProjectBinding,
};

pub(super) fn binding_account_scope(
    account: &AccountView,
    generation: &str,
    environment: Option<&CloudBindingEnvironment>,
) -> Option<CloudAccountScope> {
    let environment = environment?;
    (account.status == "signed-in"
        && account.generation == generation
        && account.issuer.as_deref() == Some(environment.issuer.as_str()))
    .then(|| CloudAccountScope {
        environment: environment.clone(),
        subject: account.subject.clone().unwrap_or_default(),
    })
    .filter(|scope| !scope.subject.is_empty())
}

fn binding_data(binding: &CloudProjectBinding) -> Value {
    serde_json::json!({
        "localProjectGuid": binding.local_project_guid.to_string(),
        "organizationId": binding.organization_id,
        "projectId": binding.project_id,
    })
}

pub(super) async fn execute_cloud_binding_action(
    request: AccountActionRequest,
    backend_epoch: String,
    hub: &HubCommandState,
    account: &AccountCommandState,
) -> Result<AccountSnapshot, AccountCommandError> {
    ensure_epoch(&backend_epoch, request.backend_epoch())?;
    let Some(broker) = account.broker() else {
        let mut snapshot = finish_account_action(
            backend_epoch,
            account.unavailable.clone(),
            Err(AccountError::Configuration),
            None,
        );
        snapshot.error = Some(
            account
                .unavailable
                .error
                .clone()
                .unwrap_or_else(|| "account_not_configured".into()),
        );
        return Ok(snapshot);
    };
    let generation = match &request {
        AccountActionRequest::CloudBinding { generation, .. }
        | AccountActionRequest::AttachCloudProject { generation, .. } => generation.clone(),
        _ => unreachable!("only binding actions enter this handler"),
    };
    let initial_view = broker.view().await;
    let scope = binding_account_scope(&initial_view, &generation, account.environment.as_ref());
    let result: Result<Option<Value>, String> = match (request, scope) {
        (_, None) => Err("account_session_expired".into()),
        (AccountActionRequest::CloudBinding { .. }, Some(scope)) => hub
            .session()
            .map(|session| {
                session
                    .selected_cloud_binding(&scope)
                    .as_ref()
                    .map(binding_data)
            })
            .map_err(|_| "hub_cloud_binding_unavailable".into()),
        (
            AccountActionRequest::AttachCloudProject {
                organization,
                project,
                selected_project_id,
                ..
            },
            Some(scope),
        ) => {
            let selected = hub
                .session()
                .ok()
                .and_then(|session| session.selected_local_cloud_project());
            match selected.filter(|local| {
                selected_project_id.len() <= 4096
                    && local.path_key == project_filesystem_path_key(&selected_project_id)
            }) {
                None => Err("hub_cloud_binding_project_unavailable".into()),
                Some(local) => match broker
                    .service_request(
                        &generation,
                        ServiceRequest::CloudHead {
                            organization: organization.clone(),
                            project: project.clone(),
                        },
                    )
                    .await
                {
                    Err(error) => Err(error.to_string()),
                    Ok(_) => {
                        let current = broker.view().await;
                        if binding_account_scope(
                            &current,
                            &generation,
                            account.environment.as_ref(),
                        ) != Some(scope.clone())
                        {
                            Err("account_session_expired".into())
                        } else {
                            hub.session()
                                .map_err(|_| "hub_cloud_binding_unavailable".into())
                                .and_then(|mut session| {
                                    session
                                        .attach_cloud_binding(scope, &local, organization, project)
                                        .map(|binding| Some(binding_data(&binding)))
                                        .map_err(|error| error.to_string())
                                })
                        }
                    }
                },
            }
        }
        _ => unreachable!("only binding actions enter this handler"),
    };
    let (view, recovery) = broker.recovery_snapshot().await;
    let context = ServiceResponseContext {
        generation,
        mutation: false,
    };
    let mut snapshot = finish_account_action(
        backend_epoch,
        view,
        Ok(result.as_ref().ok().cloned().flatten()),
        Some(&context),
    );
    if snapshot.error.is_none() {
        if let Err(error) = result {
            snapshot.error = Some(error);
        }
    }
    apply_recovery(&mut snapshot, recovery);
    Ok(snapshot)
}

#[cfg(test)]
#[path = "tests/cloud_binding.rs"]
mod tests;
