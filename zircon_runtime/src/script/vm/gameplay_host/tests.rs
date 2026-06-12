use std::sync::{Arc, Mutex};

use super::*;
use crate::core::framework::scene::WorldHandle;
use crate::core::CoreRuntime;
use crate::scene::{LevelMetadata, LevelSystem, World};
use crate::script::{
    with_script_runtime_call_context, CapabilitySet, HostExportRegistry, HostRegistry,
    ScriptRuntimeCallContext,
};

#[test]
fn gameplay_pose_exports_update_entity_transform() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty);
    let target = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(
            target,
            Transform::from_translation(Vec3::new(3.0, 0.0, -2.0)),
        )
        .unwrap();

    let level = LevelSystem::new(
        WorldHandle::new(42),
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
                    "set_scale",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(1.2),
                        ScriptHostValue::Float(0.9),
                        ScriptHostValue::Float(1.1),
                    ],
                    &capabilities,
                )
                .unwrap();
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "face_direction",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(1.0),
                        ScriptHostValue::Float(0.0),
                    ],
                    &capabilities,
                )
                .unwrap();
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "follow_position",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Int(target as i64),
                        ScriptHostValue::Float(0.5),
                        ScriptHostValue::Float(1.25),
                        ScriptHostValue::Float(-0.25),
                    ],
                    &capabilities,
                )
                .unwrap();
        },
    );

    let transform = level.with_world(|world| world.world_transform(entity).unwrap());
    assert_vec3_close(transform.translation, Vec3::new(3.5, 1.25, -2.25));
    assert_vec3_close(transform.scale, Vec3::new(1.2, 0.9, 1.1));
    assert_quat_close(
        transform.rotation,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    );
}

#[test]
fn gameplay_host_spawn_model_sets_bindings_and_hud_text() {
    let core = CoreRuntime::new();
    let world = World::empty();
    let level = LevelSystem::new(
        WorldHandle::new(43),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    let spawned = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity: 0,
            delta_seconds: 0.016,
        },
        || {
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "spawn_model",
                    vec![
                        ScriptHostValue::String("spawned ghoul".to_string()),
                        ScriptHostValue::String("[1.0,0.0,-2.0]".to_string()),
                        ScriptHostValue::String("res://models/ghoul_capsule.model.toml".to_string()),
                        ScriptHostValue::String(
                            "46b108bb-3d1b-48b5-80b4-2e10b8fbd080".to_string(),
                        ),
                        ScriptHostValue::String(
                            r#"[{"package":"vampire_game","module":"main","enabled":true,"properties":{"role":"enemy","hp":15}}]"#
                                .to_string(),
                        ),
                    ],
                    &capabilities,
                )
                .unwrap()
        },
    );
    let ScriptHostValue::Int(spawned) = spawned else {
        panic!("spawn_model should return an entity id");
    };
    let spawned = spawned as u64;

    with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity: spawned,
            delta_seconds: 0.016,
        },
        || {
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "set_hud_text",
                    vec![
                        ScriptHostValue::Int(spawned as i64),
                        ScriptHostValue::String("Lv 1\nBuff: none".to_string()),
                    ],
                    &capabilities,
                )
                .unwrap();
        },
    );

    let (position, count, hud) = level.with_world(|world| {
        (
            world.world_transform(spawned).unwrap().translation,
            world
                .node_records()
                .into_iter()
                .filter(|node| {
                    world
                        .dynamic_component(node.id, SCRIPT_BINDINGS_COMPONENT)
                        .is_some_and(|bindings| {
                            script_binding_property_matches(bindings, "role", "enemy")
                        })
                })
                .count(),
            world
                .dynamic_component(spawned, "gameplay.hud_text")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
        )
    });
    assert_vec3_close(position, Vec3::new(1.0, 0.0, -2.0));
    assert_eq!(count, 1);
    assert_eq!(hud.as_deref(), Some("Lv 1\nBuff: none"));
}

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

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    let delta = (actual - expected).abs();
    assert!(
        delta.max_element() <= 0.0001,
        "expected {expected:?}, received {actual:?}"
    );
}

fn assert_quat_close(actual: Quat, expected: Quat) {
    let delta = (actual.to_array(), expected.to_array());
    let max_component_delta = delta
        .0
        .iter()
        .zip(delta.1.iter())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        max_component_delta <= 0.0001,
        "expected {expected:?}, received {actual:?}"
    );
}
