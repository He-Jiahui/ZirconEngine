use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Mutex, MutexGuard};

use crate::core::framework::scene::WorldHandle;
use crate::core::{CoreHandle, CoreWeak};

use crate::scene::LevelSystem;

#[derive(Debug, Default)]
pub struct DefaultLevelManager {
    pub(super) next_handle: AtomicU64,
    pub(super) levels: Mutex<HashMap<WorldHandle, LevelSystem>>,
    pub(super) core: Option<CoreWeak>,
}

impl DefaultLevelManager {
    pub(super) fn with_core(core: &CoreHandle) -> Self {
        Self {
            core: Some(core.downgrade()),
            ..Self::default()
        }
    }

    pub(super) fn lock_levels(&self) -> MutexGuard<'_, HashMap<WorldHandle, LevelSystem>> {
        self.levels
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
