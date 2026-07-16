use crate::core::CoreRuntime;
use crate::plugin::RuntimeExtensionRegistry;

use super::{RuntimeDynamicSessionError, RuntimeDynamicSessionResult};

pub(super) fn install_builtin_scene_runtime_hooks(
    runtime: &CoreRuntime,
) -> RuntimeDynamicSessionResult<()> {
    let mut extensions = RuntimeExtensionRegistry::default();
    register_missing_scene_hook(
        runtime,
        &mut extensions,
        crate::animation::scene_hook_registration(),
    )?;
    register_missing_scene_hook(
        runtime,
        &mut extensions,
        crate::script::script_scene_fixed_update_hook_registration(),
    )?;
    register_missing_scene_hook(
        runtime,
        &mut extensions,
        crate::script::script_scene_update_hook_registration(),
    )?;
    crate::scene::install_scene_runtime_hooks(
        &runtime.handle(),
        extensions.scene_hooks().iter().cloned(),
    )
    .map_err(|source| RuntimeDynamicSessionError::CoreStep {
        step: "install scene runtime hooks",
        source,
    })
}

fn register_missing_scene_hook(
    runtime: &CoreRuntime,
    extensions: &mut RuntimeExtensionRegistry,
    registration: crate::scene::SceneRuntimeHookRegistration,
) -> RuntimeDynamicSessionResult<()> {
    let descriptor = registration.descriptor();
    let hook = descriptor.id.clone();
    let already_installed =
        crate::scene::scene_runtime_hooks_for_stage(&runtime.handle(), descriptor.stage)
            .map_err(|source| RuntimeDynamicSessionError::CoreStep {
                step: "query installed scene runtime hooks",
                source,
            })?
            .iter()
            .any(|hook| hook.descriptor().id == descriptor.id);
    if already_installed {
        return Ok(());
    }
    extensions
        .register_scene_hook(registration)
        .map_err(|source| RuntimeDynamicSessionError::RegisterSceneRuntimeHook { hook, source })
}
