use serde::{Deserialize, Serialize};

use super::{SceneSystemDescriptor, SceneSystemRegistry, ScheduleError, SystemStage};

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
