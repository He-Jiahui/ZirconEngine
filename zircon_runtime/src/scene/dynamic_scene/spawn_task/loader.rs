use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::asset::{project::ProjectManager, AssetUri, SceneAsset};
use crate::core::framework::tasks::{
    AsyncTaskDescriptor, AsyncTaskHandle, AsyncTaskStatus, TaskCancellationPolicy, TaskPoolKind,
};
use crate::core::JobScheduler;

use super::super::{DynamicScene, DynamicSceneError};
use super::prepared::PreparedDynamicSceneSpawn;
use super::task::DynamicSceneSpawnTask;
use super::SpawnTaskResult;

static NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID: AtomicU64 = AtomicU64::new(1);

impl DynamicSceneSpawnTask {
    pub fn schedule_scene(
        scheduler: &JobScheduler,
        scene: DynamicScene,
        label: impl Into<String>,
    ) -> Self {
        Self::schedule_with_loader(scheduler, label, move || Ok(scene))
    }

    pub fn schedule_json(
        scheduler: &JobScheduler,
        json: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        let json = json.into();
        Self::schedule_with_loader(scheduler, label, move || {
            DynamicScene::from_versioned_json(&json)
        })
    }

    pub fn schedule_json_from_path(
        scheduler: &JobScheduler,
        path: impl Into<PathBuf>,
        label: impl Into<String>,
    ) -> Self {
        let path = path.into();
        Self::schedule_with_loader(scheduler, label, move || {
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
    ) -> Self {
        Self::schedule_with_loader(scheduler, label, move || {
            DynamicScene::from_scene_asset(&project, &asset)
        })
    }

    pub fn schedule_scene_asset_uri(
        scheduler: &JobScheduler,
        project: ProjectManager,
        uri: AssetUri,
        label: impl Into<String>,
    ) -> Self {
        Self::schedule_with_loader(scheduler, label, move || {
            DynamicScene::from_scene_asset_uri(&project, &uri)
        })
    }

    fn schedule_with_loader(
        scheduler: &JobScheduler,
        label: impl Into<String>,
        loader: impl FnOnce() -> Result<DynamicScene, DynamicSceneError> + Send + 'static,
    ) -> Self {
        let handle =
            AsyncTaskHandle::new(NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID.fetch_add(1, Ordering::Relaxed));
        let descriptor = AsyncTaskDescriptor::new(handle, TaskPoolKind::Compute, label)
            .with_cancellation_policy(TaskCancellationPolicy::DetachOnDrop);
        let status = Arc::new(Mutex::new(AsyncTaskStatus::pending(handle)));
        let result = Arc::new(Mutex::new(None));

        let status_for_task = Arc::clone(&status);
        let result_for_task = Arc::clone(&result);
        let completion = scheduler.schedule(move || {
            status_for_task
                .lock()
                .expect("dynamic scene spawn task status lock poisoned")
                .mark_running();

            let prepared: SpawnTaskResult = loader().and_then(PreparedDynamicSceneSpawn::new);
            {
                let mut status = status_for_task
                    .lock()
                    .expect("dynamic scene spawn task status lock poisoned");
                match &prepared {
                    Ok(_) => status.mark_completed(),
                    Err(error) => status.mark_failed(error.to_string()),
                }
            }
            *result_for_task
                .lock()
                .expect("dynamic scene spawn task result lock poisoned") = Some(prepared);
        });

        Self {
            descriptor,
            status,
            completion,
            result,
        }
    }
}
