use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::{Transform, Vec3};
use crate::scene::components::NodeKind;
use crate::scene::{EntityId, LevelMetadata, LevelSystem, World};

use super::{
    active_script_binding_projection, with_script_binding_number_and_world_mut,
    with_script_binding_property_matches, ScriptSceneLifecyclePhase, SCRIPT_BINDINGS_COMPONENT,
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
    let binding_key = &first.update_bindings[0].binding_key;
    assert!(!level.script_binding_started(entity, binding_key));
    level.mark_script_binding_started(entity, binding_key.clone());

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
    assert!(level.script_binding_started(entity, binding_key));

    level
        .with_world_mut(|world| {
            world.set_dynamic_component(entity, "runtime13.unrelated", serde_json::json!(true))
        })
        .expect("unrelated component update succeeds");

    let after_unrelated_update =
        active_script_binding_projection(&level).expect("unrelated projection");
    assert!(Rc::ptr_eq(&first, &after_unrelated_update));
    assert!(level.script_binding_started(entity, binding_key));

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
fn runtime13_projection_keeps_duplicate_activation_and_preserves_unchanged_binding_start_state() {
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
    for binding in bindings {
        assert!(!level.script_binding_started(entity, &binding.binding_key));
        level.mark_script_binding_started(entity, binding.binding_key.clone());
    }
    let disabled_key = initial.update_bindings[0].binding_key.clone();
    let unchanged_key = initial.update_bindings[1].binding_key.clone();

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
    assert_eq!(bindings[0].binding_key, unchanged_key);
    assert!(level.script_binding_started(entity, &unchanged_key));
    assert!(!level.script_binding_started(entity, &disabled_key));

    level
        .with_world_mut(|world| {
            world.set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([
                    { "package": "runtime13_projection", "module": "main", "update": true },
                    { "package": "runtime13_projection", "module": "main", "update": true }
                ]),
            )
        })
        .expect("re-enabled binding update succeeds");
    let reenabled = active_script_binding_projection(&level).expect("re-enabled projection");
    assert_eq!(reenabled.update_bindings.len(), 2);
    assert!(!level.script_binding_started(entity, &disabled_key));
    assert!(level.script_binding_started(entity, &unchanged_key));
}

fn level_with_bindings(handle: u64, bindings: serde_json::Value) -> (LevelSystem, EntityId) {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)
        .unwrap();
    (
        LevelSystem::new(
            WorldHandle::new(handle),
            Arc::new(Mutex::new(world)),
            LevelMetadata::default(),
        ),
        entity,
    )
}

#[test]
fn runtime13_property_only_hp_rewrite_preserves_start_state_and_updates_projection() {
    let (level, entity) = level_with_bindings(
        130_018,
        serde_json::json!([{
            "package": "runtime13_projection", "module": "main",
            "properties": { "hp": 73.0 }
        }]),
    );
    let before = active_script_binding_projection(&level).unwrap();
    let key = &before.update_bindings[0].binding_key;
    level.mark_script_binding_started(entity, key.clone());
    level.with_world_mut(|world| {
        let mut bindings = world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .unwrap()
            .clone();
        bindings[0]["properties"]["hp"] = serde_json::json!(72.0);
        world
            .set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT, bindings)
            .unwrap();
    });

    // Combat's query path can rebuild the projection in the middle of a phase.
    let hp = with_script_binding_number_and_world_mut(&level, entity, "hp", |hp, _| hp)
        .expect("updated numeric property");
    let after = active_script_binding_projection(&level).unwrap();
    assert!(!Rc::ptr_eq(&before, &after));
    assert_eq!(before.number_for_entity(entity, "hp"), Some(73.0));
    assert_eq!(hp, Some(72.0));
    assert_eq!(after.number_for_entity(entity, "hp"), Some(72.0));
    assert!(level.script_binding_started(entity, key));
}

#[test]
fn runtime13_module_change_restarts_only_the_changed_binding() {
    let (level, entity) = level_with_bindings(
        130_019,
        serde_json::json!([
            { "package": "runtime13_projection", "module": "main" },
            { "package": "runtime13_projection", "module": "other" }
        ]),
    );
    let before = active_script_binding_projection(&level).unwrap();
    for binding in &before.update_bindings {
        level.mark_script_binding_started(entity, binding.binding_key.clone());
    }
    let changed_key = &before.update_bindings[0].binding_key;
    let unchanged_key = &before.update_bindings[1].binding_key;
    level.with_world_mut(|world| {
        world
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([
                    { "package": "runtime13_projection", "module": "replacement" },
                    { "package": "runtime13_projection", "module": "other", "update": false, "fixed_update": false }
                ]),
            )
            .unwrap();
    });

    let after = active_script_binding_projection(&level).unwrap();
    assert_eq!(after.update_bindings.len(), 1);
    assert!(!level.script_binding_started(entity, changed_key));
    assert!(!level.script_binding_started(entity, &after.update_bindings[0].binding_key));
    assert!(
        level.script_binding_started(entity, unchanged_key),
        "an enabled binding keeps lifecycle even when both phase callbacks are disabled"
    );
}

#[test]
fn runtime13_removed_entity_prunes_binding_start_state() {
    let (level, entity) = level_with_bindings(
        130_020,
        serde_json::json!([{
            "package": "runtime13_projection", "module": "main",
            "properties": { "role": "player" }
        }]),
    );
    let before = active_script_binding_projection(&level).unwrap();
    let key = &before.update_bindings[0].binding_key;
    level.mark_script_binding_started(entity, key.clone());
    level.with_world_mut(|world| world.remove_entity(entity).unwrap());

    let matches =
        with_script_binding_property_matches(&level, "role", "player", |ids, _| ids.to_vec())
            .expect("removed entity property query");
    assert!(matches.is_empty());
    assert!(!level.script_binding_started(entity, key));
    assert!(active_script_binding_projection(&level)
        .unwrap()
        .update_bindings
        .is_empty());
}

#[test]
fn runtime13_world_replacement_with_same_entity_and_key_starts_unstarted() {
    let (level, entity) = level_with_bindings(
        130_021,
        serde_json::json!([{ "package": "runtime13_projection", "module": "main" }]),
    );
    let before = active_script_binding_projection(&level).unwrap();
    let key = &before.update_bindings[0].binding_key;
    level.mark_script_binding_started(entity, key.clone());
    let replacement = level
        .with_world(|world| serde_json::from_value(serde_json::to_value(world).unwrap()).unwrap());

    level.replace(replacement);

    assert!(!level.script_binding_started(entity, key));
    let after = active_script_binding_projection(&level).unwrap();
    assert!(!Rc::ptr_eq(&before, &after));
    assert_eq!(after.update_bindings[0].entity, entity);
    assert_eq!(&after.update_bindings[0].binding_key, key);
    assert!(!level.script_binding_started(entity, key));
}

#[test]
fn runtime13_invalid_binding_element_is_isolated() {
    use crate::scene::ScriptBindingFaultKind;
    let (level, entity_a) = level_with_bindings(
        130_022,
        serde_json::json!([
            { "package": 17, "module": "bad" },
            { "package": "runtime13_projection", "module": "valid_a", "properties": { "role": "player", "hp": 7 } }
        ]),
    );
    let entity_b = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{ "package": "runtime13_projection", "module": "valid_b" }]),
            )
            .unwrap();
        entity
    });
    let before = level.with_world(|world| {
        (
            world.world_generation(),
            world.dynamic_component_generation(SCRIPT_BINDINGS_COMPONENT),
        )
    });
    let projection = active_script_binding_projection(&level).unwrap();
    assert_eq!(projection.update_bindings.len(), 2);
    assert_eq!(projection.update_bindings[0].entity, entity_a);
    assert_eq!(
        projection.update_bindings[0].binding_key,
        "runtime13_projection::valid_a#1"
    );
    assert_eq!(projection.update_bindings[1].entity, entity_b);
    assert_eq!(projection.number_for_entity(entity_a, "hp"), Some(7.0));
    assert_eq!(projection.property_matches("role", "player"), &[entity_a]);
    assert_eq!(level.script_binding_fault_total(), 1);
    assert_eq!(
        level.script_binding_faults()[0].kind,
        ScriptBindingFaultKind::InvalidBinding
    );
    assert_eq!(level.script_binding_faults()[0].entity, entity_a);
    // Evict the thread-local cache: deduplication must still be Level-owned.
    let (other, _) = level_with_bindings(130_023, serde_json::json!([]));
    active_script_binding_projection(&other).unwrap();
    active_script_binding_projection(&level).unwrap();
    assert_eq!(level.script_binding_fault_total(), 1);
    assert_eq!(
        level.with_world(|world| (
            world.world_generation(),
            world.dynamic_component_generation(SCRIPT_BINDINGS_COMPONENT)
        )),
        before
    );
    level.with_world_mut(|world| {
        world.set_dynamic_component(entity_b, SCRIPT_BINDINGS_COMPONENT,
            serde_json::json!([{ "package": "runtime13_projection", "module": "valid_b", "properties": { "hp": 8 } }])).unwrap();
    });
    active_script_binding_projection(&level).unwrap();
    assert_eq!(
        level.script_binding_fault_total(),
        2,
        "one invalid-element fault per generation"
    );
}

#[test]
fn runtime13_invalid_binding_row_is_isolated_per_entity_and_generation() {
    let (level, entity_a) =
        level_with_bindings(130_024, serde_json::json!({ "package": "not_an_array" }));
    level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{ "package": "runtime13_projection", "module": "valid" }]),
            )
            .unwrap();
    });
    assert_eq!(
        active_script_binding_projection(&level)
            .unwrap()
            .update_bindings
            .len(),
        1
    );
    assert_eq!(level.script_binding_fault_total(), 1);
    let fault = level.script_binding_faults().remove(0);
    assert_eq!(fault.entity, entity_a);
    assert_eq!(fault.binding_key, None);
    active_script_binding_projection(&level).unwrap();
    assert_eq!(level.script_binding_fault_total(), 1);
}

#[test]
fn script_binding_fault_disabling_and_reenabling_is_fresh_but_property_edits_keep_faulted() {
    use crate::scene::{ScriptBindingFault, ScriptBindingFaultKind};
    let (level, entity) = level_with_bindings(
        130_025,
        serde_json::json!([{ "package": "probe", "module": "main" }]),
    );
    let projection = active_script_binding_projection(&level).unwrap();
    let key = &projection.update_bindings[0].binding_key;
    level.record_script_binding_fault(ScriptBindingFault {
        entity,
        binding_key: Some(key.clone()),
        package: "probe".into(),
        module: "main".into(),
        export: "onStart".into(),
        kind: ScriptBindingFaultKind::Guest,
        message: "guest failed".into(),
        host_fault_count: 0,
        outer_frame_index: 1,
    });
    level.with_world_mut(|world| world.set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT,
        serde_json::json!([{ "package": "probe", "module": "main", "properties": { "hp": 3 } }])).unwrap());
    active_script_binding_projection(&level).unwrap();
    assert!(level.script_binding_faulted(entity, key));
    level.with_world_mut(|world| {
        world
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{ "package": "probe", "module": "main", "enabled": false }]),
            )
            .unwrap()
    });
    assert!(active_script_binding_projection(&level)
        .unwrap()
        .update_bindings
        .is_empty());
    assert!(!level.script_binding_faulted(entity, key));
    level.with_world_mut(|world| {
        world
            .set_dynamic_component(
                entity,
                SCRIPT_BINDINGS_COMPONENT,
                serde_json::json!([{ "package": "probe", "module": "main" }]),
            )
            .unwrap()
    });
    assert_eq!(
        active_script_binding_projection(&level)
            .unwrap()
            .update_bindings
            .len(),
        1
    );
    assert!(!level.script_binding_faulted(entity, key));
    assert_eq!(level.script_binding_fault_total(), 1);
}

#[test]
fn script_binding_fault_host_cause_wins_over_guest_error_and_keeps_secondary_text() {
    use crate::scene::ScriptBindingFaultKind;
    use crate::script::vm::runtime_context::{ScriptHostFault, ScriptHostFaultSummary};
    use crate::script::{VmError, VmHostInterfaceError};
    let (level, _) = level_with_bindings(
        130_026,
        serde_json::json!([{ "package": "probe", "module": "main" }]),
    );
    let projection = active_script_binding_projection(&level).unwrap();
    let binding = &projection.update_bindings[0];
    let summary = ScriptHostFaultSummary {
        first: Some(ScriptHostFault {
            module: "host".into(),
            function: "fail".into(),
            message: "界".repeat(341),
        }),
        count: 2,
    };
    let failure = super::fault::callback_outcome(
        &level,
        binding,
        "onUpdate",
        Err(VmHostInterfaceError::CallbackFailed(VmError::Operation(
            "secondary guest error".into(),
        ))),
        summary,
    )
    .unwrap_err();
    assert_eq!(
        failure.kind,
        ScriptBindingFaultKind::HostCall {
            module: "host".into(),
            function: "fail".into()
        }
    );
    assert_eq!(failure.host_fault_count, 2);
    assert!(failure.message.contains("secondary guest error"));
    assert!(failure.message.len() <= 1024);
    let guest = super::fault::callback_outcome(
        &level,
        binding,
        "onUpdate",
        Err(VmHostInterfaceError::CallbackFailed(VmError::Operation(
            "guest only".into(),
        ))),
        ScriptHostFaultSummary::default(),
    )
    .unwrap_err();
    assert_eq!(guest.kind, ScriptBindingFaultKind::Guest);
    assert!(guest.message.contains("guest only"));
}

#[test]
fn script_binding_fault_malformed_known_identity_does_not_revive_on_property_repair() {
    let (level, entity) = level_with_bindings(
        130_027,
        serde_json::json!([{ "package": "probe", "module": "main", "properties": 17 }]),
    );
    let key = "probe::main#0";
    assert!(active_script_binding_projection(&level)
        .unwrap()
        .update_bindings
        .is_empty());
    assert!(level.script_binding_faulted(entity, key));
    assert_eq!(level.script_binding_fault_total(), 1);
    level.with_world_mut(|world| world.set_dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT,
        serde_json::json!([{ "package": "probe", "module": "main", "properties": { "hp": 3 } }])).unwrap());
    assert_eq!(
        active_script_binding_projection(&level)
            .unwrap()
            .update_bindings
            .len(),
        1
    );
    assert!(
        level.script_binding_faulted(entity, key),
        "property repair cannot revive a known faulted identity"
    );
    assert_eq!(level.script_binding_fault_total(), 1);
}
