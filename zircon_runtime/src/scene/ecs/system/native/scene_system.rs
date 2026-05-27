use std::fmt;

use crate::scene::ecs::{SceneSystemMetadata, SystemParamAccess, SystemStage};
use crate::scene::World;

pub type BoxedSceneSystem = Box<dyn SceneSystem>;

pub trait SceneSystem: Send + 'static {
    fn metadata(&self) -> &SceneSystemMetadata;
    fn access(&self) -> &SystemParamAccess;
    fn run(&mut self, world: &mut World);

    fn id(&self) -> &str {
        self.metadata().id()
    }

    fn stage(&self) -> SystemStage {
        self.metadata().stage()
    }

    fn order(&self) -> i32 {
        self.metadata().order()
    }

    fn has_deferred_commands(&self) -> bool {
        self.access().has_deferred_commands()
    }
}

impl fmt::Debug for dyn SceneSystem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneSystem")
            .field("id", &self.id())
            .field("stage", &self.stage())
            .field("order", &self.order())
            .field("has_deferred_commands", &self.has_deferred_commands())
            .finish_non_exhaustive()
    }
}
