use std::{cmp::Ordering, fmt};

use serde::{Deserialize, Serialize};

mod executable_returns;
mod runtime_system_slots;

pub(crate) use executable_returns::ExecutableReturnHandle;
use executable_returns::{ExecutableReturns, ReturnedSceneSystems};

use runtime_system_slots::{RuntimeSystemSlots, RuntimeSystems};

use super::scene_system_plan_descriptor::SceneSystemPlanDescriptor;

use super::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, InternalSceneSystem, IntoSceneSystem,
    IntoWorldlessSceneSystem, SceneSystem, SceneSystemDescriptor, SceneSystemMetadata,
    SceneSystemThreadAffinity, ScheduleConflictGraph, ScheduleError, SystemParam, SystemStage,
    WorldlessSystemParam,
};

// BUG: [CR-SCENE-ECS-0003] 反序列化直接填充 systems，绕过 register_system 的唯一 ID 校验；重复 ID 会使日程按 ID 解析目标时产生歧义。
/// 场景日程中的系统注册表；内建描述可序列化，原生与运行时系统仅在当前进程持有。
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SceneSystemRegistry {
    systems: Vec<SceneSystemDescriptor>,
    #[serde(skip, default)]
    native_systems: Vec<BoxedSceneSystem>,
    #[serde(skip, default)]
    runtime_systems: RuntimeSystemSlots,
    // Graph definitions survive take/restore; boxes own only executable state.
    #[serde(skip, default)]
    plan_systems: Vec<SceneSystemPlanDescriptor>,
    // Returned callbacks follow this registry across ownership moves.
    #[serde(skip, default)]
    executable_returns: ExecutableReturns,
}

impl SceneSystemRegistry {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            native_systems: Vec::new(),
            runtime_systems: RuntimeSystemSlots::new(),
            plan_systems: Vec::new(),
            executable_returns: ExecutableReturns::default(),
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

    /// 验证并注册内建系统描述；经 Schedule 调用时还会重编译拓扑，失败后撤销这次注册。
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
        insert_plan_system_sorted(
            &mut self.plan_systems,
            SceneSystemPlanDescriptor::native(system.as_ref()),
        );
        insert_native_system_sorted(&mut self.native_systems, system);
        Ok(())
    }

    pub fn register_worldless_native_system<P, S>(
        &mut self,
        id: impl Into<String>,
        stage: SystemStage,
        order: i32,
        world: &mut crate::scene::World,
        system: S,
    ) -> Result<(), ScheduleError>
    where
        P: WorldlessSystemParam + 'static,
        P::State: Send,
        S: IntoWorldlessSceneSystem<P>,
    {
        let id = id.into();
        validate_system_id(&id)?;
        self.ensure_unique_system_id(&id)?;
        let metadata = SceneSystemMetadata::new(id.clone(), stage, order)
            .with_thread_affinity(SceneSystemThreadAffinity::WorkerSafe);
        let system = system
            .into_worldless_scene_system(metadata, world)
            .map_err(|source| ScheduleError::SystemParam {
                system_id: id,
                source,
            })?;
        insert_plan_system_sorted(
            &mut self.plan_systems,
            SceneSystemPlanDescriptor::native(system.as_ref()),
        );
        insert_native_system_sorted(&mut self.native_systems, system);
        Ok(())
    }

    pub(crate) fn register_boxed_native_system(
        &mut self,
        system: BoxedSceneSystem,
    ) -> Result<(), ScheduleError> {
        validate_system_id(system.id())?;
        validate_native_system_execution_policy(
            system.id(),
            system.stage(),
            system.tick_policy(),
            system.has_deferred_commands(),
        )?;
        self.ensure_unique_system_id(system.id())?;
        insert_plan_system_sorted(
            &mut self.plan_systems,
            SceneSystemPlanDescriptor::native(system.as_ref()),
        );
        insert_native_system_sorted(&mut self.native_systems, system);
        Ok(())
    }

    pub(crate) fn register_boxed_runtime_system(
        &mut self,
        system: BoxedRuntimeSceneSystem,
    ) -> Result<(), ScheduleError> {
        validate_system_id(system.id())?;
        validate_system_tick_policy(system.id(), system.stage(), system.tick_policy())?;
        self.ensure_unique_system_id(system.id())?;
        insert_plan_system_sorted(
            &mut self.plan_systems,
            SceneSystemPlanDescriptor::runtime(system.as_ref()),
        );
        self.runtime_systems.insert(system);
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

    pub(super) fn plan_systems(&self) -> &[SceneSystemPlanDescriptor] {
        &self.plan_systems
    }

    pub(crate) fn runtime_systems(&self) -> RuntimeSystems<'_> {
        self.runtime_systems.iter()
    }

    pub(crate) fn runtime_system_id_exists(&self, id: &str) -> bool {
        self.runtime_systems.contains(id)
    }

    #[cfg(test)]
    pub(crate) fn native_system_steps_for_stage(
        &self,
        stage: SystemStage,
    ) -> Vec<super::ScheduledSceneStep> {
        let mut steps = Vec::new();
        for system in &self.plan_systems {
            if system.is_native() && system.stage() == stage {
                system.append_steps(system.order(), &mut steps);
            }
        }
        steps
    }

    pub(crate) fn native_system_steps_by_stage(
        &self,
    ) -> [Vec<super::ScheduledSceneStep>; SystemStage::COUNT] {
        let native_step_counts = native_step_counts_by_stage(&self.plan_systems);
        let mut by_stage = native_step_groups_with_capacity(&native_step_counts);
        for native in [true, false] {
            for system in &self.plan_systems {
                if system.is_native() == native {
                    system.append_steps(system.order(), &mut by_stage[system.stage().rank()]);
                }
            }
        }
        by_stage
    }

    pub fn native_system_conflict_graph_for_stage(
        &self,
        stage: SystemStage,
    ) -> ScheduleConflictGraph {
        let node_count = native_conflict_graph_node_count_for_stage(&self.plan_systems, stage);
        let mut nodes = Vec::with_capacity(node_count);
        // Match the native-before-runtime diagnostic order even while boxes are taken.
        for native in [true, false] {
            for system in &self.plan_systems {
                if system.is_native() == native && system.stage() == stage {
                    system.append_conflict_nodes(&mut nodes);
                }
            }
        }
        ScheduleConflictGraph::from_node_vec(nodes)
    }

    pub(crate) fn executable_return_handle(&self) -> ExecutableReturnHandle {
        self.executable_returns.handle()
    }

    pub(crate) fn take_returned_executables(&mut self) -> ReturnedSceneSystems {
        self.executable_returns.take()
    }

    pub(crate) fn returned_executables_cover(
        &self,
        native_ids: &[String],
        runtime_count: usize,
    ) -> bool {
        self.executable_returns.cover(native_ids, runtime_count)
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
                let system = self.native_systems.remove(index);
                self.remove_plan_system(id);
                return Some(system);
            }
            index += 1;
        }
        None
    }

    pub(crate) fn take_runtime_system(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        self.runtime_systems.take(id)
    }

    pub(crate) fn remove_runtime_system(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        let system = self.runtime_systems.remove(id)?;
        self.remove_plan_system(id);
        Some(system)
    }

    pub(crate) fn restore_native_system(&mut self, system: BoxedSceneSystem) {
        insert_native_system_sorted(&mut self.native_systems, system);
    }

    pub(crate) fn restore_removed_native_system(&mut self, system: BoxedSceneSystem) {
        // A failed permanent removal must restore its graph definition as well
        // as the executable box. Ordinary take/restore keeps that definition.
        insert_plan_system_sorted(
            &mut self.plan_systems,
            SceneSystemPlanDescriptor::native(system.as_ref()),
        );
        insert_native_system_sorted(&mut self.native_systems, system);
    }

    pub(crate) fn restore_runtime_system(&mut self, system: BoxedRuntimeSceneSystem) {
        self.runtime_systems.restore(system);
    }

    pub fn into_systems(self) -> Vec<SceneSystemDescriptor> {
        self.systems
    }

    fn remove_plan_system(&mut self, id: &str) {
        let mut index = 0_usize;
        while index < self.plan_systems.len() {
            if self.plan_systems[index].id() == id {
                self.plan_systems.remove(index);
                return;
            }
            index += 1;
        }
    }

    fn ensure_unique_system_id(&self, id: &str) -> Result<(), ScheduleError> {
        if registered_system_id_exists(&self.systems, id)
            || registered_plan_system_id_exists(&self.plan_systems, id)
            || self.runtime_systems.contains(id)
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
            runtime_systems: RuntimeSystemSlots::new(),
            plan_systems: Vec::new(),
            executable_returns: ExecutableReturns::default(),
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
            "zircon.scene.events_update_all",
            SystemStage::First,
            InternalSceneSystem::UpdateEvents,
        )
        .with_order(-10_000),
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

pub(crate) fn validate_system_tick_policy(
    system_id: &str,
    stage: SystemStage,
    tick_policy: super::SceneSystemTickPolicy,
) -> Result<(), ScheduleError> {
    if tick_policy.is_valid_for_stage(stage) {
        Ok(())
    } else {
        Err(ScheduleError::InvalidTickPolicy {
            system_id: system_id.to_string(),
            stage,
            tick_policy,
        })
    }
}

pub(crate) fn validate_native_system_execution_policy(
    system_id: &str,
    stage: SystemStage,
    tick_policy: super::SceneSystemTickPolicy,
    has_deferred_commands: bool,
) -> Result<(), ScheduleError> {
    validate_system_tick_policy(system_id, stage, tick_policy)?;
    if tick_policy.runs_when_virtual_paused() && has_deferred_commands {
        return Err(ScheduleError::PausedSystemDeferredCommands {
            system_id: system_id.to_string(),
            tick_policy,
        });
    }
    Ok(())
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

fn registered_plan_system_id_exists(systems: &[SceneSystemPlanDescriptor], id: &str) -> bool {
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

fn insert_plan_system_sorted(
    systems: &mut Vec<SceneSystemPlanDescriptor>,
    system: SceneSystemPlanDescriptor,
) {
    let insert_index = match systems.binary_search_by(|existing| {
        existing
            .stage()
            .rank()
            .cmp(&system.stage().rank())
            .then(existing.order().cmp(&system.order()))
            .then(existing.id().cmp(system.id()))
    }) {
        Ok(index) | Err(index) => index,
    };
    systems.insert(insert_index, system);
}

fn native_conflict_graph_node_count_for_stage(
    systems: &[SceneSystemPlanDescriptor],
    stage: SystemStage,
) -> usize {
    let mut count = 0_usize;
    for system in systems {
        if system.stage() == stage {
            count += system.step_count();
        }
    }
    count
}

fn native_step_counts_by_stage(
    systems: &[SceneSystemPlanDescriptor],
) -> [usize; SystemStage::COUNT] {
    let mut counts = [0_usize; SystemStage::COUNT];
    for system in systems {
        counts[system.stage().rank()] += system.step_count();
    }
    counts
}

fn native_step_groups_with_capacity(
    native_step_counts: &[usize; SystemStage::COUNT],
) -> [Vec<super::ScheduledSceneStep>; SystemStage::COUNT] {
    std::array::from_fn(|stage_index| Vec::with_capacity(native_step_counts[stage_index]))
}
