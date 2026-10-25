use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::core::framework::scene::WorldHandle;
use crate::core::runtime::ServiceHandle;
use crate::core::{CoreError, CoreWeak};
use crate::scene::world::World;

use super::DefaultLevelManager;
use crate::scene::{LevelMetadata, LevelSystem};

/// A fully initialized level that is not visible to the runtime registry until publication.
/// The idle seed retains an exact service generation without holding call admission.
pub struct PreparedLevel {
    // Release an unsubmitted World/resource lease before its idle service token.
    level: Option<LevelSystem>,
    manager: ServiceHandle<DefaultLevelManager>,
}

/// Preserves the distinct admission and consumer installation failures.
#[derive(Debug)]
pub enum PreparedLevelPublishError<E> {
    Admission(CoreError),
    Install(E),
}

impl PreparedLevel {
    pub(super) fn new(manager: ServiceHandle<DefaultLevelManager>, level: LevelSystem) -> Self {
        Self {
            level: Some(level),
            manager,
        }
    }

    /// Publishes and installs the level under one exact-generation call admission.
    ///
    /// Installation runs outside the registry mutex. An error or panic removes the
    /// temporary registry entry and releases this transaction's level before admission
    /// drains. A successful consumer may retain its own LevelSystem resource lease.
    pub fn try_publish_with<E>(
        mut self,
        install: impl FnOnce(&LevelSystem) -> Result<(), E>,
    ) -> Result<LevelSystem, PreparedLevelPublishError<E>> {
        let manager = self
            .manager
            .enter()
            .map_err(PreparedLevelPublishError::Admission)?;
        let level = self
            .level
            .take()
            .expect("a prepared level can be published at most once");
        let publication = PreparedLevelRollback {
            manager: &manager,
            level: Some(level),
        };
        let previous = manager
            .lock_levels()
            .insert(publication.level().handle(), publication.level().clone());
        debug_assert!(
            previous.is_none(),
            "prepared level handles are globally unique"
        );
        install(publication.level()).map_err(PreparedLevelPublishError::Install)?;
        Ok(publication.commit())
    }
}

/// This rollback owner cannot outlive the publishing call's admitted manager.
struct PreparedLevelRollback<'a> {
    manager: &'a DefaultLevelManager,
    level: Option<LevelSystem>,
}

impl PreparedLevelRollback<'_> {
    fn level(&self) -> &LevelSystem {
        self.level
            .as_ref()
            .expect("published level remains available until commit")
    }

    fn commit(mut self) -> LevelSystem {
        self.level
            .take()
            .expect("published level remains available until commit")
    }
}

impl Drop for PreparedLevelRollback<'_> {
    fn drop(&mut self) {
        if let Some(level) = self.level.take() {
            let removed = self.manager.lock_levels().remove(&level.handle());
            // Drop World/resource leases outside the registry lock and before the
            // enclosing ServiceCallGuard announces that the publishing call drained.
            drop(removed);
            drop(level);
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
        namespaces: &std::collections::BTreeSet<String>,
        commit: impl FnOnce() -> T,
    ) -> crate::scene::SceneResult<T> {
        let levels = self.lock_levels();
        let ordered_levels = levels.values().cloned().collect::<Vec<_>>();
        let mut worlds = ordered_levels
            .iter()
            .map(LevelSystem::lock_world)
            .collect::<Vec<_>>();
        for world in &worlds {
            world.validate_vm_type_sync(registrations, namespaces)?;
        }
        let snapshots = worlds
            .iter()
            .map(|world| (*world).clone())
            .collect::<Vec<_>>();
        for index in 0..worlds.len() {
            if let Err(error) = worlds[index].sync_vm_types(registrations, namespaces) {
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
