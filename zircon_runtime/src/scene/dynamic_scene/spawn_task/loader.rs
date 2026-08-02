use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::asset::{AssetUri, SceneAsset, project::ProjectManager};
use crate::core::JobScheduler;
use crate::core::framework::tasks::{
    AsyncTaskDescriptor, AsyncTaskHandle, AsyncTaskStatus, TaskCancellationPolicy, TaskPoolKind,
};

use super::super::{DynamicScene, DynamicSceneError};
use super::SpawnTaskResult;
use super::prepared::PreparedDynamicSceneSpawn;
use super::task::{DynamicSceneSpawnTask, lock_spawn_result, lock_spawn_status};

static NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID: AtomicU64 = AtomicU64::new(1);

impl DynamicSceneSpawnTask {
    pub fn schedule_scene(
        scheduler: &JobScheduler,
        scene: DynamicScene,
        label: impl Into<String>,
    ) -> Self {
        Self::schedule_with_loader(scheduler, label, usize::MAX, move || Ok(scene))
    }

    pub fn schedule_json(
        scheduler: &JobScheduler,
        json: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        let json = json.into();
        Self::schedule_with_loader(scheduler, label, usize::MAX, move || {
            DynamicScene::from_versioned_json(&json)
        })
    }

    pub fn schedule_json_from_path(
        scheduler: &JobScheduler,
        path: impl Into<PathBuf>,
        label: impl Into<String>,
    ) -> Self {
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
    ) -> Self {
        Self::schedule_with_loader(scheduler, label, usize::MAX, move || {
            DynamicScene::from_scene_asset(&project, &asset)
        })
    }

    pub fn schedule_scene_asset_uri(
        scheduler: &JobScheduler,
        project: ProjectManager,
        uri: AssetUri,
        label: impl Into<String>,
    ) -> Self {
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
    ) -> Self {
        Self::schedule_with_loader(scheduler, label, max_prepared_scene_bytes, move || {
            DynamicScene::from_scene_asset_uri_with_raw_payload_limit(
                &project,
                &uri,
                u64::try_from(max_prepared_scene_bytes).unwrap_or(u64::MAX),
            )
        })
    }

    fn schedule_with_loader(
        scheduler: &JobScheduler,
        label: impl Into<String>,
        max_prepared_scene_bytes: usize,
        loader: impl FnOnce() -> Result<DynamicScene, DynamicSceneError> + Send + 'static,
    ) -> Self {
        let handle =
            AsyncTaskHandle::new(NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID.fetch_add(1, Ordering::Relaxed));
        let descriptor = AsyncTaskDescriptor::new(handle, TaskPoolKind::Compute, label)
            .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop);
        let task_label = descriptor.label.clone();
        let status = Arc::new(Mutex::new(AsyncTaskStatus::pending(handle)));
        let result = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));

        let status_for_task = Arc::clone(&status);
        let result_for_task = Arc::clone(&result);
        let cancel_for_task = Arc::clone(&cancel_requested);
        let completion = scheduler.schedule(move || {
            {
                let mut status = lock_spawn_status(&status_for_task);
                if cancel_for_task.load(Ordering::Acquire) {
                    status.mark_cancelled();
                    return;
                }
                status.mark_running();
            }

            let prepared: SpawnTaskResult = loader().and_then(|scene| {
                if cancel_for_task.load(Ordering::Acquire) {
                    return Err(DynamicSceneError::SpawnTaskCancelled {
                        label: task_label.clone(),
                    });
                }
                PreparedDynamicSceneSpawn::new_with_limit(scene, max_prepared_scene_bytes)
            });
            {
                let mut status = lock_spawn_status(&status_for_task);
                let mut result = lock_spawn_result(&result_for_task);
                if cancel_for_task.load(Ordering::Acquire) {
                    result.take();
                    status.mark_cancelled();
                } else {
                    match &prepared {
                        Ok(_) => status.mark_completed(),
                        Err(error) => status.mark_failed(error.to_string()),
                    }
                    *result = Some(prepared);
                }
            }
        });

        Self {
            descriptor,
            status,
            completion,
            result,
            cancel_requested,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::{thread, time::Duration};

    use crate::core::framework::tasks::AsyncTaskState;

    use super::*;

    #[test]
    fn dynamic_scene_asset_reload_cancellation_prevents_running_loader_publication() {
        let scheduler = JobScheduler::default();
        let (started_tx, started_rx) = mpsc::sync_channel(0);
        let (release_tx, release_rx) = mpsc::sync_channel(0);
        let task = DynamicSceneSpawnTask::schedule_with_loader(
            &scheduler,
            "cancelled-running-loader",
            usize::MAX,
            move || {
                started_tx
                    .send(())
                    .expect("test should observe loader start");
                release_rx.recv().expect("test should release loader");
                Ok(DynamicScene::empty())
            },
        );

        started_rx.recv().expect("loader should start");
        assert!(task.request_cancel());
        assert_eq!(task.status_snapshot().state, AsyncTaskState::Running);
        assert!(!task.is_ready());
        release_tx.send(()).expect("loader should resume");
        task.completion_handle().wait();

        assert_eq!(task.status_snapshot().state, AsyncTaskState::Cancelled);
        assert!(matches!(
            task.take_ready(),
            Some(Err(DynamicSceneError::SpawnTaskCancelled { .. }))
        ));
    }

    #[test]
    fn dynamic_scene_prepare_worker_delay_matrix_completes_without_duplicate_publication() {
        for delay_ms in [0u64, 10, 1_000] {
            let scheduler = JobScheduler::default();
            let task = DynamicSceneSpawnTask::schedule_with_loader(
                &scheduler,
                format!("prepare-delay-{delay_ms}ms"),
                usize::MAX,
                move || {
                    thread::sleep(Duration::from_millis(delay_ms));
                    Ok(DynamicScene::empty())
                },
            );
            task.completion_handle().wait();

            assert_eq!(task.status_snapshot().state, AsyncTaskState::Completed);
            assert!(task.take_ready().is_some_and(|result| result.is_ok()));
            assert!(task.take_ready().is_some_and(|result| matches!(
                result,
                Err(DynamicSceneError::SpawnTaskResultUnavailable { .. })
            )));
        }
    }
}
