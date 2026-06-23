use super::*;

#[test]
fn gameplay_host_current_hp_and_particle_sprites_use_dynamic_components() {
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
                "properties": { "role": "player", "hp": 73.0 }
            }]),
        )
        .unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(46),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    let hp = with_script_runtime_call_context(
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
                    "current_hp",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(120.0),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(hp, ScriptHostValue::Float(73.0));

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
                    "set_particle_sprites",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String(
                            r#"{"style":"blood_bolt","sprites":[{"position":[0.0,1.0,0.0],"size":0.35,"color":[1.0,0.0,0.0,0.8],"intensity":1.5}]}"#
                                .to_string(),
                        ),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );

    let particles = level.with_world(|world| {
        world
            .dynamic_component(entity, "render.particle_sprites")
            .cloned()
    });
    assert_eq!(
        particles
            .as_ref()
            .and_then(|value| value.get("style"))
            .and_then(serde_json::Value::as_str),
        Some("blood_bolt")
    );
    assert_eq!(
        particles
            .as_ref()
            .and_then(|value| value.get("sprites"))
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(1)
    );
}

#[test]
fn gameplay_host_component_string_reads_string_dynamic_state() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            entity,
            "gameplay.control_state",
            serde_json::json!("start_game"),
        )
        .unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(49),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    let action = with_script_runtime_call_context(
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
                    "component_string",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String("gameplay.control_state".to_string()),
                        ScriptHostValue::String("none".to_string()),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(action, ScriptHostValue::String("start_game".to_string()));

    let missing = with_script_runtime_call_context(
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
                    "component_string",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String("missing".to_string()),
                        ScriptHostValue::String("none".to_string()),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );
    assert_eq!(missing, ScriptHostValue::String("none".to_string()));
}
