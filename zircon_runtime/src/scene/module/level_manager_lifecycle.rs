use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::core::framework::scene::WorldHandle;
use crate::core::{CoreError, CoreWeak};
use crate::scene::world::World;

use super::DefaultLevelManager;
use crate::scene::{LevelMetadata, LevelSystem};

/// A fully initialized level that is not visible to the runtime registry until publication.
pub struct PreparedLevel {
    manager: Arc<DefaultLevelManager>,
    level: Option<LevelSystem>,
}

impl PreparedLevel {
    pub(super) fn new(manager: Arc<DefaultLevelManager>, level: LevelSystem) -> Self {
        Self {
            manager,
            level: Some(level),
        }
    }

    pub fn publish(mut self) -> PreparedLevelPublication {
        let level = self
            .level
            .take()
            .expect("a prepared level can be published at most once");
        let previous = self
            .manager
            .lock_levels()
            .insert(level.handle(), level.clone());
        debug_assert!(
            previous.is_none(),
            "prepared level handles are globally unique"
        );
        PreparedLevelPublication {
            manager: Arc::clone(&self.manager),
            level: Some(level),
            committed: false,
        }
    }
}

/// Removes a published level on drop until its consumer commits the publication.
pub struct PreparedLevelPublication {
    manager: Arc<DefaultLevelManager>,
    level: Option<LevelSystem>,
    committed: bool,
}

impl PreparedLevelPublication {
    pub fn level(&self) -> &LevelSystem {
        self.level
            .as_ref()
            .expect("published level remains available until commit")
    }

    pub fn commit(mut self) -> LevelSystem {
        self.committed = true;
        self.level
            .take()
            .expect("published level remains available until commit")
    }
}

impl Drop for PreparedLevelPublication {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        if let Some(level) = self.level.as_ref() {
            self.manager.lock_levels().remove(&level.handle());
        }
    }
}

impl DefaultLevelManager {
    pub fn create_default_level(&self) -> LevelSystem {
        self.try_create_default_level()
            .expect("default level runtime extensions must apply")
    }

    pub fn try_create_default_level(&self) -> Result<LevelSystem, CoreError> {
        self.try_create_level(World::new(), LevelMetadata::default())
    }

    pub fn create_level(&self, world: World, metadata: LevelMetadata) -> LevelSystem {
        self.try_create_level(world, metadata)
            .expect("level runtime extensions must apply")
    }

    pub fn try_create_level(
        &self,
        world: World,
        metadata: LevelMetadata,
    ) -> Result<LevelSystem, CoreError> {
        let level = self.try_prepare_level(world, metadata)?;
        self.lock_levels().insert(level.handle(), level.clone());
        Ok(level)
    }

    pub(super) fn try_prepare_level(
        &self,
        mut world: World,
        metadata: LevelMetadata,
    ) -> Result<LevelSystem, CoreError> {
        let core = self.core.as_ref().and_then(CoreWeak::upgrade);
        let initial_time_policy = core
            .as_ref()
            .map(crate::core::CoreHandle::time_policy)
            .unwrap_or_default();
        if let Some(core) = &core {
            let driver = core.resolve_driver::<super::WorldDriver>(super::WORLD_DRIVER_NAME)?;
            driver.apply_world_runtime_extensions(&mut world)?;
        }
        let handle = self
            .next_handle
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map(|previous| WorldHandle::new(previous + 1))
            .map_err(|_| CoreError::LevelHandleExhausted)?;
        let level = LevelSystem::new(handle, Arc::new(Mutex::new(world)), metadata);
        level
            .initialize_time_policy(initial_time_policy)
            .map_err(|error| {
                CoreError::Initialization("WorldTimeController".to_string(), error.to_string())
            })?;
        Ok(level)
    }

    pub fn level(&self, handle: WorldHandle) -> Option<LevelSystem> {
        self.lock_levels().get(&handle).cloned()
    }

    pub(crate) fn try_for_each_world<E>(
        &self,
        mut operation: impl FnMut(&mut World) -> Result<(), E>,
    ) -> Result<(), E> {
        let levels = self.level_snapshots_in_handle_order();
        for level in levels {
            level.with_world_mut(&mut operation)?;
        }
        Ok(())
    }

    fn level_snapshots_in_handle_order(&self) -> Vec<LevelSystem> {
        self.lock_levels().values().cloned().collect()
    }

    pub(crate) fn sync_vm_types_atomically<T>(
        &self,
        registrations: &[zircon_runtime_interface::reflect::ReflectTypeRegistration],
        commit: impl FnOnce() -> T,
    ) -> crate::scene::SceneResult<T> {
        let levels = self.lock_levels();
        let ordered_levels = levels.values().cloned().collect::<Vec<_>>();
        let mut worlds = ordered_levels
            .iter()
            .map(LevelSystem::lock_world)
            .collect::<Vec<_>>();
        for world in &worlds {
            world.validate_vm_type_sync(registrations)?;
        }
        let snapshots = worlds
            .iter()
            .map(|world| (*world).clone())
            .collect::<Vec<_>>();
        for index in 0..worlds.len() {
            if let Err(error) = worlds[index].sync_vm_types(registrations) {
                for (world, snapshot) in worlds.iter_mut().zip(snapshots) {
                    **world = snapshot;
                }
                return Err(error);
            }
        }
        Ok(commit())
    }
}

#[cfg(test)]
#[path = "tests/level_manager_lifecycle.rs"]
mod tests;
