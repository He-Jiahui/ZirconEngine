use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

#[cfg(test)]
use super::ScheduledSceneStep;
use super::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, IntoSceneSystem, IntoWorldlessSceneSystem,
    SceneScheduleStagePlan, SceneSystem, SceneSystemDescriptor, SceneSystemRegistry,
    ScheduleBuildReceipt, ScheduleConflictGraph, ScheduleError, SystemParam, SystemStage,
    WorldlessSystemParam,
};

#[derive(Debug, Serialize)]
pub struct Schedule {
    stages: Vec<SystemStage>,
    systems: SceneSystemRegistry,
    // Executor-facing cache rebuilt only when the schedule definition changes.
    #[serde(skip)]
    executor_plan: Arc<SceneScheduleStagePlan>,
    #[serde(skip)]
    executor_plan_dirty: bool,
    #[serde(skip)]
    taken_native_system_ids: Vec<String>,
    #[serde(skip)]
    taken_runtime_system_count: usize,
}

impl Schedule {
    pub fn register_system(
        &mut self,
        descriptor: SceneSystemDescriptor,
    ) -> Result<(), ScheduleError> {
        let system_id = descriptor.id.clone();
        self.ensure_id_not_taken(&descriptor.id)?;
        self.systems.register_system(descriptor)?;
        if let Err(error) = self.refresh_or_defer_executor_plan() {
            self.systems.remove_system(&system_id);
            self.refresh_or_defer_executor_plan()?;
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn register_boxed_runtime_system(
        &mut self,
        system: BoxedRuntimeSceneSystem,
    ) -> Result<(), ScheduleError> {
        let id = system.id().to_string();
        self.ensure_id_not_taken(&id)?;
        self.systems.register_boxed_runtime_system(system)?;
        if let Err(error) = self.refresh_or_defer_executor_plan() {
            self.systems.remove_runtime_system(&id);
            self.refresh_or_defer_executor_plan()?;
            return Err(error);
        }
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
        self.ensure_id_not_taken(&id)?;
        self.systems
            .register_native_system::<P, S>(id.clone(), stage, order, world, system)?;
        if let Err(error) = self.refresh_or_defer_executor_plan() {
            if let Some(mut system) = self.systems.remove_native_system(&id) {
                system.retire(world);
            }
            self.refresh_or_defer_executor_plan()?;
            return Err(error);
        }
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
        self.ensure_id_not_taken(&id)?;
        self.systems.register_worldless_native_system::<P, S>(
            id.clone(),
            stage,
            order,
            world,
            system,
        )?;
        if let Err(error) = self.refresh_or_defer_executor_plan() {
            if let Some(mut system) = self.systems.remove_native_system(&id) {
                system.retire(world);
            }
            self.refresh_or_defer_executor_plan()?;
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn register_boxed_native_system(
        &mut self,
        world: &mut crate::scene::World,
        system: BoxedSceneSystem,
    ) -> Result<(), ScheduleError> {
        let id = system.id().to_string();
        self.ensure_id_not_taken(&id)?;
        self.systems.register_boxed_native_system(system)?;
        if let Err(error) = self.refresh_or_defer_executor_plan() {
            if let Some(mut system) = self.systems.remove_native_system(&id) {
                system.retire(world);
            }
            self.refresh_or_defer_executor_plan()?;
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn unregister_native_system(
        &mut self,
        world: &mut crate::scene::World,
        id: &str,
    ) -> Result<bool, ScheduleError> {
        if taken_system_id_exists(&self.taken_native_system_ids, id) {
            return Err(ScheduleError::SystemInFlight(id.to_string()));
        }
        let Some(mut system) = self.systems.remove_native_system(id) else {
            return Ok(false);
        };
        if let Err(error) = self.refresh_or_defer_executor_plan() {
            self.systems.restore_native_system(system);
            return Err(error);
        }
        system.retire(world);
        Ok(true)
    }

    pub fn stages(&self) -> &[SystemStage] {
        &self.stages
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

    #[cfg(test)]
    pub(crate) fn native_system_steps_for_stage(
        &self,
        stage: SystemStage,
    ) -> Vec<ScheduledSceneStep> {
        self.executor_plan.native_steps_for_stage(stage).to_vec()
    }

    pub(crate) fn stage_plan(&self) -> Arc<SceneScheduleStagePlan> {
        Arc::clone(&self.executor_plan)
    }

    /// Returns the immutable receipt for the currently compiled execution graph.
    pub fn build_receipt(&self) -> ScheduleBuildReceipt {
        self.executor_plan.build_receipt()
    }

    pub(crate) fn take_native_system(&mut self, id: &str) -> Option<BoxedSceneSystem> {
        let system = self.systems.take_native_system(id)?;
        self.taken_native_system_ids.push(system.id().to_string());
        Some(system)
    }

    pub(crate) fn native_system_deferred_key(&self, id: &str) -> Option<super::DeferredSystemKey> {
        self.executor_plan.native_system_deferred_key(id)
    }

    pub(crate) fn take_runtime_system(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        let system = self.systems.take_runtime_system(id)?;
        self.taken_runtime_system_count += 1;
        Some(system)
    }

    pub(crate) fn restore_native_system(&mut self, system: BoxedSceneSystem) {
        let system_id = system.id();
        remove_taken_system_id(&mut self.taken_native_system_ids, system_id);
        self.systems.restore_native_system(system);
        if self.no_taken_systems() && self.executor_plan_dirty {
            self.refresh_executor_plan()
                .expect("deferred scene schedule refresh must stay valid");
        }
    }

    pub(crate) fn restore_runtime_system(&mut self, system: BoxedRuntimeSceneSystem) {
        debug_assert!(self.taken_runtime_system_count > 0);
        self.taken_runtime_system_count -= 1;
        self.systems.restore_runtime_system(system);
        if self.no_taken_systems() && self.executor_plan_dirty {
            self.refresh_executor_plan()
                .expect("deferred scene schedule refresh must stay valid");
        }
    }

    fn from_parts(stages: Vec<SystemStage>, systems: SceneSystemRegistry) -> Self {
        let executor_plan = Arc::new(
            SceneScheduleStagePlan::from_registry(&stages, &systems)
                .expect("default scene schedule must produce an executor plan"),
        );
        Self {
            stages,
            systems,
            executor_plan,
            executor_plan_dirty: false,
            taken_native_system_ids: Vec::new(),
            taken_runtime_system_count: 0,
        }
    }

    fn refresh_or_defer_executor_plan(&mut self) -> Result<(), ScheduleError> {
        if self.no_taken_systems() {
            self.refresh_executor_plan()?;
        } else {
            self.executor_plan_dirty = true;
        }
        Ok(())
    }

    fn refresh_executor_plan(&mut self) -> Result<(), ScheduleError> {
        self.executor_plan = Arc::new(SceneScheduleStagePlan::from_registry(
            &self.stages,
            &self.systems,
        )?);
        self.executor_plan_dirty = false;
        Ok(())
    }

    fn ensure_id_not_taken(&self, id: &str) -> Result<(), ScheduleError> {
        if taken_system_id_exists(&self.taken_native_system_ids, id)
            || self.systems.runtime_system_id_exists(id)
        {
            return Err(ScheduleError::DuplicateSystem(id.to_string()));
        }
        Ok(())
    }

    fn no_taken_systems(&self) -> bool {
        self.taken_native_system_ids.is_empty() && self.taken_runtime_system_count == 0
    }
}

fn taken_system_id_exists(taken_system_ids: &[String], id: &str) -> bool {
    for taken_id in taken_system_ids {
        if taken_id.as_str() == id {
            return true;
        }
    }
    false
}

fn remove_taken_system_id(taken_system_ids: &mut Vec<String>, id: &str) {
    let mut index = 0_usize;
    while index < taken_system_ids.len() {
        if taken_system_ids[index].as_str() == id {
            // Native taken IDs are only a membership guard; registry restore owns ordering.
            taken_system_ids.swap_remove(index);
            return;
        }
        index += 1;
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::from_parts(default_stage_order(), default_system_registry())
    }
}

impl Clone for Schedule {
    fn clone(&self) -> Self {
        Self::from_parts(self.stages.clone(), self.systems.clone())
    }
}

impl PartialEq for Schedule {
    fn eq(&self, other: &Self) -> bool {
        self.stages == other.stages && self.systems == other.systems
    }
}

impl Eq for Schedule {}

impl<'de> Deserialize<'de> for Schedule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ScheduleDocument {
            #[serde(default = "default_stage_order")]
            stages: Vec<SystemStage>,
            #[serde(default = "default_system_registry")]
            systems: SceneSystemRegistry,
        }

        let document = ScheduleDocument::deserialize(deserializer)?;
        Schedule::try_from_parts(document.stages, document.systems).map_err(D::Error::custom)
    }
}

impl Schedule {
    fn try_from_parts(
        stages: Vec<SystemStage>,
        systems: SceneSystemRegistry,
    ) -> Result<Self, ScheduleError> {
        let executor_plan = Arc::new(SceneScheduleStagePlan::from_registry(&stages, &systems)?);
        Ok(Self {
            stages,
            systems,
            executor_plan,
            executor_plan_dirty: false,
            taken_native_system_ids: Vec::new(),
            taken_runtime_system_count: 0,
        })
    }
}

pub fn default_stage_order() -> Vec<SystemStage> {
    let mut stages = Vec::with_capacity(SystemStage::ORDER.len());
    for stage in SystemStage::ORDER.iter().copied() {
        stages.push(stage);
    }
    stages
}

fn default_system_registry() -> SceneSystemRegistry {
    SceneSystemRegistry::with_builtin_systems()
}
