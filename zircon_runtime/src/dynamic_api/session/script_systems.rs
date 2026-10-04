use crate::plugin::{RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use crate::scene::WorldRuntimeExtensionPlan;
use crate::script::{
    ScriptSceneRuntimeSystem, SCRIPT_SCENE_FIXED_UPDATE_SYSTEM, SCRIPT_SCENE_RUNTIME_SYSTEM_SET,
    SCRIPT_SCENE_UPDATE_SYSTEM,
};

use super::{RuntimeDynamicSessionError, RuntimeDynamicSessionResult};

pub(super) fn merge_builtin_script_scene_systems(
    linked_registry: &RuntimeExtensionRegistry,
) -> RuntimeDynamicSessionResult<WorldRuntimeExtensionPlan> {
    let mut linked_owns_fixed_update = false;
    let mut linked_owns_update = false;
    for (_, registration) in linked_registry.plugin_runtime_systems() {
        match registration.id.as_str() {
            SCRIPT_SCENE_FIXED_UPDATE_SYSTEM => linked_owns_fixed_update = true,
            SCRIPT_SCENE_UPDATE_SYSTEM => linked_owns_update = true,
            _ => {}
        }
        // 两个阶段都由 linked 插件提供时保留原计划，避免重建或重复注册其身份。
        if linked_owns_fixed_update && linked_owns_update {
            return linked_registry
                .world_runtime_extension_plan()
                .map_err(register_builtin_error);
        }
    }

    let mut merged = linked_registry.clone();
    let owner = merged
        .intern_plugin_module("zr_vm_language.runtime")
        .map_err(register_builtin_error)?;
    let system_set = merged
        .intern_system_set(SCRIPT_SCENE_RUNTIME_SYSTEM_SET)
        .map_err(register_builtin_error)?;

    for (system, linked_owns_system) in [
        (
            ScriptSceneRuntimeSystem::fixed_update(),
            linked_owns_fixed_update,
        ),
        (ScriptSceneRuntimeSystem::update(), linked_owns_update),
    ] {
        if !linked_owns_system {
            let id = system.id();
            let stage = system.stage();
            merged
                .register_runtime_scene_system(owner, id, stage, move || {
                    let system = system.clone();
                    move |context| system.run(context)
                })
                .in_set(system_set)
                .with_order(10)
                .register()
                .map_err(register_builtin_error)?;
        }
    }

    merged
        .world_runtime_extension_plan()
        .map_err(register_builtin_error)
}

fn register_builtin_error(source: RuntimeExtensionRegistryError) -> RuntimeDynamicSessionError {
    RuntimeDynamicSessionError::RuntimeExtensionRegistryStep {
        step: "register builtin script runtime scene systems",
        source,
    }
}

#[cfg(test)]
#[path = "tests/script_systems.rs"]
mod tests;
