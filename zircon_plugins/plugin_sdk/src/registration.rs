//! 将插件模块身份绑定到 Runtime 扩展注册表的作者 API；实际条目和撤销权仍归注册表。

use std::sync::Arc;

use zircon_runtime::core::framework::bridge::PluginInterface;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::{
    BridgeImport, PluginEventCatalogManifest, PluginEventManifest, PluginModuleId,
    PluginOptionManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};
use zircon_runtime::scene::ecs::{
    Event, Resource, RuntimeSceneSystemContext, SceneSystemClockDomain, SceneSystemTickPolicy,
    SystemOrderingConstraint, SystemRef, SystemStage,
};
use zircon_runtime::scene::world::SceneComponentSerializer;

/// 借用 Runtime 扩展注册表；`module` 从该注册表取得可追踪的 owner 身份。
pub struct RuntimePluginRegistrationBuilder<'registry> {
    registry: &'registry mut RuntimeExtensionRegistry,
}

impl<'registry> RuntimePluginRegistrationBuilder<'registry> {
    pub fn new(registry: &'registry mut RuntimeExtensionRegistry) -> Self {
        Self { registry }
    }

    /// 在注册表中登记模块名并返回独占借用；后续注册项均归此 owner。
    pub fn module(
        self,
        module_name: impl Into<String>,
    ) -> Result<RuntimePluginModuleRegistration<'registry>, RuntimeExtensionRegistryError> {
        let module_name = module_name.into();
        let owner = self.registry.intern_plugin_module(module_name.clone())?;
        Ok(RuntimePluginModuleRegistration {
            registry: self.registry,
            module_name,
            owner,
        })
    }
}

/// 在一个模块 owner 下提交系统、资源、事件、组件与接口注册项。
pub struct RuntimePluginModuleRegistration<'registry> {
    registry: &'registry mut RuntimeExtensionRegistry,
    module_name: String,
    owner: PluginModuleId,
}

impl<'registry> RuntimePluginModuleRegistration<'registry> {
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn owner(&self) -> PluginModuleId {
        self.owner
    }

    /// Registers a factory that produces a fresh callback for every runtime scene-system instance.
    /// 为每个 Runtime 场景系统实例创建独立回调；工厂须可跨线程共享，回调由系统实例持有。
    pub fn runtime_scene_system<S, F>(
        &mut self,
        id: impl Into<String>,
        stage: SystemStage,
        system_factory: F,
    ) -> RuntimePluginRuntimeSceneSystemBuilder<'_, F>
    where
        S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
        F: Fn() -> S + Send + Sync + 'static,
    {
        RuntimePluginRuntimeSceneSystemBuilder {
            registry: self.registry,
            owner: self.owner,
            id: id.into(),
            stage,
            system_factory,
            sets: Vec::new(),
            constraints: Vec::new(),
            order: 0,
            tick_policy: SceneSystemTickPolicy::for_stage(stage),
        }
    }

    /// Registers an immutable factory that creates one fresh resource value
    /// for every world initialized from the runtime extension plan.
    /// 保存可并发调用的工厂；扩展计划应用到每个 World 时分别创建资源值。
    pub fn resource<T>(
        &mut self,
        init: impl Fn() -> T + Send + Sync + 'static,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: Resource,
    {
        self.registry.register_resource::<T>(self.owner, init)
    }

    /// 组件声明必须与当前模块 owner 匹配；注册表拒绝伪造的其他插件归属。
    pub fn component(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.registry
            .register_component_for_owner(self.owner, descriptor)
    }

    /// Registers the owner-qualified scene codec that is projected through the linked runtime
    /// plugin catalog into every normal ProjectManager open. The descriptor must be registered
    /// first through [`Self::component`], preserving one owner and one identity boundary.
    pub fn scene_component_codec(
        &mut self,
        serializer: SceneComponentSerializer,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.registry
            .register_scene_component_codec_for_owner(self.owner, serializer)
    }

    pub fn event<E>(
        &mut self,
        manifest: PluginEventManifest,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        E: Event,
    {
        self.registry.register_event::<E>(self.owner, manifest)
    }

    // BUG: [CR-PLUGIN-SDK-0002] 选项键或事件目录前缀与当前模块不同时，注册表改按前缀归属，撤销当前模块会遗留条目；证据：register_plugin_option/register_plugin_event_catalog。
    /// 提交选项元数据；底层当前按选项键前缀计算 owner。
    pub fn plugin_option(
        &mut self,
        manifest: PluginOptionManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.registry.register_plugin_option(manifest)
    }

    /// 提交事件目录；底层当前按命名空间前缀计算 owner。
    pub fn plugin_event_catalog(
        &mut self,
        manifest: PluginEventCatalogManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.registry.register_plugin_event_catalog(manifest)
    }

    pub fn export_interface<T>(
        &mut self,
        implementation: Arc<T>,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        self.registry
            .export_interface::<T>(self.owner, implementation)
    }

    /// 声明对接口的 owner 级依赖；返回的句柄在注册表完成合并前可能处于缺席状态。
    pub fn import_interface<T>(&mut self) -> Result<BridgeImport<T>, RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        self.registry.import_interface::<T>(self.owner)
    }

    /// 登记模块撤销通知；宿主撤销该 owner 时调用，回调应自行清理外部持有状态。
    pub fn owner_revocation_listener(
        &mut self,
        callback: impl Fn(PluginModuleId) + Send + Sync + 'static,
    ) {
        self.registry
            .register_owner_revocation_listener(self.owner, callback);
    }
}

/// 收集系统阶段、时钟和排序约束，在 `register` 时交给 Runtime 注册表验证。
pub struct RuntimePluginRuntimeSceneSystemBuilder<'registry, F> {
    registry: &'registry mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
    id: String,
    stage: SystemStage,
    system_factory: F,
    sets: Vec<String>,
    constraints: Vec<SystemOrderingConstraint>,
    order: i32,
    tick_policy: SceneSystemTickPolicy,
}

impl<'registry, F> RuntimePluginRuntimeSceneSystemBuilder<'registry, F> {
    pub fn in_set(mut self, set: impl Into<String>) -> Self {
        self.sets.push(set.into());
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

    /// 先解析系统集合，再提交系统；非法阶段与时钟组合由注册表返回错误。
    pub fn register<S>(self) -> Result<(), RuntimeExtensionRegistryError>
    where
        S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
        F: Fn() -> S + Send + Sync + 'static,
    {
        let set_ids = self
            .sets
            .into_iter()
            .map(|set| self.registry.intern_system_set(set))
            .collect::<Result<Vec<_>, _>>()?;
        let system_factory = self.system_factory;

        let mut builder = self
            .registry
            .register_runtime_scene_system(self.owner, self.id, self.stage, system_factory)
            .with_order(self.order)
            .with_tick_policy(self.tick_policy);

        for set in set_ids {
            builder = builder.in_set(set);
        }
        for constraint in self.constraints {
            builder = match constraint {
                SystemOrderingConstraint::Before(reference) => builder.before(reference),
                SystemOrderingConstraint::After(reference) => builder.after(reference),
            };
        }
        builder.register()
    }
}

#[cfg(test)]
#[path = "tests/registration.rs"]
mod tests;
