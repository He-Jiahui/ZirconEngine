use serde::{Deserialize, Serialize};

use super::{InternalSceneSystem, SceneSystemDescriptor, ScheduleError, SystemStage};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SceneSystemRegistry {
    systems: Vec<SceneSystemDescriptor>,
}

impl SceneSystemRegistry {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn with_builtin_systems() -> Self {
        let mut registry = Self::new();
        for descriptor in builtin_scene_systems() {
            registry
                .register_system(descriptor)
                .expect("built-in scene systems must have stable unique ids");
        }
        registry
    }

    pub fn register_system(
        &mut self,
        descriptor: SceneSystemDescriptor,
    ) -> Result<(), ScheduleError> {
        validate_system_descriptor(&descriptor)?;
        if self.systems.iter().any(|system| system.id == descriptor.id) {
            return Err(ScheduleError::DuplicateSystem(descriptor.id));
        }
        self.systems.push(descriptor);
        sort_systems(&mut self.systems);
        Ok(())
    }

    pub fn systems(&self) -> &[SceneSystemDescriptor] {
        &self.systems
    }

    pub fn systems_for_stage(
        &self,
        stage: SystemStage,
    ) -> impl Iterator<Item = &SceneSystemDescriptor> {
        self.systems
            .iter()
            .filter(move |system| system.stage == stage)
    }

    pub fn into_systems(self) -> Vec<SceneSystemDescriptor> {
        self.systems
    }
}

impl Default for SceneSystemRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn builtin_scene_systems() -> Vec<SceneSystemDescriptor> {
    vec![
        SceneSystemDescriptor::new(
            "zircon.scene.hierarchy_validity",
            SystemStage::PostUpdate,
            InternalSceneSystem::HierarchyValidity,
        )
        .with_order(-10_000),
        SceneSystemDescriptor::new(
            "zircon.scene.active_hierarchy",
            SystemStage::PostUpdate,
            InternalSceneSystem::ActiveHierarchy,
        )
        .with_order(-9_990),
        SceneSystemDescriptor::new(
            "zircon.scene.world_transform",
            SystemStage::PostUpdate,
            InternalSceneSystem::WorldTransform,
        )
        .with_order(-9_980),
        SceneSystemDescriptor::new(
            "zircon.scene.node_cache",
            SystemStage::PostUpdate,
            InternalSceneSystem::NodeCache,
        )
        .with_order(-9_970),
        SceneSystemDescriptor::new(
            "zircon.scene.render_extract_prepare",
            SystemStage::RenderExtract,
            InternalSceneSystem::RenderExtractPrepare,
        )
        .with_order(-10_000),
    ]
}

fn validate_system_descriptor(descriptor: &SceneSystemDescriptor) -> Result<(), ScheduleError> {
    if descriptor.id.trim().is_empty() || descriptor.id.trim() != descriptor.id {
        return Err(ScheduleError::EmptySystemId);
    }
    Ok(())
}

fn sort_systems(systems: &mut [SceneSystemDescriptor]) {
    systems.sort_by(|left, right| {
        left.stage
            .rank()
            .cmp(&right.stage.rank())
            .then(left.order.cmp(&right.order))
            .then(left.id.as_str().cmp(right.id.as_str()))
    });
}
