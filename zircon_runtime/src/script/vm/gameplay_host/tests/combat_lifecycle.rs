use super::*;

#[test]
fn gameplay_host_damage_report_preserves_death_position() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(
            entity,
            Transform::from_translation(Vec3::new(4.0, 0.0, -3.0)),
        )
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            SCRIPT_BINDINGS_COMPONENT,
            serde_json::json!([{
                "package": "vampire_game",
                "module": "main",
                "enabled": true,
                "properties": { "role": "enemy", "hp": 5.0 }
            }]),
        )
        .unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(44),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    let report = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || {
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "damage_entity_report",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(6.0),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );
    let ScriptHostValue::String(report) = report else {
        panic!("damage_entity_report should return JSON text");
    };
    let report: serde_json::Value = serde_json::from_str(&report).unwrap();

    assert_eq!(
        report.get("hit").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        report.get("killed").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("position")
            .and_then(serde_json::Value::as_array)
            .and_then(|position| position.first())
            .and_then(serde_json::Value::as_f64),
        Some(4.0)
    );
    assert!(
        level.with_world(|world| world.find_node(entity).is_none()),
        "killed entity should be removed from the world"
    );
}

#[test]
fn script_held_entity_handle_reports_invalid_after_despawn() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(
            entity,
            Transform::from_translation(Vec3::new(2.0, 0.0, -1.0)),
        )
        .unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(50),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    let before = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || {
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "position_json",
                    vec![ScriptHostValue::Int(entity as i64)],
                    &capabilities,
                )
                .unwrap()
        },
    );
    let ScriptHostValue::String(before) = before else {
        panic!("position_json should return JSON text before despawn");
    };
    assert!(
        serde_json::from_str::<serde_json::Value>(&before)
            .unwrap()
            .is_array(),
        "live script-held entity id should resolve to a position array"
    );

    let removed = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || {
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "despawn",
                    vec![ScriptHostValue::Int(entity as i64)],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(removed, ScriptHostValue::Bool(true));
    assert!(!level.with_world(|world| world.contains_entity(entity)));

    let after = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || {
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "position_json",
                    vec![ScriptHostValue::Int(entity as i64)],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(after, ScriptHostValue::String("null".to_string()));

    let stale_write = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || {
            exports.call_with_capabilities(
                GAMEPLAY_MODULE,
                "set_position",
                vec![
                    ScriptHostValue::Int(entity as i64),
                    ScriptHostValue::Float(5.0),
                    ScriptHostValue::Float(0.0),
                    ScriptHostValue::Float(0.0),
                ],
                &capabilities,
            )
        },
    );
    let stale_error = stale_write.unwrap_err().to_string();
    assert!(
        stale_error.contains("cannot update transform for missing entity"),
        "stale script-held entity id should be rejected by the typed SceneError boundary"
    );
    assert!(
        !stale_error.contains("missing node"),
        "stale script-held entity diagnostics must not drift back to the retired node wording"
    );
}

#[test]
fn gameplay_host_damage_entity_reports_hit_before_death() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            entity,
            SCRIPT_BINDINGS_COMPONENT,
            serde_json::json!([{
                "package": "vampire_game",
                "module": "main",
                "enabled": true,
                "properties": { "role": "enemy", "hp": 24.0 }
            }]),
        )
        .unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(48),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    let hit = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || {
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "damage_entity",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(6.0),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );

    assert_eq!(hit, ScriptHostValue::Bool(true));
    let hp = level.with_world(|world| {
        world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .and_then(serde_json::Value::as_array)
            .and_then(|bindings| bindings.first())
            .and_then(|binding| binding.get("properties"))
            .and_then(|properties| properties.get("hp"))
            .and_then(serde_json::Value::as_f64)
    });
    assert_eq!(hp, Some(18.0));
}
