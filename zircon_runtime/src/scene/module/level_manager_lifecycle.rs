use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::core::framework::scene::WorldHandle;
use crate::core::{CoreError, CoreWeak};
use crate::scene::world::World;

use super::DefaultLevelManager;
use crate::scene::{LevelMetadata, LevelSystem};

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
        mut world: World,
        metadata: LevelMetadata,
    ) -> Result<LevelSystem, CoreError> {
        let core = self.core.as_ref().and_then(CoreWeak::upgrade);
        let mut levels = self.lock_levels();
        if let Some(core) = core {
            let driver = core.resolve_driver::<super::WorldDriver>(super::WORLD_DRIVER_NAME)?;
            driver.apply_world_runtime_extensions(&mut world)?;
        }
        let handle = WorldHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst) + 1);
        let level = LevelSystem::new(handle, Arc::new(Mutex::new(world)), metadata);
        levels.insert(handle, level.clone());
        Ok(level)
    }

    pub fn level(&self, handle: WorldHandle) -> Option<LevelSystem> {
        self.lock_levels().get(&handle).cloned()
    }

    pub(crate) fn try_for_each_world<E>(
        &self,
        mut operation: impl FnMut(&mut World) -> Result<(), E>,
    ) -> Result<(), E> {
        let levels = self.lock_levels();
        for level in levels.values() {
            level.with_world_mut(&mut operation)?;
        }
        Ok(())
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
