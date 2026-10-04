use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::asset::{project::ProjectManager, AssetUri, SceneAsset};
use crate::core::{
    JobScheduler, TaskCancellationPolicy, TaskCancellationToken, TaskDescriptor,
    TaskGraphAdmissionError, TaskGraphScope, TaskHandle, TaskId,
};

use super::super::{DynamicScene, DynamicSceneError};
use super::prepared::PreparedDynamicSceneSpawn;
use super::task::{lock_spawn_result, DynamicSceneSpawnTask};
use super::SpawnTaskResult;

static NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID: AtomicU64 = AtomicU64::new(1);

impl DynamicSceneSpawnTask {
    pub fn schedule_scene(
        scheduler: &JobScheduler,
        scene: DynamicScene,
        label: impl Into<String>,
    ) -> Result<Self, TaskGraphAdmissionError> {
        Self::schedule_with_loader(scheduler, label, usize::MAX, move || Ok(scene))
    }

    pub fn schedule_json(
        scheduler: &JobScheduler,
        json: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<Self, TaskGraphAdmissionError> {
        let json = json.into();
        Self::schedule_with_loader(scheduler, label, usize::MAX, move || {
            DynamicScene::from_versioned_json(&json)
        })
    }

    pub fn schedule_json_from_path(
        scheduler: &JobScheduler,
        path: impl Into<PathBuf>,
        label: impl Into<String>,
    ) -> Result<Self, TaskGraphAdmissionError> {
        let path = path.into();
        Self::schedule_with_loader(scheduler, label, usize::MAX, move || {
            let json = std::fs::read_to_string(&path).map_err(|error| DynamicSceneError::Io {
                reason: format!("{}: {error}", path.display()),
            })?;
            DynamicScene::from_versioned_json(&json)
        })
    }

    pub fn schedule_scene_asset(
        scheduler: &JobScheduler,
        project: ProjectManager,
        asset: SceneAsset,
        label: impl Into<String>,
    ) -> Result<Self, TaskGraphAdmissionError> {
        Self::schedule_with_loader(scheduler, label, usize::MAX, move || {
            DynamicScene::from_scene_asset(&project, &asset)
        })
    }

    pub fn schedule_scene_asset_uri(
        scheduler: &JobScheduler,
        project: ProjectManager,
        uri: AssetUri,
        label: impl Into<String>,
    ) -> Result<Self, TaskGraphAdmissionError> {
        Self::schedule_with_loader(scheduler, label, usize::MAX, move || {
            DynamicScene::from_scene_asset_uri(&project, &uri)
        })
    }

    pub(crate) fn schedule_scene_asset_uri_with_limit(
        scheduler: &JobScheduler,
        project: ProjectManager,
        uri: AssetUri,
        label: impl Into<String>,
        max_prepared_scene_bytes: usize,
    ) -> Result<Self, TaskGraphAdmissionError> {
        Self::schedule_with_loader(scheduler, label, max_prepared_scene_bytes, move || {
            DynamicScene::from_scene_asset_uri_with_raw_payload_limit(
                &project,
                &uri,
                u64::try_from(max_prepared_scene_bytes).unwrap_or(u64::MAX),
            )
        })
    }

    pub(crate) fn schedule_scene_asset_uri_with_limit_in_scope(
        scheduler: &JobScheduler,
        scope: &TaskGraphScope,
        project: ProjectManager,
        uri: AssetUri,
        label: impl Into<String>,
        max_prepared_scene_bytes: usize,
    ) -> Result<Self, TaskGraphAdmissionError> {
        Self::schedule_with_loader_in_scope(
            scheduler,
            scope,
            label,
            max_prepared_scene_bytes,
            move || {
                DynamicScene::from_scene_asset_uri_with_raw_payload_limit(
                    &project,
                    &uri,
                    u64::try_from(max_prepared_scene_bytes).unwrap_or(u64::MAX),
                )
            },
        )
    }

    fn schedule_with_loader(
        scheduler: &JobScheduler,
        label: impl Into<String>,
        max_prepared_scene_bytes: usize,
        loader: impl FnOnce() -> Result<DynamicScene, DynamicSceneError> + Send + 'static,
    ) -> Result<Self, TaskGraphAdmissionError> {
        let id = TaskId::new(NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID.fetch_add(1, Ordering::Relaxed));
        let descriptor = TaskDescriptor::new(id, scheduler.pool_kind(), label)
            .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop);
        let task_label = descriptor.label.clone();
        let result = Arc::new(Mutex::new(None));

        let result_for_task = Arc::clone(&result);
        let task = TaskHandle::try_schedule_detached(scheduler, descriptor, move |token| {
            run_loader(
                result_for_task,
                task_label,
                max_prepared_scene_bytes,
                loader,
                token,
            );
        })?;

        Ok(Self { task, result })
    }

    fn schedule_with_loader_in_scope(
        scheduler: &JobScheduler,
        scope: &TaskGraphScope,
        label: impl Into<String>,
        max_prepared_scene_bytes: usize,
        loader: impl FnOnce() -> Result<DynamicScene, DynamicSceneError> + Send + 'static,
    ) -> Result<Self, TaskGraphAdmissionError> {
        let id = TaskId::new(NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID.fetch_add(1, Ordering::Relaxed));
        let descriptor = TaskDescriptor::new(id, scheduler.pool_kind(), label)
            .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop);
        let task_label = descriptor.label.clone();
        let result = Arc::new(Mutex::new(None));

        let result_for_task = Arc::clone(&result);
        let task = scope.submit_on_scheduler(scheduler, descriptor, move |token| {
            run_loader(
                result_for_task,
                task_label,
                max_prepared_scene_bytes,
                loader,
                token,
            );
        })?;

        Ok(Self { task, result })
    }
}

fn run_loader(
    result: Arc<Mutex<Option<SpawnTaskResult>>>,
    task_label: String,
    max_prepared_scene_bytes: usize,
    loader: impl FnOnce() -> Result<DynamicScene, DynamicSceneError>,
    token: TaskCancellationToken,
) {
    if acknowledge_if_cancelled(&token) {
        return;
    }

    let prepared: SpawnTaskResult = loader().and_then(|scene| {
        if token.is_cancellation_requested() {
            return Err(DynamicSceneError::SpawnTaskCancelled {
                label: task_label.clone(),
            });
        }
        PreparedDynamicSceneSpawn::new_with_limit(scene, max_prepared_scene_bytes)
    });

    // 与 request_cancel 共用结果锁；最终取消确认与载荷发布在此互斥，已确认取消时只丢弃结果。
    let mut result = lock_spawn_result(&result);
    if acknowledge_if_cancelled(&token) {
        result.take();
    } else {
        *result = Some(prepared);
    }
}

fn acknowledge_if_cancelled(token: &TaskCancellationToken) -> bool {
    let cancelled = token.is_cancellation_requested();
    if cancelled {
        let acknowledged = token.acknowledge_cancellation();
        debug_assert!(
            acknowledged,
            "observed task cancellation must be acknowledged"
        );
    }
    cancelled
}

#[cfg(test)]
#[path = "tests/loader.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/loader_scheduler_owner_regression_tests.rs"]
mod scheduler_owner_regression_tests;
