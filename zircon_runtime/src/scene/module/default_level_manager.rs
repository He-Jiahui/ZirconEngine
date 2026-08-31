use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::AtomicU64;
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::core::framework::scene::WorldHandle;
use crate::core::runtime::TaskPool;
use crate::core::{CoreHandle, CoreWeak};

use crate::scene::{LevelSystem, world::SceneProjectError};

use super::scene_artifact_io::SceneArtifactIo;

pub struct DefaultLevelManager {
    pub(super) next_handle: AtomicU64,
    pub(super) levels: Mutex<HashMap<WorldHandle, LevelSystem>>,
    pub(super) core: Option<CoreWeak>,
    scene_io_pool: Option<TaskPool>,
    scene_artifact_io: OnceLock<SceneArtifactIo>,
}

impl Default for DefaultLevelManager {
    fn default() -> Self {
        Self {
            next_handle: AtomicU64::new(0),
            levels: Mutex::new(HashMap::new()),
            core: None,
            scene_io_pool: None,
            scene_artifact_io: OnceLock::new(),
        }
    }
}

impl fmt::Debug for DefaultLevelManager {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DefaultLevelManager")
            .field("has_core", &self.core.is_some())
            .field("has_scene_io_owner", &self.scene_io_pool.is_some())
            .field("scene_artifact_io", &self.scene_artifact_io.get())
            .finish_non_exhaustive()
    }
}

impl DefaultLevelManager {
    pub(super) fn with_core(core: &CoreHandle) -> Self {
        Self {
            next_handle: AtomicU64::new(0),
            levels: Mutex::new(HashMap::new()),
            core: Some(core.downgrade()),
            scene_io_pool: Some(core.task_graph().worker_pool().clone()),
            scene_artifact_io: OnceLock::new(),
        }
    }

    #[cfg(test)]
    pub(super) fn with_scene_io_pool(scene_io_pool: TaskPool) -> Self {
        Self {
            next_handle: AtomicU64::new(0),
            levels: Mutex::new(HashMap::new()),
            core: None,
            scene_io_pool: Some(scene_io_pool),
            scene_artifact_io: OnceLock::new(),
        }
    }

    pub(super) fn scene_artifact_io(&self) -> Result<&SceneArtifactIo, SceneProjectError> {
        let pool = self
            .scene_io_pool
            .as_ref()
            .ok_or(SceneProjectError::RuntimeUnavailable)?;
        Ok(self
            .scene_artifact_io
            .get_or_init(|| SceneArtifactIo::new(pool.clone())))
    }

    pub(super) fn lock_levels(&self) -> MutexGuard<'_, HashMap<WorldHandle, LevelSystem>> {
        self.levels
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
