use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::{Transform, Vec3};
use crate::scene::components::NodeKind;
use crate::scene::{LevelMetadata, LevelSystem, World};

use super::{
    active_script_binding_projection, ScriptSceneLifecyclePhase, SCRIPT_BINDINGS_COMPONENT,
};

#[test]
fn runtime13_owned_context_construction_stays_measured_until_scope_hard_cut() {
    let source = include_str!("../scene_system.rs");
    let call_export = source
        .split("fn call_export_for_binding")
        .nth(1)
        .and_then(|source| source.split("fn trace_script_binding_export").next())
        .expect("scene-system export dispatcher");

    for required in [
        "ScriptHostHotPathMetrics::record_script_context_weak_handle();",
        "core.downgrade()",
        "ScriptHostHotPathMetrics::record_script_context_level_clone();",
        "level.clone()",
    ] {
        assert_eq!(
            call_export.matches(required).count(),
            1,
            "owned context construction must have exactly one measured occurrence before the scope hard cut: {required}"
        );
    }
}

#[test]
fn runtime13_script_binding_projection_ignores_unrelated_world_mutations() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            SCRIPT_BINDINGS_COMPONENT,
            serde_json::json!([{
                "package": "runtime13_projection",
                "module": "main",
                "enabled": true,
                "update": true,
                "fixed_update": false,
                "properties": { "role": "player", "hp": 73.0 }
            }]),
        )
        .expect("script bindings component is accepted");
    let level = LevelSystem::new(
        WorldHandle::new(130_013),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );

    let first = active_script_binding_projection(&level).expect("first projection");
    let second = active_script_binding_projection(&level).expect("stable projection");
    assert!(Rc::ptr_eq(&first, &second));
    assert_eq!(first.update_bindings.len(), 1);
    assert!(first.fixed_update_bindings.is_empty());
    assert_eq!(first.property_matches("role", "player"), &[entity]);
    assert_eq!(first.number_for_entity(entity, "hp"), Some(73.0));
    assert!(!first.update_bindings[0].started.replace(true));

    level
        .with_world_mut(|world| {
            world.update_transform(
                entity,
                Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            )
        })
        .expect("transform update succeeds");

    let after_transform_update =
        active_script_binding_projection(&level).expect("transform projection");
    assert!(Rc::ptr_eq(&first, &after_transform_update));
    assert!(after_transform_update.update_bindings[0].started.get());

    level
        .with_world_mut(|world| {
            world.set_dynamic_component(entity, "runtime13.unrelated", serde_json::json!(true))
        })
        .expect("unrelated component update succeeds");

    let after_unrelated_update =
        active_script_binding_projection(&level).expect("unrelated projection");
    assert!(Rc::ptr_eq(&first, &after_unrelated_update));
    assert!(after_unrelated_update.update_bindings[0].started.get());

    level
        .with_world_mut(|world| {
            world.set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{
                    "package": "runtime13_projection",
                    "module": "replacement",
                    "enabled": true,
                    "update": true,
                    "fixed_update": false,
                    "properties": { "role": "enemy", "hp": 19.0 }
                }]),
            )
        })
        .expect("generation-changing update succeeds");

    let updated = active_script_binding_projection(&level).expect("updated projection");
    assert!(!Rc::ptr_eq(&first, &updated));
    assert_eq!(updated.update_bindings[0].module, "replacement");
    assert_eq!(updated.property_matches("role", "enemy"), &[entity]);
    assert_eq!(updated.number_for_entity(entity, "hp"), Some(19.0));

    level.with_world_mut(|world| world.remove_entity(entity).unwrap());
    let after_removal = active_script_binding_projection(&level).expect("removed projection");
    assert!(!Rc::ptr_eq(&updated, &after_removal));
    assert!(after_removal.update_bindings.is_empty());
}

#[test]
fn runtime13_projection_rebuilds_after_deserialized_world_replacement() {
    let level = LevelSystem::new(
        WorldHandle::new(130_015),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );
    let empty_projection =
        active_script_binding_projection(&level).expect("empty world projection");
    assert!(empty_projection.update_bindings.is_empty());

    let mut persisted = World::empty();
    let entity = persisted
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    persisted
        .set_dynamic_component(
            entity,
            SCRIPT_BINDINGS_COMPONENT,
            serde_json::json!([{
                "package": "runtime13_projection",
                "module": "restored",
                "enabled": true,
                "update": true
            }]),
        )
        .expect("script bindings component is accepted");
    let replacement: World = serde_json::from_value(
        serde_json::to_value(&persisted).expect("world serialization succeeds"),
    )
    .expect("world deserialization succeeds");
    assert_eq!(
        replacement.dynamic_component_generation(SCRIPT_BINDINGS_COMPONENT),
        0,
        "deserialization resets runtime-only component revisions"
    );

    level.replace(replacement);

    let restored_projection =
        active_script_binding_projection(&level).expect("restored world projection");
    assert!(!Rc::ptr_eq(&empty_projection, &restored_projection));
    assert_eq!(restored_projection.update_bindings.len(), 1);
    assert_eq!(restored_projection.update_bindings[0].entity, entity);
    assert_eq!(restored_projection.update_bindings[0].module, "restored");
}

#[test]
fn runtime13_projection_invalidates_when_world_replacement_removes_bindings() {
    let mut persisted = World::empty();
    let entity = persisted
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    persisted
        .set_dynamic_component(
            entity,
            SCRIPT_BINDINGS_COMPONENT,
            serde_json::json!([{
                "package": "runtime13_projection",
                "module": "retired",
                "enabled": true,
                "update": true
            }]),
        )
        .expect("script bindings component is accepted");
    let source: World = serde_json::from_value(
        serde_json::to_value(&persisted).expect("world serialization succeeds"),
    )
    .expect("world deserialization succeeds");
    assert_eq!(
        source.dynamic_component_generation(SCRIPT_BINDINGS_COMPONENT),
        0,
        "deserialization starts without a runtime component revision"
    );

    let level = LevelSystem::new(
        WorldHandle::new(130_016),
        Arc::new(Mutex::new(source)),
        LevelMetadata::default(),
    );
    let before = active_script_binding_projection(&level).expect("binding projection");
    assert_eq!(before.update_bindings.len(), 1);

    level.replace(World::empty());

    let after = active_script_binding_projection(&level).expect("empty replacement projection");
    assert!(!Rc::ptr_eq(&before, &after));
    assert!(after.update_bindings.is_empty());
}

#[test]
fn runtime13_projection_invalidates_when_staged_world_removes_bindings() {
    let mut persisted = World::empty();
    let entity = persisted
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    persisted
        .set_dynamic_component(
            entity,
            SCRIPT_BINDINGS_COMPONENT,
            serde_json::json!([{
                "package": "runtime13_projection",
                "module": "retired_transactionally",
                "enabled": true,
                "update": true
            }]),
        )
        .expect("script bindings component is accepted");
    let source: World = serde_json::from_value(
        serde_json::to_value(&persisted).expect("world serialization succeeds"),
    )
    .expect("world deserialization succeeds");
    let level = LevelSystem::new(
        WorldHandle::new(130_017),
        Arc::new(Mutex::new(source)),
        LevelMetadata::default(),
    );
    let before = active_script_binding_projection(&level).expect("binding projection");
    let expected_generation = level.with_world(World::world_generation);

    level
        .replace_world_if_generation(expected_generation, World::empty())
        .expect("current generation commits staged world");

    let after = active_script_binding_projection(&level).expect("empty staged projection");
    assert!(!Rc::ptr_eq(&before, &after));
    assert!(after.update_bindings.is_empty());
}

#[test]
fn runtime13_projection_keeps_duplicate_activation_and_reload_start_state_independent() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            SCRIPT_BINDINGS_COMPONENT,
            serde_json::json!([
                { "package": "runtime13_projection", "module": "main", "update": true },
                { "package": "runtime13_projection", "module": "main", "update": true }
            ]),
        )
        .expect("duplicate bindings are accepted");
    let level = LevelSystem::new(
        WorldHandle::new(130_014),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );

    let initial = active_script_binding_projection(&level).expect("initial projection");
    let bindings = initial.bindings_for_phase(ScriptSceneLifecyclePhase::Update);
    assert_eq!(bindings.len(), 2);
    assert_ne!(bindings[0].binding_key, bindings[1].binding_key);
    assert!(!bindings[0].started.replace(true));
    assert!(!bindings[1].started.replace(true));

    level
        .with_world_mut(|world| {
            world.set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([
                    { "package": "runtime13_projection", "module": "main", "enabled": false, "update": true },
                    { "package": "runtime13_projection", "module": "main", "update": true }
                ]),
            )
        })
        .expect("disabled binding update succeeds");

    let reloaded = active_script_binding_projection(&level).expect("reloaded projection");
    let bindings = reloaded.bindings_for_phase(ScriptSceneLifecyclePhase::Update);
    assert_eq!(bindings.len(), 1);
    assert!(!bindings[0].started.get());
}
