use std::{cmp::Ordering, fmt};

use serde::{Deserialize, Serialize};

use super::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, InternalSceneSystem, IntoSceneSystem,
    RuntimeSceneSystem, SceneSystem, SceneSystemDescriptor, SceneSystemMetadata,
    ScheduleConflictGraph, ScheduleConflictNode, ScheduleError, SystemParam, SystemStage,
};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SceneSystemRegistry {
    systems: Vec<SceneSystemDescriptor>,
    #[serde(skip, default)]
    native_systems: Vec<BoxedSceneSystem>,
    #[serde(skip, default)]
    runtime_systems: Vec<BoxedRuntimeSceneSystem>,
}

impl SceneSystemRegistry {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            native_systems: Vec::new(),
            runtime_systems: Vec::new(),
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
        insert_system_sorted(&mut self.systems, descriptor);
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
        let system = match system.into_scene_system(metadata, world) {
            Ok(system) => system,
            Err(source) => {
                return Err(ScheduleError::SystemParam {
                    system_id: id,
                    source,
                });
            }
        };
        insert_native_system_sorted(&mut self.native_systems, system);
        Ok(())
    }

    pub(crate) fn register_boxed_native_system(
        &mut self,
        system: BoxedSceneSystem,
    ) -> Result<(), ScheduleError> {
        validate_system_id(system.id())?;
        self.ensure_unique_system_id(system.id())?;
        insert_native_system_sorted(&mut self.native_systems, system);
        Ok(())
    }

    pub(crate) fn register_boxed_runtime_system(
        &mut self,
        system: BoxedRuntimeSceneSystem,
    ) -> Result<(), ScheduleError> {
        validate_system_id(system.id())?;
        self.ensure_unique_system_id(system.id())?;
        insert_runtime_system_sorted(&mut self.runtime_systems, system);
        Ok(())
    }

    pub fn systems(&self) -> &[SceneSystemDescriptor] {
        &self.systems
    }

    pub fn systems_for_stage(
        &self,
        stage: SystemStage,
    ) -> impl Iterator<Item = &SceneSystemDescriptor> {
        SystemsForStage::new(&self.systems, stage)
    }

    pub fn native_systems_for_stage(
        &self,
        stage: SystemStage,
    ) -> impl Iterator<Item = &dyn SceneSystem> {
        NativeSystemsForStage::new(&self.native_systems, stage)
    }

    pub(crate) fn native_systems(&self) -> &[BoxedSceneSystem] {
        &self.native_systems
    }

    pub(crate) fn runtime_systems(&self) -> &[BoxedRuntimeSceneSystem] {
        &self.runtime_systems
    }

    #[cfg(test)]
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

    pub(crate) fn native_system_steps_by_stage(
        &self,
    ) -> [Vec<super::ScheduledSceneStep>; SystemStage::COUNT] {
        let native_step_counts =
            native_step_counts_by_stage(&self.native_systems, &self.runtime_systems);
        let mut by_stage = native_step_groups_with_capacity(&native_step_counts);
        for system in &self.native_systems {
            let steps = &mut by_stage[system.stage().rank()];
            steps.push(super::ScheduledSceneStep::native(
                system.id(),
                system.stage(),
                system.order(),
            ));
            if system.has_deferred_commands() {
                steps.push(super::ScheduledSceneStep::apply_deferred_after(
                    system.id(),
                    system.stage(),
                    system.order(),
                ));
            }
        }
        for system in &self.runtime_systems {
            let steps = &mut by_stage[system.stage().rank()];
            steps.push(super::ScheduledSceneStep::runtime(
                system.id(),
                system.stage(),
                system.order(),
            ));
        }
        by_stage
    }

    pub fn native_system_conflict_graph_for_stage(
        &self,
        stage: SystemStage,
    ) -> ScheduleConflictGraph {
        let node_count = native_conflict_graph_node_count_for_stage(
            &self.native_systems,
            &self.runtime_systems,
            stage,
        );
        let mut nodes = Vec::with_capacity(node_count);
        for system in &self.native_systems {
            if system.stage() != stage {
                continue;
            }

            nodes.push(ScheduleConflictNode::new(
                system.id(),
                system.stage(),
                system.access().clone(),
            ));
            if system.has_deferred_commands() {
                nodes.push(ScheduleConflictNode::barrier(
                    apply_deferred_node_id(system.id()),
                    system.stage(),
                ));
            }
        }
        for system in &self.runtime_systems {
            if system.stage() != stage {
                continue;
            }

            nodes.push(ScheduleConflictNode::new(
                system.id(),
                system.stage(),
                system.access().clone(),
            ));
        }
        ScheduleConflictGraph::from_node_vec(nodes)
    }

    pub(crate) fn take_native_system(&mut self, id: &str) -> Option<BoxedSceneSystem> {
        let mut index = 0_usize;
        while index < self.native_systems.len() {
            if self.native_systems[index].id() == id {
                return Some(self.native_systems.remove(index));
            }
            index += 1;
        }
        None
    }

    pub(crate) fn remove_system(&mut self, id: &str) -> Option<SceneSystemDescriptor> {
        let mut index = 0_usize;
        while index < self.systems.len() {
            if self.systems[index].id == id {
                return Some(self.systems.remove(index));
            }
            index += 1;
        }
        None
    }

    pub(crate) fn remove_native_system(&mut self, id: &str) -> Option<BoxedSceneSystem> {
        let mut index = 0_usize;
        while index < self.native_systems.len() {
            if self.native_systems[index].id() == id {
                return Some(self.native_systems.remove(index));
            }
            index += 1;
        }
        None
    }

    pub(crate) fn take_runtime_system(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        let mut index = 0_usize;
        while index < self.runtime_systems.len() {
            if self.runtime_systems[index].id() == id {
                return Some(self.runtime_systems.remove(index));
            }
            index += 1;
        }
        None
    }

    pub(crate) fn remove_runtime_system(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        let mut index = 0_usize;
        while index < self.runtime_systems.len() {
            if self.runtime_systems[index].id() == id {
                return Some(self.runtime_systems.remove(index));
            }
            index += 1;
        }
        None
    }

    pub(crate) fn restore_native_system(&mut self, system: BoxedSceneSystem) {
        insert_native_system_sorted(&mut self.native_systems, system);
    }

    pub(crate) fn restore_runtime_system(&mut self, system: BoxedRuntimeSceneSystem) {
        insert_runtime_system_sorted(&mut self.runtime_systems, system);
    }

    pub fn into_systems(self) -> Vec<SceneSystemDescriptor> {
        self.systems
    }

    fn ensure_unique_system_id(&self, id: &str) -> Result<(), ScheduleError> {
        if registered_system_id_exists(&self.systems, id)
            || registered_native_system_id_exists(&self.native_systems, id)
            || registered_runtime_system_id_exists(&self.runtime_systems, id)
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
            runtime_systems: Vec::new(),
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

struct SystemsForStage<'registry> {
    systems: std::slice::Iter<'registry, SceneSystemDescriptor>,
    stage: SystemStage,
}

impl<'registry> SystemsForStage<'registry> {
    fn new(systems: &'registry [SceneSystemDescriptor], stage: SystemStage) -> Self {
        Self {
            systems: systems.iter(),
            stage,
        }
    }
}

impl<'registry> Iterator for SystemsForStage<'registry> {
    type Item = &'registry SceneSystemDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        for system in self.systems.by_ref() {
            if system.stage == self.stage {
                return Some(system);
            }
        }
        None
    }
}

struct NativeSystemsForStage<'registry> {
    systems: std::slice::Iter<'registry, BoxedSceneSystem>,
    stage: SystemStage,
}

impl<'registry> NativeSystemsForStage<'registry> {
    fn new(systems: &'registry [BoxedSceneSystem], stage: SystemStage) -> Self {
        Self {
            systems: systems.iter(),
            stage,
        }
    }
}

impl<'registry> Iterator for NativeSystemsForStage<'registry> {
    type Item = &'registry dyn SceneSystem;

    fn next(&mut self) -> Option<Self::Item> {
        for system in self.systems.by_ref() {
            if system.stage() == self.stage {
                return Some(system.as_ref());
            }
        }
        None
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

fn registered_system_id_exists(systems: &[SceneSystemDescriptor], id: &str) -> bool {
    for system in systems {
        if system.id == id {
            return true;
        }
    }
    false
}

fn registered_native_system_id_exists(systems: &[BoxedSceneSystem], id: &str) -> bool {
    for system in systems {
        if system.id() == id {
            return true;
        }
    }
    false
}

fn registered_runtime_system_id_exists(systems: &[BoxedRuntimeSceneSystem], id: &str) -> bool {
    for system in systems {
        if system.id() == id {
            return true;
        }
    }
    false
}

fn insert_system_sorted(
    systems: &mut Vec<SceneSystemDescriptor>,
    descriptor: SceneSystemDescriptor,
) {
    let insert_index = match systems
        .binary_search_by(|existing| compare_system_descriptors(existing, &descriptor))
    {
        Ok(index) | Err(index) => index,
    };
    systems.insert(insert_index, descriptor);
}

fn compare_system_descriptors(
    left: &SceneSystemDescriptor,
    right: &SceneSystemDescriptor,
) -> Ordering {
    left.stage
        .rank()
        .cmp(&right.stage.rank())
        .then(left.order.cmp(&right.order))
        .then(left.id.as_str().cmp(right.id.as_str()))
}

fn insert_native_system_sorted(systems: &mut Vec<BoxedSceneSystem>, system: BoxedSceneSystem) {
    let insert_index = match systems
        .binary_search_by(|existing| compare_native_systems(existing.as_ref(), system.as_ref()))
    {
        Ok(index) | Err(index) => index,
    };
    systems.insert(insert_index, system);
}

fn compare_native_systems(left: &dyn SceneSystem, right: &dyn SceneSystem) -> Ordering {
    left.stage()
        .rank()
        .cmp(&right.stage().rank())
        .then(left.order().cmp(&right.order()))
        .then(left.id().cmp(right.id()))
}

fn insert_runtime_system_sorted(
    systems: &mut Vec<BoxedRuntimeSceneSystem>,
    system: BoxedRuntimeSceneSystem,
) {
    let insert_index = match systems
        .binary_search_by(|existing| compare_runtime_systems(existing.as_ref(), system.as_ref()))
    {
        Ok(index) | Err(index) => index,
    };
    systems.insert(insert_index, system);
}

fn compare_runtime_systems(
    left: &dyn RuntimeSceneSystem,
    right: &dyn RuntimeSceneSystem,
) -> Ordering {
    left.stage()
        .rank()
        .cmp(&right.stage().rank())
        .then(left.order().cmp(&right.order()))
        .then(left.id().cmp(right.id()))
}

fn apply_deferred_node_id(system_id: &str) -> String {
    format!("apply_deferred:{system_id}")
}

fn native_conflict_graph_node_count_for_stage(
    systems: &[BoxedSceneSystem],
    runtime_systems: &[BoxedRuntimeSceneSystem],
    stage: SystemStage,
) -> usize {
    let mut count = 0_usize;
    for system in systems {
        if system.stage() != stage {
            continue;
        }
        count += if system.has_deferred_commands() { 2 } else { 1 };
    }
    for system in runtime_systems {
        if system.stage() == stage {
            count += 1;
        }
    }
    count
}

fn native_step_counts_by_stage(
    systems: &[BoxedSceneSystem],
    runtime_systems: &[BoxedRuntimeSceneSystem],
) -> [usize; SystemStage::COUNT] {
    let mut counts = [0_usize; SystemStage::COUNT];
    for system in systems {
        let step_count = if system.has_deferred_commands() { 2 } else { 1 };
        counts[system.stage().rank()] += step_count;
    }
    for system in runtime_systems {
        counts[system.stage().rank()] += 1;
    }
    counts
}

fn native_step_groups_with_capacity(
    native_step_counts: &[usize; SystemStage::COUNT],
) -> [Vec<super::ScheduledSceneStep>; SystemStage::COUNT] {
    std::array::from_fn(|stage_index| Vec::with_capacity(native_step_counts[stage_index]))
}
