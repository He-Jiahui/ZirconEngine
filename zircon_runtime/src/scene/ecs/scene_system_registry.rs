use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    BoxedSceneSystem, InternalSceneSystem, IntoSceneSystem, SceneSystem, SceneSystemDescriptor,
    SceneSystemMetadata, ScheduleConflictGraph, ScheduleConflictNode, ScheduleError, SystemParam,
    SystemStage,
};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SceneSystemRegistry {
    systems: Vec<SceneSystemDescriptor>,
    #[serde(skip, default)]
    native_systems: Vec<BoxedSceneSystem>,
}

impl SceneSystemRegistry {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            native_systems: Vec::new(),
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
        self.ensure_unique_system_id(&descriptor.id)?;
        self.systems.push(descriptor);
        sort_systems(&mut self.systems);
        Ok(())
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
        let id = id.into();
        validate_system_id(&id)?;
        self.ensure_unique_system_id(&id)?;
        let metadata = SceneSystemMetadata::new(id.clone(), stage, order);
        let system = system
            .into_scene_system(metadata, world)
            .map_err(|source| ScheduleError::SystemParam {
                system_id: id,
                source,
            })?;
        self.native_systems.push(system);
        sort_native_systems(&mut self.native_systems);
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

    pub fn native_systems_for_stage(
        &self,
        stage: SystemStage,
    ) -> impl Iterator<Item = &dyn SceneSystem> {
        self.native_systems
            .iter()
            .map(|system| system.as_ref())
            .filter(move |system| system.stage() == stage)
    }

    pub(crate) fn native_system_steps_for_stage(
        &self,
        stage: SystemStage,
    ) -> Vec<super::ScheduledSceneStep> {
        self.native_systems
            .iter()
            .filter(|system| system.stage() == stage)
            .flat_map(|system| {
                let native_step =
                    super::ScheduledSceneStep::native(system.id(), system.stage(), system.order());
                let apply_deferred_step = system.has_deferred_commands().then(|| {
                    super::ScheduledSceneStep::apply_deferred_after(
                        system.id(),
                        system.stage(),
                        system.order(),
                    )
                });
                std::iter::once(native_step).chain(apply_deferred_step)
            })
            .collect()
    }

    pub fn native_system_conflict_graph_for_stage(
        &self,
        stage: SystemStage,
    ) -> ScheduleConflictGraph {
        ScheduleConflictGraph::from_nodes(
            self.native_systems
                .iter()
                .filter(|system| system.stage() == stage)
                .flat_map(|system| {
                    let system_node = ScheduleConflictNode::new(
                        system.id(),
                        system.stage(),
                        system.access().clone(),
                    );
                    let barrier_node = system.has_deferred_commands().then(|| {
                        ScheduleConflictNode::barrier(
                            apply_deferred_node_id(system.id()),
                            system.stage(),
                        )
                    });
                    std::iter::once(system_node).chain(barrier_node)
                }),
        )
    }

    pub(crate) fn take_native_system(&mut self, id: &str) -> Option<BoxedSceneSystem> {
        let index = self
            .native_systems
            .iter()
            .position(|system| system.id() == id)?;
        Some(self.native_systems.remove(index))
    }

    pub(crate) fn restore_native_system(&mut self, system: BoxedSceneSystem) {
        self.native_systems.push(system);
        sort_native_systems(&mut self.native_systems);
    }

    pub fn into_systems(self) -> Vec<SceneSystemDescriptor> {
        self.systems
    }

    fn ensure_unique_system_id(&self, id: &str) -> Result<(), ScheduleError> {
        if self.systems.iter().any(|system| system.id == id)
            || self.native_systems.iter().any(|system| system.id() == id)
        {
            return Err(ScheduleError::DuplicateSystem(id.to_string()));
        }
        Ok(())
    }
}

impl Clone for SceneSystemRegistry {
    fn clone(&self) -> Self {
        Self {
            systems: self.systems.clone(),
            native_systems: Vec::new(),
        }
    }
}

impl fmt::Debug for SceneSystemRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneSystemRegistry")
            .field("systems", &self.systems)
            .finish()
    }
}

impl PartialEq for SceneSystemRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.systems == other.systems
    }
}

impl Eq for SceneSystemRegistry {}

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
    validate_system_id(&descriptor.id)
}

fn validate_system_id(id: &str) -> Result<(), ScheduleError> {
    if id.trim().is_empty() || id.trim() != id {
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

fn sort_native_systems(systems: &mut [BoxedSceneSystem]) {
    systems.sort_by(|left, right| {
        left.stage()
            .rank()
            .cmp(&right.stage().rank())
            .then(left.order().cmp(&right.order()))
            .then(left.id().cmp(right.id()))
    });
}

fn apply_deferred_node_id(system_id: &str) -> String {
    format!("apply_deferred:{system_id}")
}
