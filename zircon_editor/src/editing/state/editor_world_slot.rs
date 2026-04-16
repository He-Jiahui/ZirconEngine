use zircon_scene::{LevelSystem, Scene};

#[derive(Debug, Default)]
pub struct EditorWorldSlot {
    inner: Option<LevelSystem>,
}

impl EditorWorldSlot {
    #[allow(dead_code)]
    pub fn loaded(world: LevelSystem) -> Self {
        Self { inner: Some(world) }
    }

    pub fn unloaded() -> Self {
        Self { inner: None }
    }

    pub fn is_loaded(&self) -> bool {
        self.inner.is_some()
    }

    #[allow(dead_code)]
    pub fn snapshot(&self) -> Scene {
        self.inner
            .as_ref()
            .expect("editor world is not loaded")
            .snapshot()
    }

    pub fn try_snapshot(&self) -> Option<Scene> {
        self.inner.as_ref().map(LevelSystem::snapshot)
    }

    #[allow(dead_code)]
    pub fn with_world<R>(&self, read: impl FnOnce(&Scene) -> R) -> R {
        self.inner
            .as_ref()
            .expect("editor world is not loaded")
            .with_world(read)
    }

    pub fn try_with_world<R>(&self, read: impl FnOnce(&Scene) -> R) -> Option<R> {
        self.inner.as_ref().map(|world| world.with_world(read))
    }

    #[allow(dead_code)]
    pub fn with_world_mut<R>(&self, write: impl FnOnce(&mut Scene) -> R) -> R {
        self.inner
            .as_ref()
            .expect("editor world is not loaded")
            .with_world_mut(write)
    }

    pub fn try_with_world_mut<R>(&self, write: impl FnOnce(&mut Scene) -> R) -> Option<R> {
        self.inner.as_ref().map(|world| world.with_world_mut(write))
    }

    pub fn replace(&mut self, world: LevelSystem) {
        self.inner = Some(world);
    }

    pub fn clear(&mut self) {
        self.inner = None;
    }
}
