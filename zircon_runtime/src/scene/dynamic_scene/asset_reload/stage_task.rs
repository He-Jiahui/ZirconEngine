use std::{
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use crate::{
    asset::{AssetEvent, SceneAsset},
    core::{JobHandle, JobScheduler, TaskState},
    scene::dynamic_scene::{DynamicSceneError, PreparedDynamicSceneSpawn, StagedDynamicSceneSpawn},
};

#[cfg(test)]
use crate::scene::dynamic_scene::DynamicSceneSpawnTargetSnapshot;

type TargetStageResult = Result<StagedDynamicSceneSpawn, DynamicSceneError>;

pub(super) struct DynamicSceneAssetReloadStageTask {
    event: AssetEvent<SceneAsset>,
    label: String,
    completion: JobHandle,
    result: Arc<Mutex<Option<TargetStageResult>>>,
    target_capture_elapsed: Arc<Mutex<Duration>>,
    cancel_requested: Arc<AtomicBool>,
    queued_at: Instant,
    reserved_bytes: Arc<AtomicUsize>,
    metadata_bytes: usize,
}

impl DynamicSceneAssetReloadStageTask {
    #[cfg(test)]
    pub(super) fn schedule(
        scheduler: &JobScheduler,
        event: AssetEvent<SceneAsset>,
        prepared: PreparedDynamicSceneSpawn,
        target: DynamicSceneSpawnTargetSnapshot,
    ) -> Self {
        let label = stage_label(&event);
        let reserved_bytes = Arc::new(AtomicUsize::new(
            prepared
                .estimated_bytes()
                .saturating_add(target.estimated_bytes()),
        ));
        let metadata_bytes = estimate_stage_task_metadata_bytes(&event);
        let result = Arc::new(Mutex::new(None));
        let target_capture_elapsed = Arc::new(Mutex::new(Duration::ZERO));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let task_result = Arc::clone(&result);
        let task_cancel = Arc::clone(&cancel_requested);
        let completion = scheduler.schedule(move || {
            if task_cancel.load(Ordering::Acquire) {
                return;
            }
            let staged = prepared.stage_target(target);
            if task_cancel.load(Ordering::Acquire) {
                return;
            }
            *lock_result(&task_result) = Some(staged);
        });
        Self {
            event,
            label,
            completion,
            result,
            target_capture_elapsed,
            cancel_requested,
            queued_at: Instant::now(),
            reserved_bytes,
            metadata_bytes,
        }
    }

    pub(super) fn schedule_for_level(
        scheduler: &JobScheduler,
        event: AssetEvent<SceneAsset>,
        prepared: PreparedDynamicSceneSpawn,
        level: crate::scene::LevelSystem,
        target_snapshot_limit_bytes: usize,
    ) -> Self {
        let label = stage_label(&event);
        let prepared_bytes = prepared.estimated_bytes();
        let initial_reserved_bytes = prepared_bytes.saturating_add(target_snapshot_limit_bytes);
        let reserved_bytes = Arc::new(AtomicUsize::new(initial_reserved_bytes));
        let metadata_bytes = estimate_stage_task_metadata_bytes(&event);
        let result = Arc::new(Mutex::new(None));
        let target_capture_elapsed = Arc::new(Mutex::new(Duration::ZERO));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let task_result = Arc::clone(&result);
        let task_capture_elapsed = Arc::clone(&target_capture_elapsed);
        let task_cancel = Arc::clone(&cancel_requested);
        let task_reserved_bytes = Arc::clone(&reserved_bytes);
        let completion = scheduler.schedule(move || {
            if task_cancel.load(Ordering::Acquire) {
                return;
            }
            let capture_started = Instant::now();
            let target = prepared.capture_level_target(&level, target_snapshot_limit_bytes);
            *lock_duration(&task_capture_elapsed) = capture_started.elapsed();
            let staged = match target {
                Ok(target) => {
                    let actual_reserved_bytes =
                        prepared_bytes.saturating_add(target.estimated_bytes());
                    debug_assert!(actual_reserved_bytes <= initial_reserved_bytes);
                    task_reserved_bytes.store(actual_reserved_bytes, Ordering::Release);
                    let staged = prepared.stage_target(target);
                    if staged.is_err() {
                        task_reserved_bytes.store(0, Ordering::Release);
                    }
                    staged
                }
                Err(error) => {
                    drop(prepared);
                    task_reserved_bytes.store(0, Ordering::Release);
                    Err(error)
                }
            };
            if task_cancel.load(Ordering::Acquire) {
                return;
            }
            *lock_result(&task_result) = Some(staged);
        });
        Self {
            event,
            label,
            completion,
            result,
            target_capture_elapsed,
            cancel_requested,
            queued_at: Instant::now(),
            reserved_bytes,
            metadata_bytes,
        }
    }

    pub(super) fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub(super) fn is_ready(&self) -> bool {
        self.completion.is_complete()
    }

    pub(super) fn state(&self) -> TaskState {
        if self.is_ready() {
            if self.cancel_requested.load(Ordering::Acquire) {
                TaskState::Cancelled
            } else {
                TaskState::Completed
            }
        } else {
            TaskState::Running
        }
    }

    pub(super) fn request_cancel(&self) -> bool {
        if self.is_ready() || self.cancel_requested.swap(true, Ordering::AcqRel) {
            return false;
        }
        lock_result(&self.result).take();
        true
    }

    pub(super) fn is_cancellation_requested(&self) -> bool {
        self.cancel_requested.load(Ordering::Acquire)
    }

    pub(super) fn take_ready(&self) -> Option<TargetStageResult> {
        if !self.is_ready() {
            return None;
        }
        if let Some(result) = lock_result(&self.result).take() {
            return Some(result);
        }
        Some(Err(if self.is_cancellation_requested() {
            DynamicSceneError::SpawnTaskCancelled {
                label: self.label.clone(),
            }
        } else {
            DynamicSceneError::SpawnTaskResultUnavailable {
                label: self.label.clone(),
            }
        }))
    }

    pub(super) fn age(&self) -> Duration {
        self.queued_at.elapsed()
    }

    pub(super) fn reserved_bytes(&self) -> usize {
        self.reserved_bytes.load(Ordering::Acquire)
    }

    pub(super) fn target_capture_elapsed(&self) -> Duration {
        *lock_duration(&self.target_capture_elapsed)
    }

    pub(super) fn estimated_metadata_bytes(&self) -> usize {
        self.metadata_bytes
    }
}

pub(super) fn estimate_stage_task_metadata_bytes(event: &AssetEvent<SceneAsset>) -> usize {
    std::mem::size_of::<DynamicSceneAssetReloadStageTask>()
        .saturating_add(stage_label(event).len())
        .saturating_add(event.locator().map_or(0, |uri| uri.to_string().len()))
        .saturating_add(
            event
                .previous_locator()
                .map_or(0, |uri| uri.to_string().len()),
        )
}

fn stage_label(event: &AssetEvent<SceneAsset>) -> String {
    format!(
        "dynamic-scene-target-stage:{:?}@{}",
        event.handle().id(),
        event.revision()
    )
}

impl Drop for DynamicSceneAssetReloadStageTask {
    fn drop(&mut self) {
        self.request_cancel();
    }
}

fn lock_result(
    result: &Mutex<Option<TargetStageResult>>,
) -> MutexGuard<'_, Option<TargetStageResult>> {
    result
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_duration(duration: &Mutex<Duration>) -> MutexGuard<'_, Duration> {
    duration
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
