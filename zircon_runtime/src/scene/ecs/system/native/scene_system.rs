use std::fmt;

use crate::scene::World;
use crate::scene::ecs::{
    SceneSystemMetadata, SceneSystemThreadAffinity, SystemOrderingConstraint, SystemParamAccess,
    SystemSetId, SystemStage,
};

pub type BoxedSceneSystem = Box<dyn SceneSystem>;

pub trait SceneSystem: Send + 'static {
    fn metadata(&self) -> &SceneSystemMetadata;
    fn access(&self) -> &SystemParamAccess;
    fn run(&mut self, world: &mut World);

    /// Runs a foreign callback that was registered without a `World` parameter. Only systems
    /// returning `true` from `supports_worldless_execution` may be dispatched to a worker.
    fn run_without_world(&mut self) {
        panic!("scene system does not support worldless execution")
    }

    fn supports_worldless_execution(&self) -> bool {
        false
    }

    fn id(&self) -> &str {
        self.metadata().id()
    }

    fn stage(&self) -> SystemStage {
        self.metadata().stage()
    }

    fn order(&self) -> i32 {
        self.metadata().order()
    }

    fn sets(&self) -> &[SystemSetId] {
        self.metadata().sets()
    }

    fn constraints(&self) -> &[SystemOrderingConstraint] {
        self.metadata().constraints()
    }

    fn thread_affinity(&self) -> SceneSystemThreadAffinity {
        self.metadata().thread_affinity()
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
            .field("thread_affinity", &self.thread_affinity())
            .finish_non_exhaustive()
    }
}
