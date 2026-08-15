use crate::plugin::{RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use crate::scene::WorldRuntimeExtensionPlan;
use crate::script::{SCRIPT_SCENE_RUNTIME_SYSTEM_SET, ScriptSceneRuntimeSystem};

use super::{RuntimeDynamicSessionError, RuntimeDynamicSessionResult};

pub(super) fn merge_builtin_script_scene_systems(
    linked_registry: &RuntimeExtensionRegistry,
    _linked_plan: WorldRuntimeExtensionPlan,
) -> RuntimeDynamicSessionResult<WorldRuntimeExtensionPlan> {
    let mut merged = linked_registry.clone();
    let owner = merged
        .intern_plugin_module("zr_vm_language.runtime")
        .map_err(register_builtin_error)?;

    for system in [
        ScriptSceneRuntimeSystem::fixed_update(),
        ScriptSceneRuntimeSystem::update(),
    ] {
        let id = system.id();
        let linked_owns_system = merged
            .plugin_runtime_systems()
            .any(|(_, registration)| registration.id == id);
        if !linked_owns_system {
            let stage = system.stage();
            let system_set = merged
                .intern_system_set(SCRIPT_SCENE_RUNTIME_SYSTEM_SET)
                .map_err(register_builtin_error)?;
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
mod tests {
    use crate::plugin::RuntimeExtensionRegistry;
    use crate::scene::ecs::ScheduledSceneStep;
    use crate::scene::{SystemStage, World};
    use crate::script::{
        SCRIPT_SCENE_FIXED_UPDATE_SYSTEM, SCRIPT_SCENE_RUNTIME_SYSTEM_SET,
        SCRIPT_SCENE_UPDATE_SYSTEM,
    };

    use super::merge_builtin_script_scene_systems;

    #[test]
    fn linked_script_runtime_system_wins_while_builtin_fills_the_missing_phase() {
        let mut linked = RuntimeExtensionRegistry::default();
        let owner = linked
            .intern_plugin_module("zr_vm_language.runtime")
            .unwrap();
        linked
            .register_runtime_scene_system(
                owner,
                SCRIPT_SCENE_FIXED_UPDATE_SYSTEM,
                SystemStage::FixedUpdate,
                || |_| Ok(()),
            )
            .register()
            .unwrap();
        let linked_plan = linked.world_runtime_extension_plan().unwrap();

        let merged = merge_builtin_script_scene_systems(&linked, linked_plan).unwrap();
        let mut world = World::empty();
        merged.apply_to_world(&mut world).unwrap();

        assert_eq!(merged.registration_count(), 2);
        assert_eq!(
            runtime_system_ids(&world, SystemStage::FixedUpdate),
            vec![SCRIPT_SCENE_FIXED_UPDATE_SYSTEM]
        );
        assert_eq!(
            runtime_system_ids(&world, SystemStage::Update),
            vec![SCRIPT_SCENE_UPDATE_SYSTEM]
        );
    }

    #[test]
    fn builtin_missing_phase_reuses_linked_system_set_identity() {
        let mut linked = RuntimeExtensionRegistry::default();
        let owner = linked
            .intern_plugin_module("zr_vm_language.runtime")
            .unwrap();
        let unrelated_set = linked.intern_system_set("unrelated.first").unwrap();
        let script_set = linked
            .intern_system_set(SCRIPT_SCENE_RUNTIME_SYSTEM_SET)
            .unwrap();
        assert_ne!(unrelated_set, script_set);
        linked
            .register_runtime_scene_system(
                owner,
                SCRIPT_SCENE_FIXED_UPDATE_SYSTEM,
                SystemStage::FixedUpdate,
                || |_| Ok(()),
            )
            .in_set(script_set)
            .register()
            .unwrap();
        let linked_plan = linked.world_runtime_extension_plan().unwrap();

        let merged = merge_builtin_script_scene_systems(&linked, linked_plan).unwrap();
        let mut world = World::empty();
        merged.apply_to_world(&mut world).unwrap();
        let runtime_systems = world.schedule().system_registry().runtime_systems();

        for system_id in [SCRIPT_SCENE_FIXED_UPDATE_SYSTEM, SCRIPT_SCENE_UPDATE_SYSTEM] {
            let system = runtime_systems
                .iter()
                .find(|system| system.id() == system_id)
                .expect("script runtime system should be registered");
            assert_eq!(system.sets(), &[script_set]);
            assert!(!system.sets().contains(&unrelated_set));
        }
    }

    fn runtime_system_ids(world: &World, stage: SystemStage) -> Vec<String> {
        world
            .scheduled_native_system_steps_for_stage(stage)
            .iter()
            .filter_map(|step| match step {
                ScheduledSceneStep::Runtime { id, .. } => Some(id.clone()),
                _ => None,
            })
            .collect()
    }
}
