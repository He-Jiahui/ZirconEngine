use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::{TaskDescriptor, TaskHandle, TaskStatus};

use super::super::DynamicSceneError;
use super::prepared::PreparedDynamicSceneSpawn;
use super::SpawnTaskResult;

/// Background scene-load/preparation task; world mutation stays on the caller thread.
#[derive(Debug)]
pub struct DynamicSceneSpawnTask {
    pub(super) task: TaskHandle,
    pub(super) result: Arc<Mutex<Option<SpawnTaskResult>>>,
}

impl DynamicSceneSpawnTask {
    pub fn descriptor(&self) -> &TaskDescriptor {
        self.task.descriptor()
    }

    pub fn status(&self) -> TaskStatus {
        self.task.status()
    }

    pub fn status_snapshot(&self) -> TaskStatus {
        self.task.status()
    }

    pub fn is_ready(&self) -> bool {
        self.task.is_complete()
    }

    pub fn wait(&self) {
        self.task.wait();
    }

    pub fn request_cancel(&self) -> bool {
        let status = self.task.status();
        if status.is_terminal() {
            return false;
        }
        let mut result = lock_spawn_result(&self.result);
        if result.is_some() {
            return false;
        }
        if self.task.is_cancellation_requested() {
            return false;
        }
        self.task.request_cancellation();
        result.take();
        true
    }

    pub fn is_cancellation_requested(&self) -> bool {
        self.task.is_cancellation_requested()
    }

    pub(crate) fn ready_estimated_bytes(&self) -> Option<usize> {
        if !self.is_ready() {
            return None;
        }
        lock_spawn_result(&self.result).as_ref().map(|result| {
            result
                .as_ref()
                .map_or(0, |prepared| prepared.estimated_bytes())
        })
    }

    /// 非终态返回 None；就绪结果槽只消费一次，空槽按取消标记返回取消错误或结果不可用错误。
    pub fn take_ready(&self) -> Option<Result<PreparedDynamicSceneSpawn, DynamicSceneError>> {
        if !self.is_ready() {
            return None;
        }
        if let Some(result) = lock_spawn_result(&self.result).take() {
            return Some(result);
        }
        let label = self.task.descriptor().label.clone();
        Some(if self.is_cancellation_requested() {
            Err(DynamicSceneError::SpawnTaskCancelled { label })
        } else {
            Err(DynamicSceneError::SpawnTaskResultUnavailable { label })
        })
    }

    pub fn wait_ready(self) -> Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        self.task.wait();
        let label = self.task.descriptor().label.clone();
        let result = lock_spawn_result(&self.result).take();
        result.unwrap_or_else(|| {
            if self.is_cancellation_requested() {
                Err(DynamicSceneError::SpawnTaskCancelled { label })
            } else {
                Err(DynamicSceneError::SpawnTaskResultUnavailable { label })
            }
        })
    }
}

impl Drop for DynamicSceneSpawnTask {
    fn drop(&mut self) {
        self.request_cancel();
    }
}

pub(super) fn lock_spawn_result(
    result: &Mutex<Option<SpawnTaskResult>>,
) -> MutexGuard<'_, Option<SpawnTaskResult>> {
    result
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
#[path = "tests/task.rs"]
mod tests;
