use crate::core::CoreHandle;
use crate::scene::LevelSystem;

/// Runtime-authoritative inputs available while one operation executes.
pub struct RuntimeOperationContext<'a> {
    core: &'a CoreHandle,
    level: &'a LevelSystem,
}

impl<'a> RuntimeOperationContext<'a> {
    pub fn new(core: &'a CoreHandle, level: &'a LevelSystem) -> Self {
        Self { core, level }
    }

    pub fn core(&self) -> &CoreHandle {
        self.core
    }

    pub fn level(&self) -> &LevelSystem {
        self.level
    }
}
