use std::fmt;
use std::sync::Arc;

use crate::core::CoreError;
use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::ecs::{
    BoxedRuntimeSceneSystem, FunctionRuntimeSceneSystem, RuntimeSceneSystemContext,
    SceneSystemMetadata, SceneSystemTickPolicy, SystemOrderingConstraint, SystemRef, SystemSetId,
    SystemStage,
};

use super::super::owner::PluginModuleId;
use super::super::RuntimeExtensionRegistry;
use super::system_registration::validate_plugin_system_id;

type RuntimeSceneSystemBuildFn = Arc<dyn Fn() -> BoxedRuntimeSceneSystem + Send + Sync>;

#[derive(Clone)]
struct SharedRuntimeSceneSystemBuild {
    system_id: String,
    inner: RuntimeSceneSystemBuildFn,
}

impl SharedRuntimeSceneSystemBuild {
    fn new(system_id: impl Into<String>, build: RuntimeSceneSystemBuildFn) -> Self {
        Self {
            system_id: system_id.into(),
            inner: build,
        }
    }

    fn build(&self) -> BoxedRuntimeSceneSystem {
        (self.inner)()
    }
}

impl fmt::Debug for SharedRuntimeSceneSystemBuild {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SharedRuntimeSceneSystemBuild")
            .field("system_id", &self.system_id)
            .finish_non_exhaustive()
    }
}

/// 运行期系统注册行；计划应用到每个 World 时重新建立持有独立回调状态的实例。
#[derive(Clone, Debug)]
pub struct RuntimeSceneSystemRegistration {
    pub id: String,
    pub stage: SystemStage,
    pub sets: Vec<SystemSetId>,
    pub constraints: Vec<SystemOrderingConstraint>,
    pub order: i32,
    pub tick_policy: SceneSystemTickPolicy,
    build: SharedRuntimeSceneSystemBuild,
}

impl RuntimeSceneSystemRegistration {
    pub fn build(&self) -> BoxedRuntimeSceneSystem {
        self.build.build()
    }
}

/// 收集阶段、时钟和排序约束；`register` 后才进入目录，构建工厂可被多个 World 复用。
pub struct RuntimeSceneSystemRegistrationBuilder<'registry, S> {
    registry: &'registry mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
    id: String,
    stage: SystemStage,
    system_factory: Arc<dyn Fn() -> S + Send + Sync>,
    sets: Vec<SystemSetId>,
    constraints: Vec<SystemOrderingConstraint>,
    order: i32,
    tick_policy: SceneSystemTickPolicy,
}

impl<'registry, S> RuntimeSceneSystemRegistrationBuilder<'registry, S>
where
    S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
{
    pub(super) fn new(
        registry: &'registry mut RuntimeExtensionRegistry,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        system_factory: Arc<dyn Fn() -> S + Send + Sync>,
    ) -> Self {
        Self {
            registry,
            owner,
            id: id.into(),
            stage,
            system_factory,
            sets: Vec::new(),
            constraints: Vec::new(),
            order: 0,
            tick_policy: SceneSystemTickPolicy::for_stage(stage),
        }
    }

    pub fn in_set(mut self, set: SystemSetId) -> Self {
        self.sets.push(set);
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn with_tick_policy(mut self, tick_policy: SceneSystemTickPolicy) -> Self {
        self.tick_policy = tick_policy;
        self
    }

    pub fn before(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::Before(reference));
        self
    }

    pub fn after(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::After(reference));
        self
    }

    /// 拒绝与阶段不兼容的时钟策略；系统 ID 及与另一类系统的冲突由注册表统一核对。
    pub fn register(self) -> Result<(), RuntimeExtensionRegistryError> {
        if !self.tick_policy.is_valid_for_stage(self.stage) {
            return Err(RuntimeExtensionRegistryError::InvalidPluginSystem(format!(
                "{} has invalid tick policy {:?} for stage {:?}",
                self.id, self.tick_policy, self.stage
            )));
        }
        let id = self.id;
        let stage = self.stage;
        let order = self.order;
        let sets = self.sets;
        let constraints = self.constraints;
        let tick_policy = self.tick_policy;
        let system_factory = self.system_factory;
        let metadata = SceneSystemMetadata::new(id.clone(), stage, order)
            .with_sets(sets.clone())
            .with_constraints(constraints.clone())
            .with_tick_policy(tick_policy);
        let build = SharedRuntimeSceneSystemBuild::new(
            id.clone(),
            Arc::new(move || {
                let mut system = system_factory();
                let system: BoxedRuntimeSceneSystem = Box::new(FunctionRuntimeSceneSystem::new(
                    metadata.clone(),
                    move |context| system(context),
                ));
                system
            }),
        );
        self.registry.register_runtime_scene_system_registration(
            self.owner,
            RuntimeSceneSystemRegistration {
                id,
                stage,
                sets,
                constraints,
                order,
                tick_policy,
                build,
            },
        )
    }
}

impl RuntimeExtensionRegistry {
    /// Registers a factory that produces a fresh callback for every runtime scene-system instance.
    pub fn register_runtime_scene_system<S>(
        &mut self,
        owner: PluginModuleId,
        id: impl Into<String>,
        stage: SystemStage,
        system_factory: impl Fn() -> S + Send + Sync + 'static,
    ) -> RuntimeSceneSystemRegistrationBuilder<'_, S>
    where
        S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
    {
        RuntimeSceneSystemRegistrationBuilder::new(self, owner, id, stage, Arc::new(system_factory))
    }

    // TODO: [CR-PLUGIN-BOUNDARY-0105] 确认合并来的 owner 是否必须是已登记的同名运行时模块；本入口只查系统 ID 和重复键，缺少 owner 与 ID 的一致性检查；下一步覆盖错误 owner 的撤销与目标筛选。
    pub(crate) fn register_runtime_scene_system_registration(
        &mut self,
        owner: PluginModuleId,
        registration: RuntimeSceneSystemRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        validate_plugin_system_id(&registration.id)?;
        if self.plugin_runtime_systems.contains_key(&registration.id)
            || self.plugin_systems.contains_key(&registration.id)
        {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginSystem(
                registration.id,
            ));
        }
        self.plugin_runtime_systems
            .register(owner, registration.id.clone(), registration)
            .expect("runtime plugin system duplicate was prechecked");
        Ok(())
    }

    pub fn plugin_runtime_systems(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &RuntimeSceneSystemRegistration)> {
        self.plugin_runtime_systems
            .iter()
            .map(|(owner, _key, registration)| (owner, registration))
    }
}

#[cfg(test)]
#[path = "tests/runtime_scene_system_registration.rs"]
mod tests;
