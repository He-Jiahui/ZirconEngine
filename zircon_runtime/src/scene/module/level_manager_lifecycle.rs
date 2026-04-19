use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use zircon_framework::scene::WorldHandle;
use zircon_scene::world::World;

use super::DefaultLevelManager;
use crate::scene::{LevelMetadata, LevelSystem};

impl DefaultLevelManager {
    pub fn create_default_level(&self) -> LevelSystem {
        self.create_level(World::new(), LevelMetadata::default())
    }

    pub fn create_level(&self, world: World, metadata: LevelMetadata) -> LevelSystem {
        let handle = WorldHandle::new(self.next_handle.fetch_add(1, Ordering::SeqCst) + 1);
        let level = LevelSystem::new(handle, Arc::new(Mutex::new(world)), metadata);
        self.levels.lock().unwrap().insert(handle, level.clone());
        level
    }

    pub fn level(&self, handle: WorldHandle) -> Option<LevelSystem> {
        self.levels.lock().unwrap().get(&handle).cloned()
    }
}
