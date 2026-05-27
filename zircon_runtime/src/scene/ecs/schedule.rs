use serde::{Deserialize, Serialize};

use super::{
    BoxedSceneSystem, IntoSceneSystem, SceneSystem, SceneSystemDescriptor, SceneSystemRegistry,
    ScheduleConflictGraph, ScheduleError, ScheduledSceneStep, SystemParam, SystemStage,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    pub stages: Vec<SystemStage>,
    #[serde(default = "default_system_registry")]
    systems: SceneSystemRegistry,
}

impl Schedule {
    pub fn register_system(
        &mut self,
        descriptor: SceneSystemDescriptor,
    ) -> Result<(), ScheduleError> {
        self.systems.register_system(descriptor)
    }

    pub fn register_native_system<P, S>(
        &mut self,
        id: impl Into<String>,
        stage: SystemStage,
        order: i32,
        world: &mut crate::scene::World,
        system: S,
    ) -> Result<(), ScheduleError>
    where
        P: SystemParam + 'static,
        P::State: Send,
        S: IntoSceneSystem<P>,
    {
        self.systems
            .register_native_system::<P, S>(id, stage, order, world, system)
    }

    pub fn system_registry(&self) -> &SceneSystemRegistry {
        &self.systems
    }

    pub fn systems(&self) -> &[SceneSystemDescriptor] {
        self.systems.systems()
    }

    pub fn systems_for_stage(
        &self,
        stage: SystemStage,
    ) -> impl Iterator<Item = &SceneSystemDescriptor> {
        self.systems.systems_for_stage(stage)
    }

    pub fn native_systems_for_stage(
        &self,
        stage: SystemStage,
    ) -> impl Iterator<Item = &dyn SceneSystem> {
        self.systems.native_systems_for_stage(stage)
    }

    pub fn native_system_conflict_graph_for_stage(
        &self,
        stage: SystemStage,
    ) -> ScheduleConflictGraph {
        self.systems.native_system_conflict_graph_for_stage(stage)
    }

    pub(crate) fn native_system_steps_for_stage(
        &self,
        stage: SystemStage,
    ) -> Vec<ScheduledSceneStep> {
        self.systems.native_system_steps_for_stage(stage)
    }

    pub(crate) fn take_native_system(&mut self, id: &str) -> Option<BoxedSceneSystem> {
        self.systems.take_native_system(id)
    }

    pub(crate) fn restore_native_system(&mut self, system: BoxedSceneSystem) {
        self.systems.restore_native_system(system);
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            stages: default_stage_order(),
            systems: default_system_registry(),
        }
    }
}

pub fn default_stage_order() -> Vec<SystemStage> {
    vec![
        SystemStage::First,
        SystemStage::PreUpdate,
        SystemStage::FixedUpdate,
        SystemStage::Update,
        SystemStage::PostUpdate,
        SystemStage::Last,
        SystemStage::RenderExtract,
    ]
}

fn default_system_registry() -> SceneSystemRegistry {
    SceneSystemRegistry::with_builtin_systems()
}
