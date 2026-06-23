use super::*;

#[test]
fn gameplay_host_script_property_match_and_heal_update_bindings() {
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
                "properties": { "role": "player", "hp": 30.0 }
            }]),
        )
        .unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(45),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    let matches = with_script_runtime_call_context(
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
                    "script_property_matches",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String("role".to_string()),
                        ScriptHostValue::String("player".to_string()),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(matches, ScriptHostValue::Bool(true));
    let hp_value = with_script_runtime_call_context(
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
                    "script_number",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String("hp".to_string()),
                        ScriptHostValue::Float(120.0),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(hp_value, ScriptHostValue::Float(30.0));
    let hp_at_most = with_script_runtime_call_context(
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
                    "script_number_at_most",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String("hp".to_string()),
                        ScriptHostValue::Float(30.0),
                        ScriptHostValue::Float(120.0),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(hp_at_most, ScriptHostValue::Bool(true));
    let entity_exists_value = with_script_runtime_call_context(
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
                    "entity_exists",
                    vec![ScriptHostValue::Int(entity as i64)],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(entity_exists_value, ScriptHostValue::Bool(true));
    let missing_entity_exists = with_script_runtime_call_context(
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
                    "entity_exists",
                    vec![ScriptHostValue::Int(0)],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(missing_entity_exists, ScriptHostValue::Bool(false));

    with_script_runtime_call_context(
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
                    "heal_entity",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(200.0),
                        ScriptHostValue::Float(120.0),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );

    let hp = level.with_world(|world| {
        world
            .dynamic_component(entity, SCRIPT_BINDINGS_COMPONENT)
            .and_then(serde_json::Value::as_array)
            .and_then(|bindings| bindings.first())
            .and_then(|binding| binding.get("properties"))
            .and_then(|properties| properties.get("hp"))
            .and_then(serde_json::Value::as_f64)
    });
    assert_eq!(hp, Some(120.0));
}

#[test]
fn gameplay_host_sets_animation_bool_and_world_hud_bar_for_scripted_gameplay() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(
            entity,
            Transform::from_translation(Vec3::new(2.0, 0.25, -4.0)),
        )
        .unwrap();
    world
        .set_animation_state_machine_player(
            entity,
            Some(
                crate::scene::components::AnimationStateMachinePlayerComponent {
                    state_machine: ResourceHandle::new(ResourceId::from_stable_label(
                        "res://animation/player.animation_state_machine.toml",
                    )),
                    parameters: Default::default(),
                    active_state: Some("idle".to_string()),
                    playing: true,
                },
            ),
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
                "properties": { "role": "player", "hp": 72.0 }
            }]),
        )
        .unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(47),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    with_script_runtime_call_context(
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
                    "set_animation_bool",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String("moving".to_string()),
                        ScriptHostValue::Bool(true),
                    ],
                    &capabilities,
                )
                .unwrap();
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "set_world_hud_bar",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(120.0),
                        ScriptHostValue::Float(1.45),
                        ScriptHostValue::Float(0.12),
                        ScriptHostValue::Float(1.92),
                        ScriptHostValue::Float(1.25),
                    ],
                    &capabilities,
                )
                .unwrap();
        },
    );

    let (moving, hud) = level.with_world(|world| {
        let moving = world
            .animation_state_machine_player(entity)
            .and_then(|player| player.parameters.get("moving"))
            .cloned();
        let hud = world
            .dynamic_component(entity, "render.world_hud_bars")
            .cloned();
        (moving, hud)
    });
    assert_eq!(moving, Some(AnimationParameterValue::Bool(true)));
    let bar = hud
        .as_ref()
        .and_then(|value| value.get("bars"))
        .and_then(serde_json::Value::as_array)
        .and_then(|bars| bars.first())
        .expect("set_world_hud_bar should create a bar");
    let y = bar
        .get("position")
        .and_then(serde_json::Value::as_array)
        .and_then(|position| position.get(1))
        .and_then(serde_json::Value::as_f64)
        .expect("world HUD bar should carry a y position");
    assert!(
        (y - 2.17).abs() <= 0.0001,
        "world HUD bar y position should include the configured offset, got {y}"
    );
    assert_eq!(
        bar.get("ratio").and_then(serde_json::Value::as_f64),
        Some(0.6)
    );
}
