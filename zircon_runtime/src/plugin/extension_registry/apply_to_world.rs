//! 把插件贡献转为可复用的 World 安装计划；每个 World 独立构造资源和系统实例。
mod component;

#[cfg(test)]
#[path = "apply_to_world/tests/capacity_tests.rs"]
mod capacity_tests;

use crate::plugin::{RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use crate::scene::{
    World, WorldRuntimeExtensionError, WorldRuntimeExtensionPlan, WorldRuntimeExtensionRegistration,
};

impl RuntimeExtensionRegistry {
    /// 在现有 World 上应用本轮贡献并冻结注册表；失败可能保留已安装的前序项，调用方需决定重建或清理目标 World。
    pub fn apply_to_world(
        &mut self,
        world: &mut World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.finalize();
        self.world_runtime_extension_plan()?
            .apply_to_world(world)
            .map_err(|error| RuntimeExtensionRegistryError::WorldRegistration(error.to_string()))
    }

    /// 为场景驱动或动态会话取得可复用计划；其中的工厂在应用到每个 World 时重新运行。
    pub fn world_runtime_extension_plan(
        &self,
    ) -> Result<WorldRuntimeExtensionPlan, RuntimeExtensionRegistryError> {
        let mut registrations =
            Vec::with_capacity(world_runtime_extension_registration_capacity(self));
        for component in self.components().iter().cloned() {
            let key = format!("component:{}", component.type_id);
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                world
                    .register_component_type(component.clone())
                    .map_err(|error| {
                        WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                    })
            }));
        }
        for (_, resource) in self.plugin_resources() {
            let resource = resource.clone();
            let key = format!("resource:{}", resource.type_name());
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                resource.apply(world).map_err(|error| {
                    WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                })
            }));
        }
        for (_, event) in self.plugin_events() {
            let event = event.clone();
            let key = format!("event:{}", event.type_name());
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                event.apply(world).map_err(|error| {
                    WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                })
            }));
        }
        for (_, system) in self.plugin_systems() {
            let registration = system.clone();
            let key = format!("system:{}", registration.id);
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                let system = registration.build(world).map_err(|error| {
                    WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                })?;
                world.register_boxed_native_system(system).map_err(|error| {
                    WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                })
            }));
        }
        for (_, system) in self.plugin_runtime_systems() {
            let registration = system.clone();
            let key = format!("system:{}", registration.id);
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                world
                    .register_boxed_runtime_scene_system(registration.build())
                    .map_err(|error| {
                        WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                    })
            }));
        }
        WorldRuntimeExtensionPlan::from_registrations(registrations)
            .map_err(|error| RuntimeExtensionRegistryError::WorldRegistration(error.to_string()))
    }
}

// 计划预留数涵盖组件、资源、事件及两类系统，保持与上方注册顺序相同的家族集合。
fn world_runtime_extension_registration_capacity(registry: &RuntimeExtensionRegistry) -> usize {
    registry
        .components()
        .len()
        .saturating_add(registry.plugin_resources().count())
        .saturating_add(registry.plugin_events().count())
        .saturating_add(registry.plugin_systems().count())
        .saturating_add(registry.plugin_runtime_systems().count())
}
