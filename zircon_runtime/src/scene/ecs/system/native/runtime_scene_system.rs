use std::fmt;

use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::scene::ecs::{
    SceneSystemMetadata, SystemOrderingConstraint, SystemParamAccess, SystemSetId, SystemStage,
};
use crate::scene::LevelSystem;

pub type BoxedRuntimeSceneSystem = Box<dyn RuntimeSceneSystem>;

pub struct RuntimeSceneSystemContext<'a> {
    pub core: &'a CoreHandle,
    pub level: &'a LevelSystem,
    pub delta_seconds: Real,
}

impl<'a> RuntimeSceneSystemContext<'a> {
    pub fn new(core: &'a CoreHandle, level: &'a LevelSystem, delta_seconds: Real) -> Self {
        Self {
            core,
            level,
            delta_seconds,
        }
    }
}

pub trait RuntimeSceneSystem: Send + 'static {
    fn metadata(&self) -> &SceneSystemMetadata;
    fn access(&self) -> &SystemParamAccess;
    fn run(&mut self, context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError>;

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
}

pub struct FunctionRuntimeSceneSystem<F> {
    metadata: SceneSystemMetadata,
    access: SystemParamAccess,
    system: F,
}

impl<F> FunctionRuntimeSceneSystem<F>
where
    F: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
{
    pub fn new(metadata: SceneSystemMetadata, system: F) -> Self {
        let mut access = SystemParamAccess::default();
        // Runtime scene systems can reach the LevelSystem and therefore the full World.
        access.add_conservative_world_access();
        Self {
            metadata,
            access,
            system,
        }
    }
}

impl<F> RuntimeSceneSystem for FunctionRuntimeSceneSystem<F>
where
    F: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
{
    fn metadata(&self) -> &SceneSystemMetadata {
        &self.metadata
    }

    fn access(&self) -> &SystemParamAccess {
        &self.access
    }

    fn run(&mut self, context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
        (self.system)(context)
    }
}

impl fmt::Debug for dyn RuntimeSceneSystem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeSceneSystem")
            .field("id", &self.id())
            .field("stage", &self.stage())
            .field("order", &self.order())
            .field(
                "conservative_world_access",
                &self.access().has_conservative_world_access(),
            )
            .finish_non_exhaustive()
    }
}
