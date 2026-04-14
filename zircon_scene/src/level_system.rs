//! Runtime level instance wrapping one ECS world plus lifecycle metadata.

use std::sync::{Arc, Mutex};

use zircon_manager::WorldHandle;

use crate::world::World;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelLifecycleState {
    Loaded,
    Unloaded,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelMetadata {
    pub project_root: Option<String>,
    pub asset_uri: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Clone)]
pub struct LevelSystem {
    handle: WorldHandle,
    inner: Arc<Mutex<World>>,
    metadata: Arc<Mutex<LevelMetadata>>,
    lifecycle: Arc<Mutex<LevelLifecycleState>>,
    subsystems: Arc<Mutex<Vec<String>>>,
}

impl LevelSystem {
    pub(crate) fn new(handle: WorldHandle, inner: Arc<Mutex<World>>, metadata: LevelMetadata) -> Self {
        Self {
            handle,
            inner,
            metadata: Arc::new(Mutex::new(metadata)),
            lifecycle: Arc::new(Mutex::new(LevelLifecycleState::Loaded)),
            subsystems: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn handle(&self) -> WorldHandle {
        self.handle
    }

    pub fn world_handle(&self) -> WorldHandle {
        self.handle
    }

    pub fn snapshot(&self) -> World {
        self.inner.lock().unwrap().clone()
    }

    pub fn replace(&self, world: World) {
        *self.inner.lock().unwrap() = world;
    }

    pub fn with_world<R>(&self, read: impl FnOnce(&World) -> R) -> R {
        let world = self.inner.lock().unwrap();
        read(&world)
    }

    pub fn with_world_mut<R>(&self, write: impl FnOnce(&mut World) -> R) -> R {
        let mut world = self.inner.lock().unwrap();
        write(&mut world)
    }

    pub fn metadata(&self) -> LevelMetadata {
        self.metadata.lock().unwrap().clone()
    }

    pub fn set_metadata(&self, metadata: LevelMetadata) {
        *self.metadata.lock().unwrap() = metadata;
    }

    pub fn lifecycle(&self) -> LevelLifecycleState {
        self.lifecycle.lock().unwrap().clone()
    }

    pub fn set_lifecycle(&self, lifecycle: LevelLifecycleState) {
        *self.lifecycle.lock().unwrap() = lifecycle;
    }

    pub fn register_subsystem(&self, subsystem_name: impl Into<String>) {
        self.subsystems.lock().unwrap().push(subsystem_name.into());
    }

    pub fn registered_subsystems(&self) -> Vec<String> {
        self.subsystems.lock().unwrap().clone()
    }
}

impl std::fmt::Debug for LevelSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LevelSystem")
            .field("handle", &self.handle)
            .field("metadata", &self.metadata())
            .field("lifecycle", &self.lifecycle())
            .finish()
    }
}
