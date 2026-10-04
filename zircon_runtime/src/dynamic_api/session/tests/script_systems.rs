use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::ecs::ScheduledSceneStep;
use crate::scene::{SystemStage, World};
use crate::script::{
    SCRIPT_SCENE_FIXED_UPDATE_SYSTEM, SCRIPT_SCENE_RUNTIME_SYSTEM_SET, SCRIPT_SCENE_UPDATE_SYSTEM,
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
    let merged = merge_builtin_script_scene_systems(&linked).unwrap();
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
    let merged = merge_builtin_script_scene_systems(&linked).unwrap();
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

#[test]
fn full_linked_script_runtime_override_preserves_both_registered_phases() {
    let mut linked = RuntimeExtensionRegistry::default();
    let owner = linked
        .intern_plugin_module("zr_vm_language.runtime")
        .unwrap();
    let script_set = linked
        .intern_system_set(SCRIPT_SCENE_RUNTIME_SYSTEM_SET)
        .unwrap();
    for (id, stage) in [
        (SCRIPT_SCENE_FIXED_UPDATE_SYSTEM, SystemStage::FixedUpdate),
        (SCRIPT_SCENE_UPDATE_SYSTEM, SystemStage::Update),
    ] {
        linked
            .register_runtime_scene_system(owner, id, stage, || |_| Ok(()))
            .in_set(script_set)
            .register()
            .unwrap();
    }

    let merged = merge_builtin_script_scene_systems(&linked).unwrap();
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
