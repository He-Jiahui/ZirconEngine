use crate::core::CoreHandle;
use crate::scene::World;

/// Runtime-authoritative inputs available while one operation executes.
pub struct RuntimeOperationContext<'a> {
    core: &'a CoreHandle,
    world: &'a mut World,
}

impl<'a> RuntimeOperationContext<'a> {
    pub fn new(core: &'a CoreHandle, world: &'a mut World) -> Self {
        Self { core, world }
    }

    pub fn core(&self) -> &CoreHandle {
        self.core
    }

    pub fn world(&self) -> &World {
        self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }
}
