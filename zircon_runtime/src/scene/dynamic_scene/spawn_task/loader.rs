use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::asset::{AssetUri, SceneAsset, project::ProjectManager};
use crate::core::{
    JobScheduler, TaskCancellationPolicy, TaskCancellationToken, TaskDescriptor,
    TaskGraphAdmissionError, TaskGraphScope, TaskHandle, TaskId, TaskPoolKind,
};

use super::super::{DynamicScene, DynamicSceneError};
use super::SpawnTaskResult;
use super::prepared::PreparedDynamicSceneSpawn;
use super::task::{DynamicSceneSpawnTask, lock_spawn_result};

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

    pub(crate) fn schedule_scene_asset_uri_with_limit_in_scope(
        scope: &TaskGraphScope,
        scheduler: &JobScheduler,
        project: ProjectManager,
        uri: AssetUri,
        label: impl Into<String>,
        max_prepared_scene_bytes: usize,
    ) -> Result<Self, TaskGraphAdmissionError> {
        Self::schedule_with_loader_in_scope(
            scope,
            scheduler,
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

    pub(crate) fn rejected(label: impl Into<String>, reason: impl Into<String>) -> Self {
        let label = label.into();
        let id = TaskId::new(NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID.fetch_add(1, Ordering::Relaxed));
        let descriptor = TaskDescriptor::new(id, TaskPoolKind::Compute, label.clone())
            .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop);
        let error = DynamicSceneError::SpawnTaskAdmission {
            label,
            reason: reason.into(),
        };
        // Admission rejection is a completed async wrapper with a business
        // error result; `Failed` is reserved for executor panics.
        let task = TaskHandle::completed(descriptor);
        Self {
            task,
            result: Arc::new(Mutex::new(Some(Err(error)))),
        }
    }

    fn schedule_with_loader(
        scheduler: &JobScheduler,
        label: impl Into<String>,
        max_prepared_scene_bytes: usize,
        loader: impl FnOnce() -> Result<DynamicScene, DynamicSceneError> + Send + 'static,
    ) -> Self {
        let id = TaskId::new(NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID.fetch_add(1, Ordering::Relaxed));
        let descriptor = TaskDescriptor::new(id, TaskPoolKind::Compute, label)
            .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop);
        let task_label = descriptor.label.clone();
        let result = Arc::new(Mutex::new(None));

        let result_for_task = Arc::clone(&result);
        let task = TaskHandle::schedule_detached(scheduler, descriptor, move |token| {
            run_loader(
                result_for_task,
                task_label,
                max_prepared_scene_bytes,
                loader,
                token,
            );
        });

        Self { task, result }
    }

    fn schedule_with_loader_in_scope(
        scope: &TaskGraphScope,
        scheduler: &JobScheduler,
        label: impl Into<String>,
        max_prepared_scene_bytes: usize,
        loader: impl FnOnce() -> Result<DynamicScene, DynamicSceneError> + Send + 'static,
    ) -> Result<Self, TaskGraphAdmissionError> {
        let id = TaskId::new(NEXT_DYNAMIC_SCENE_SPAWN_TASK_ID.fetch_add(1, Ordering::Relaxed));
        let descriptor = TaskDescriptor::new(id, TaskPoolKind::Compute, label)
            .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop);
        let task_label = descriptor.label.clone();
        let result = Arc::new(Mutex::new(None));

        let result_for_task = Arc::clone(&result);
        let task = scope.schedule(scheduler, descriptor, move |token| {
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
mod tests {
    use std::sync::mpsc;
    use std::{thread, time::Duration};

    use crate::core::TaskState;
    use crate::core::runtime::tasks::{
        EngineTaskGraph, EngineTaskGraphOptions, TaskGraphScopeDescriptor, TaskPools,
    };

    use super::*;

    fn test_job_scheduler() -> JobScheduler {
        JobScheduler::from_pool(TaskPools::process_default().compute().clone())
    }

    #[test]
    fn dynamic_scene_asset_reload_cancellation_prevents_running_loader_publication() {
        let scheduler = test_job_scheduler();
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
        assert_eq!(task.status_snapshot().state, TaskState::Running);
        assert!(!task.is_ready());
        release_tx.send(()).expect("loader should resume");
        task.wait();

        assert_eq!(task.status_snapshot().state, TaskState::Cancelled);
        assert!(matches!(
            task.take_ready(),
            Some(Err(DynamicSceneError::SpawnTaskCancelled { .. }))
        ));
    }

    #[test]
    fn dynamic_scene_prepare_worker_delay_matrix_completes_without_duplicate_publication() {
        for delay_ms in [0u64, 10, 1_000] {
            let scheduler = test_job_scheduler();
            let task = DynamicSceneSpawnTask::schedule_with_loader(
                &scheduler,
                format!("prepare-delay-{delay_ms}ms"),
                usize::MAX,
                move || {
                    thread::sleep(Duration::from_millis(delay_ms));
                    Ok(DynamicScene::empty())
                },
            );
            task.wait();

            assert_eq!(task.status_snapshot().state, TaskState::Completed);
            assert!(task.take_ready().is_some_and(|result| result.is_ok()));
            assert!(task.take_ready().is_some_and(|result| matches!(
                result,
                Err(DynamicSceneError::SpawnTaskResultUnavailable { .. })
            )));
        }
    }

    #[test]
    fn scoped_dynamic_scene_prepare_cancels_before_a_queued_loader_starts() {
        let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("explicit runtime should create its worker budget");
        let scope = runtime
            .create_scope(TaskGraphScopeDescriptor::new("dynamic-scene"))
            .expect("running runtime should admit its scene scope");
        let scheduler = JobScheduler::from_pool(runtime.worker_pool().clone());
        let (started_tx, started_rx) = mpsc::sync_channel(0);
        let (release_tx, release_rx) = mpsc::sync_channel(0);
        scheduler.schedule(move || {
            started_tx.send(()).expect("worker blocker should start");
            release_rx.recv().expect("worker blocker should release");
        });
        started_rx
            .recv()
            .expect("compute worker should be occupied");

        let (loader_tx, loader_rx) = mpsc::sync_channel(1);
        let task = DynamicSceneSpawnTask::schedule_with_loader_in_scope(
            &scope,
            &scheduler,
            "scope-cancelled-loader",
            usize::MAX,
            move || {
                loader_tx.send(()).expect("cancelled loader must not run");
                Ok(DynamicScene::empty())
            },
        )
        .expect("scope should admit the queued scene loader");

        scope.close_admission();
        release_tx.send(()).expect("worker blocker should release");
        task.wait();
        runtime
            .shutdown(Duration::from_secs(1))
            .expect("scoped task should drain before runtime shutdown");

        assert_eq!(task.status_snapshot().state, TaskState::Cancelled);
        assert!(loader_rx.try_recv().is_err());
        assert!(matches!(
            task.take_ready(),
            Some(Err(DynamicSceneError::SpawnTaskCancelled { .. }))
        ));
    }
}
