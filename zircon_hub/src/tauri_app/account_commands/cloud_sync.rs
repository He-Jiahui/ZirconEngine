use std::{future::Future, path::PathBuf, sync::Arc};

use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    account::{
        cloud::{manifest::Manifest, snapshot, staging, sync::ProjectCloudSyncLease},
        operations::{CloudCommitCleanupRecord, OperationStatus, OperationSummary},
        AccountBroker, AccountError, AccountView, ServiceRequest,
    },
    projects::{CloudAccountScope, CloudBindingEnvironment, CloudProjectBinding},
};

use super::{
    apply_recovery, ensure_epoch, finish_account_action, AccountActionRequest, AccountCommandError,
    AccountCommandState, AccountSnapshot, HubCommandState, ServiceResponseContext,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CloudHead {
    organization_id: String,
    project_id: String,
    #[allow(dead_code)]
    base_revision: String,
    revision: String,
    manifest_digest: String,
    #[allow(dead_code)]
    created_at: u64,
    #[allow(dead_code)]
    created_by: String,
    manifest: Manifest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LocalProject {
    root: PathBuf,
    path_key: String,
    guid: zircon_runtime_interface::project::ProjectGuid,
}

pub(super) async fn execute_cloud_sync_action(
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

    let generation = request_generation(&request).to_owned();
    let initial = broker.view().await;
    let scope = super::cloud_binding::binding_account_scope(
        &initial,
        &generation,
        account.environment.as_ref(),
    );
    let result = match scope {
        None => Err("account_session_expired".to_owned()),
        Some(scope) => execute_bound_sync(&request, &generation, &scope, hub, &broker).await,
    };

    let (view, recovery) = broker.recovery_snapshot().await;
    let mutation = matches!(
        request,
        AccountActionRequest::CloudPush { .. }
            | AccountActionRequest::CloudResumePush { .. }
            | AccountActionRequest::CloudApplyDownload { .. }
            | AccountActionRequest::CloudDiscardDownload { .. }
    );
    let context = ServiceResponseContext {
        generation,
        mutation,
    };
    let mut snapshot = finish_account_action(
        backend_epoch,
        view,
        Ok(result.as_ref().ok().cloned()),
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

async fn execute_bound_sync(
    request: &AccountActionRequest,
    generation: &str,
    scope: &CloudAccountScope,
    hub: &HubCommandState,
    broker: &Arc<AccountBroker>,
) -> Result<Value, String> {
    let (local, binding) = selected_binding(hub, scope)?;
    let project_lease = Arc::new(ProjectCloudSyncLease::acquire(&local.root)?);
    let scope_fingerprint = staging::download_scope_fingerprint(
        local.guid,
        scope,
        &binding.organization_id,
        &binding.project_id,
    );
    let recovery_root = local.root.clone();
    let organization_id = binding.organization_id.clone();
    let project_id = binding.project_id.clone();
    let project_guid = local.guid;
    let recovery_lease = Arc::clone(&project_lease);
    guarded_recovery_cleanup(
        broker.cloud_cleanup_snapshot(),
        generation,
        scope,
        move |operations, terminal_cloud_operations| async move {
            let journal_operation_ids = operations
                .into_iter()
                .map(|operation| operation.operation_id)
                .collect::<Vec<_>>();
            tokio::task::spawn_blocking(move || {
                let _lease = recovery_lease;
                staging::recover_orphaned_publish_temporaries(&recovery_root, &_lease)?;
                staging::prune_upload_snapshots(
                    &recovery_root,
                    &organization_id,
                    &project_id,
                    project_guid,
                    &scope_fingerprint,
                    &journal_operation_ids,
                    &terminal_cloud_operations,
                    &_lease,
                )
            })
            .await
            .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
            .map_err(str::to_owned)
        },
    )
    .await?;
    match request {
        AccountActionRequest::CloudPush {
            operation_id,
            base_revision,
            ..
        } => {
            push_snapshot(
                operation_id,
                Some(base_revision),
                generation,
                &local,
                &binding,
                scope,
                hub,
                broker,
                &project_lease,
            )
            .await
        }
        AccountActionRequest::CloudResumePush { operation_id, .. } => {
            resume_snapshot(
                operation_id,
                generation,
                &local,
                &binding,
                scope,
                hub,
                broker,
                &project_lease,
            )
            .await
        }
        AccountActionRequest::CloudStageDownload {
            expected_revision, ..
        } => {
            let scope_fingerprint = staging::download_scope_fingerprint(
                local.guid,
                scope,
                &binding.organization_id,
                &binding.project_id,
            );
            let stage_id = staging::stable_download_stage_id(
                local.guid,
                scope,
                &binding.organization_id,
                &binding.project_id,
                expected_revision,
            )
            .map_err(str::to_owned)?;
            stage_download(
                &stage_id,
                &scope_fingerprint,
                expected_revision,
                generation,
                &local,
                &binding,
                scope,
                hub,
                broker,
                &project_lease,
            )
            .await
        }
        AccountActionRequest::CloudDiscardDownload {
            expected_revision, ..
        } => {
            ensure_context(hub, broker, generation, scope, &local, &binding).await?;
            let scope_fingerprint = staging::download_scope_fingerprint(
                local.guid,
                scope,
                &binding.organization_id,
                &binding.project_id,
            );
            let stage_id = staging::stable_download_stage_id(
                local.guid,
                scope,
                &binding.organization_id,
                &binding.project_id,
                expected_revision,
            )
            .map_err(str::to_owned)?;
            let root = local.root.clone();
            let organization_id = binding.organization_id.clone();
            let project_id = binding.project_id.clone();
            let expected_revision = expected_revision.clone();
            let result_revision = expected_revision.clone();
            let stage_id_for_discard = stage_id.clone();
            let lease = Arc::clone(&project_lease);
            tokio::task::spawn_blocking(move || {
                let _lease = lease;
                staging::discard_bound_download_stage(
                    &root,
                    &stage_id_for_discard,
                    &organization_id,
                    &project_id,
                    &expected_revision,
                    &scope_fingerprint,
                    &_lease,
                )
            })
            .await
            .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
            .map_err(str::to_owned)?;
            Ok(json!({
                "status": "discarded",
                "stageId": stage_id,
                "revision": result_revision,
            }))
        }
        AccountActionRequest::CloudApplyDownload {
            stage_id,
            expected_revision,
            ..
        } => {
            apply_download(
                stage_id,
                expected_revision,
                generation,
                &local,
                &binding,
                scope,
                hub,
                broker,
                &project_lease,
            )
            .await
        }
        _ => unreachable!("only cloud sync actions enter this handler"),
    }
}

async fn push_snapshot(
    operation_id: &str,
    requested_base_revision: Option<&str>,
    generation: &str,
    local: &LocalProject,
    binding: &CloudProjectBinding,
    scope: &CloudAccountScope,
    hub: &HubCommandState,
    broker: &Arc<AccountBroker>,
    project_lease: &Arc<ProjectCloudSyncLease>,
) -> Result<Value, String> {
    let scope_fingerprint = staging::download_scope_fingerprint(
        local.guid,
        scope,
        &binding.organization_id,
        &binding.project_id,
    );
    let upload = match load_upload(local, binding, operation_id, &scope_fingerprint)? {
        Some(upload) => {
            if requested_base_revision.is_some_and(|revision| upload.base_revision != revision) {
                return Err("hub_cloud_sync_stage_invalid".into());
            }
            upload
        }
        None => {
            let base_revision =
                requested_base_revision.ok_or_else(|| "hub_cloud_sync_stage_invalid".to_owned())?;
            let snapshot = capture_local(local.clone(), Arc::clone(project_lease)).await?;
            let upload = staging::prepare_upload_snapshot(
                &local.root,
                operation_id,
                &binding.organization_id,
                &binding.project_id,
                local.guid,
                &scope_fingerprint,
                base_revision,
                snapshot.manifest.clone(),
            )
            .map_err(str::to_owned)?;
            for entry in &snapshot.manifest.files {
                let root = local.root.clone();
                let entry_for_read = entry.clone();
                let lease = Arc::clone(project_lease);
                let bytes = tokio::task::spawn_blocking(move || {
                    let _lease = lease;
                    snapshot::read_blob(&root, &entry_for_read)
                })
                .await
                .map_err(|_| "hub_cloud_sync_snapshot_invalid".to_owned())?
                .map_err(str::to_owned)?;
                let upload_copy = upload.clone();
                let entry_for_store = entry.clone();
                let lease = Arc::clone(project_lease);
                tokio::task::spawn_blocking(move || {
                    let _lease = lease;
                    staging::store_upload_blob(&upload_copy, &entry_for_store, &bytes)
                })
                .await
                .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
                .map_err(str::to_owned)?;
            }
            let after_stage = capture_local(local.clone(), Arc::clone(project_lease)).await?;
            if after_stage.manifest != snapshot.manifest {
                return Err("hub_cloud_sync_project_changed".into());
            }
            load_upload(local, binding, operation_id, &scope_fingerprint)?
                .ok_or_else(|| "hub_cloud_sync_stage_invalid".to_owned())?
        }
    };

    ensure_context(hub, broker, generation, scope, local, binding).await?;
    for entry in &upload.manifest.files {
        let upload_copy = upload.clone();
        let entry_copy = entry.clone();
        let lease = Arc::clone(project_lease);
        let bytes = tokio::task::spawn_blocking(move || {
            let _lease = lease;
            staging::read_upload_blob(&upload_copy, &entry_copy)
        })
        .await
        .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
        .map_err(str::to_owned)?;
        broker
            .upload_cloud_blob(
                generation,
                &binding.organization_id,
                &binding.project_id,
                &entry.digest,
                bytes,
            )
            .await
            .map_err(|error| error.to_string())?;
    }
    ensure_context(hub, broker, generation, scope, local, binding).await?;
    let manifest = serde_json::to_value(&upload.manifest)
        .map_err(|_| "hub_cloud_sync_snapshot_invalid".to_owned())?;
    let operation_id = upload.operation_id.clone();
    let result = broker
        .service_request(
            generation,
            ServiceRequest::CloudCommit {
                organization: binding.organization_id.clone(),
                project: binding.project_id.clone(),
                operation_id: operation_id.clone(),
                base_revision: upload.base_revision.clone(),
                manifest,
            },
        )
        .await;
    let (cleanup_view, cleanup_snapshot) = broker.cloud_cleanup_snapshot().await;
    let terminal = recovery_snapshot_matches_scope(&cleanup_view, generation, scope)
        && cleanup_snapshot.ok().is_some_and(|(_, operations, _)| {
            operations.iter().any(|operation| {
                operation.operation_id == operation_id
                    && operation.organization_id == binding.organization_id
                    && operation.project_id == binding.project_id
                    && operation.status.is_terminal()
            })
        });
    if terminal {
        // Unknown receipts keep the immutable upload snapshot so the same operation ID can
        // be resumed. A durable terminal receipt no longer needs the local upload copy.
        let root = local.root.clone();
        let organization_id = binding.organization_id.clone();
        let project_id = binding.project_id.clone();
        let project_guid = local.guid;
        let operation_id = operation_id.clone();
        let lease = Arc::clone(project_lease);
        let cleanup = tokio::task::spawn_blocking(move || {
            let _lease = lease;
            staging::discard_upload_snapshot(
                &root,
                &operation_id,
                &organization_id,
                &project_id,
                project_guid,
                &scope_fingerprint,
                &_lease,
            )
        })
        .await
        .map_err(|_| "hub_cloud_sync_cleanup_pending".to_owned())?;
        cleanup.map_err(|_| "hub_cloud_sync_cleanup_pending".to_owned())?;
    }
    result.map_err(|error| error.to_string())
}

async fn resume_snapshot(
    operation_id: &str,
    generation: &str,
    local: &LocalProject,
    binding: &CloudProjectBinding,
    scope: &CloudAccountScope,
    hub: &HubCommandState,
    broker: &Arc<AccountBroker>,
    project_lease: &Arc<ProjectCloudSyncLease>,
) -> Result<Value, String> {
    let (_, recovery) = broker.recovery_snapshot().await;
    let (operations, _) = recovery.map_err(|error| error.to_string())?;
    if operations.iter().any(|operation| {
        operation.operation_id == operation_id
            && (operation.action != "cloud-commit" || operation.status != OperationStatus::Unknown)
    }) {
        return Err("hub_cloud_sync_stage_invalid".into());
    }
    push_snapshot(
        operation_id,
        None,
        generation,
        local,
        binding,
        scope,
        hub,
        broker,
        project_lease,
    )
    .await
}

async fn stage_download(
    stage_id: &str,
    scope_fingerprint: &str,
    expected_revision: &str,
    generation: &str,
    local: &LocalProject,
    binding: &CloudProjectBinding,
    scope: &CloudAccountScope,
    hub: &HubCommandState,
    broker: &Arc<AccountBroker>,
    project_lease: &Arc<ProjectCloudSyncLease>,
) -> Result<Value, String> {
    let head = fetch_head(generation, binding, broker).await?;
    let head = head.ok_or_else(|| "hub_cloud_sync_stage_invalid".to_owned())?;
    if head.revision != expected_revision {
        return Err("hub_cloud_sync_remote_changed".into());
    }
    ensure_context(hub, broker, generation, scope, local, binding).await?;
    let root = local.root.clone();
    let organization_id = binding.organization_id.clone();
    let project_id = binding.project_id.clone();
    let scope_fingerprint = scope_fingerprint.to_owned();
    let cleanup_scope_fingerprint = scope_fingerprint.clone();
    let current_revision = head.revision.clone();
    let lease = Arc::clone(project_lease);
    tokio::task::spawn_blocking(move || {
        let _lease = lease;
        staging::prune_stale_download_stages(
            &root,
            &organization_id,
            &project_id,
            &cleanup_scope_fingerprint,
            &current_revision,
            &_lease,
        )
    })
    .await
    .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
    .map_err(str::to_owned)?;
    let stage = staging::prepare_scoped_download_stage(
        &local.root,
        stage_id,
        &binding.organization_id,
        &binding.project_id,
        &head.revision,
        &head.manifest_digest,
        &scope_fingerprint,
        head.manifest.clone(),
    )
    .map_err(str::to_owned)?;
    for entry in &head.manifest.files {
        let bytes = broker
            .download_cloud_blob(
                generation,
                &binding.organization_id,
                &binding.project_id,
                &entry.digest,
            )
            .await
            .map_err(|error| error.to_string())?;
        let stage_copy = stage.clone();
        let entry_copy = entry.clone();
        let lease = Arc::clone(project_lease);
        tokio::task::spawn_blocking(move || {
            let _lease = lease;
            staging::store_download_blob(&stage_copy, &entry_copy, &bytes)
        })
        .await
        .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
        .map_err(str::to_owned)?;
    }
    let stage_copy = stage.clone();
    let lease = Arc::clone(project_lease);
    tokio::task::spawn_blocking(move || {
        let _lease = lease;
        staging::verify_staged_files(&stage_copy)
    })
    .await
    .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
    .map_err(str::to_owned)?;
    ensure_context(hub, broker, generation, scope, local, binding).await?;
    let latest = fetch_head(generation, binding, broker).await?;
    if latest
        .as_ref()
        .map(|head| (&head.revision, &head.manifest_digest))
        != Some((&head.revision, &head.manifest_digest))
    {
        return Err("hub_cloud_sync_remote_changed".into());
    }
    let total_bytes = head
        .manifest
        .files
        .iter()
        .map(|entry| entry.bytes)
        .sum::<u64>();
    Ok(json!({
        "status": "staged",
        "stageId": stage_id,
        "revision": head.revision,
        "manifestDigest": head.manifest_digest,
        "fileCount": head.manifest.files.len(),
        "totalBytes": total_bytes.to_string(),
    }))
}

async fn apply_download(
    stage_id: &str,
    expected_revision: &str,
    generation: &str,
    local: &LocalProject,
    binding: &CloudProjectBinding,
    scope: &CloudAccountScope,
    hub: &HubCommandState,
    broker: &Arc<AccountBroker>,
    project_lease: &Arc<ProjectCloudSyncLease>,
) -> Result<Value, String> {
    let head = fetch_head(generation, binding, broker).await?;
    let head = head.ok_or_else(|| "hub_cloud_sync_stage_invalid".to_owned())?;
    if head.revision != expected_revision {
        return Err("hub_cloud_sync_remote_changed".into());
    }
    let root = local.root.clone();
    let stage_id = stage_id.to_owned();
    let stage_id_for_load = stage_id.clone();
    let expected_revision_owned = expected_revision.to_owned();
    let binding = binding.clone();
    let guid = local.guid;
    let lease = Arc::clone(project_lease);
    let staged = tokio::task::spawn_blocking(move || {
        let _lease = lease;
        staging::load_staged_snapshot(&root, &stage_id_for_load, &expected_revision_owned)
    })
    .await
    .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
    .map_err(str::to_owned)?;
    if staged.organization_id != binding.organization_id
        || staged.project_id != binding.project_id
        || staged.manifest_digest != head.manifest_digest
        || staged.manifest != head.manifest
    {
        return Err("hub_cloud_sync_stage_invalid".into());
    }
    ensure_context(hub, broker, generation, scope, local, &binding).await?;
    let latest = fetch_head(generation, &binding, broker).await?;
    if latest
        .as_ref()
        .map(|latest| (&latest.revision, &latest.manifest_digest))
        != Some((&head.revision, &head.manifest_digest))
    {
        return Err("hub_cloud_sync_remote_changed".into());
    }
    ensure_context(hub, broker, generation, scope, local, &binding).await?;
    let root = local.root.clone();
    let revision = head.revision;
    let lease = Arc::clone(project_lease);
    let result = tokio::task::spawn_blocking(move || {
        let _lease = lease;
        staging::apply_additions(&root, guid, &staged, &revision, &_lease)
    })
    .await
    .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
    .map_err(str::to_owned)?;
    match result {
        staging::ApplyOutcome::Applied {
            applied_files,
            unchanged_files,
        } => {
            let root = local.root.clone();
            let stage_id_for_discard = stage_id.clone();
            let organization_id = binding.organization_id.clone();
            let project_id = binding.project_id.clone();
            let scope_fingerprint = staging::download_scope_fingerprint(
                local.guid,
                scope,
                &organization_id,
                &project_id,
            );
            let expected_revision = expected_revision.to_owned();
            let expected_revision_for_discard = expected_revision.clone();
            let lease = Arc::clone(project_lease);
            tokio::task::spawn_blocking(move || {
                let _lease = lease;
                staging::discard_bound_download_stage(
                    &root,
                    &stage_id_for_discard,
                    &organization_id,
                    &project_id,
                    &expected_revision_for_discard,
                    &scope_fingerprint,
                    &_lease,
                )
            })
            .await
            .map_err(|_| "hub_cloud_sync_stage_invalid".to_owned())?
            .map_err(str::to_owned)?;
            Ok(json!({
                "status": "applied",
                "stageId": stage_id,
                "revision": expected_revision,
                "appliedFiles": applied_files,
                "unchangedFiles": unchanged_files,
            }))
        }
        staging::ApplyOutcome::Conflict {
            paths,
            conflict_count,
        } => Ok(json!({
            "status": "conflict",
            "stageId": stage_id,
            "revision": expected_revision,
            "conflictCount": conflict_count,
            "paths": paths,
        })),
    }
}

async fn fetch_head(
    generation: &str,
    binding: &CloudProjectBinding,
    broker: &Arc<AccountBroker>,
) -> Result<Option<CloudHead>, String> {
    let value = broker
        .service_request(
            generation,
            ServiceRequest::CloudHead {
                organization: binding.organization_id.clone(),
                project: binding.project_id.clone(),
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    if value.is_null() {
        return Ok(None);
    }
    let head: CloudHead =
        serde_json::from_value(value).map_err(|_| "account_service_operation_failed".to_owned())?;
    if head.organization_id != binding.organization_id || head.project_id != binding.project_id {
        return Err("account_service_operation_failed".into());
    }
    Ok(Some(head))
}

async fn capture_local(
    local: LocalProject,
    project_lease: Arc<ProjectCloudSyncLease>,
) -> Result<snapshot::LocalSnapshot, String> {
    let _lease = project_lease;
    snapshot::capture(&local.root, local.guid)
        .await
        .map_err(str::to_owned)
}

fn load_upload(
    local: &LocalProject,
    binding: &CloudProjectBinding,
    operation_id: &str,
    scope_fingerprint: &str,
) -> Result<Option<staging::UploadSnapshot>, String> {
    staging::find_upload_snapshot(
        &local.root,
        operation_id,
        &binding.organization_id,
        &binding.project_id,
        local.guid,
        scope_fingerprint,
    )
    .map_err(str::to_owned)
}

fn selected_binding(
    hub: &HubCommandState,
    scope: &CloudAccountScope,
) -> Result<(LocalProject, CloudProjectBinding), String> {
    let session = hub
        .session()
        .map_err(|_| "hub_cloud_sync_project_unavailable".to_owned())?;
    let local = session
        .selected_local_cloud_project()
        .ok_or_else(|| "hub_cloud_sync_project_unavailable".to_owned())?;
    let binding = session
        .selected_cloud_binding(scope)
        .ok_or_else(|| "hub_cloud_sync_project_unavailable".to_owned())?;
    Ok((
        LocalProject {
            root: local.root,
            path_key: local.path_key,
            guid: local.guid,
        },
        binding,
    ))
}

async fn ensure_context(
    hub: &HubCommandState,
    broker: &Arc<AccountBroker>,
    generation: &str,
    scope: &CloudAccountScope,
    local: &LocalProject,
    binding: &CloudProjectBinding,
) -> Result<(), String> {
    let view = broker.view().await;
    let environment: CloudBindingEnvironment = scope.environment.clone();
    if super::cloud_binding::binding_account_scope(&view, generation, Some(&environment)).as_ref()
        != Some(scope)
    {
        return Err("account_session_expired".into());
    }
    let (current_local, current_binding) = selected_binding(hub, scope)?;
    if &current_local != local || &current_binding != binding {
        return Err("hub_cloud_sync_project_changed".into());
    }
    Ok(())
}

fn request_generation(request: &AccountActionRequest) -> &str {
    match request {
        AccountActionRequest::CloudPush { generation, .. }
        | AccountActionRequest::CloudResumePush { generation, .. }
        | AccountActionRequest::CloudStageDownload { generation, .. }
        | AccountActionRequest::CloudApplyDownload { generation, .. }
        | AccountActionRequest::CloudDiscardDownload { generation, .. } => generation,
        _ => unreachable!("only cloud sync actions enter this handler"),
    }
}

fn recovery_snapshot_matches_scope(
    recovery_view: &AccountView,
    generation: &str,
    scope: &CloudAccountScope,
) -> bool {
    super::cloud_binding::binding_account_scope(recovery_view, generation, Some(&scope.environment))
        .as_ref()
        == Some(scope)
}

async fn guarded_recovery_cleanup<T, Recovery, Cleanup, CleanupFuture>(
    recovery_snapshot: Recovery,
    generation: &str,
    scope: &CloudAccountScope,
    cleanup: Cleanup,
) -> Result<T, String>
where
    Recovery: Future<
        Output = (
            AccountView,
            Result<(Vec<OperationSummary>, Vec<CloudCommitCleanupRecord>, String), AccountError>,
        ),
    >,
    Cleanup: FnOnce(Vec<OperationSummary>, Vec<CloudCommitCleanupRecord>) -> CleanupFuture,
    CleanupFuture: Future<Output = Result<T, String>>,
{
    let (recovery_view, recovery) = recovery_snapshot.await;
    if !recovery_snapshot_matches_scope(&recovery_view, generation, scope) {
        return Err("account_session_expired".into());
    }
    let (operations, cloud_operations, _) = recovery.map_err(|error| error.to_string())?;
    cleanup(operations, cloud_operations).await
}

#[cfg(test)]
#[path = "tests/cloud_sync.rs"]
mod tests;
