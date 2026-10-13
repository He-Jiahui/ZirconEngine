use super::*;
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeProjectSceneTransitionPolicyV1, ZrRuntimeProjectSceneTransitionRequestV1,
};

#[test]
fn gameplay_scene_transition_is_capability_gated_and_replaces_the_pending_request() {
    let core = CoreRuntime::new();
    let level = LevelSystem::new(
        WorldHandle::new(41),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).expect("gameplay host registration");

    let denied = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity: 0,
            delta_seconds: 0.016,
        },
        || {
            exports.call_with_capabilities(
                GAMEPLAY_MODULE,
                "request_scene_transition",
                vec![ScriptHostValue::String(
                    "res://scenes/intro.scene.toml".to_string(),
                )],
                &CapabilitySet::default(),
            )
        },
    )
    .expect_err("scene transitions require their dedicated capability");
    assert!(denied.to_string().contains("missing capability"));

    let capabilities = CapabilitySet::default().with("gameplay.scene_transition");
    let first = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity: 0,
            delta_seconds: 0.016,
        },
        || {
            exports.call_with_capabilities(
                GAMEPLAY_MODULE,
                "request_scene_transition",
                vec![ScriptHostValue::String(
                    "res://scenes/intro.scene.toml".to_string(),
                )],
                &capabilities,
            )
        },
    )
    .expect("capability grants a scene transition request");
    let ScriptHostValue::Int(first_id) = first else {
        panic!("scene transition should return a request id");
    };
    assert!(first_id >= 0);

    let first_request = level.with_world(|world| {
        world
            .get_resource::<ZrRuntimeProjectSceneTransitionRequestV1>()
            .cloned()
            .expect("first request is stored for the project transition owner")
    });
    assert_eq!(first_request.request_id, first_id as u64);
    assert_eq!(first_request.scene_uri, "res://scenes/intro.scene.toml");
    assert_eq!(
        first_request.policy,
        ZrRuntimeProjectSceneTransitionPolicyV1::ReplaceActive
    );

    let invalid = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity: 0,
            delta_seconds: 0.016,
        },
        || {
            exports.call_with_capabilities(
                GAMEPLAY_MODULE,
                "request_scene_transition",
                vec![ScriptHostValue::String(
                    "res://scenes/../outside.scene.toml".to_string(),
                )],
                &capabilities,
            )
        },
    )
    .expect_err("non-canonical project scene URI is rejected");
    assert!(invalid
        .to_string()
        .contains("canonical forward-slash segments"));
    assert_eq!(
        level.with_world(|world| {
            world
                .get_resource::<ZrRuntimeProjectSceneTransitionRequestV1>()
                .cloned()
        }),
        Some(first_request.clone()),
        "a rejected request must preserve the last pending transition"
    );

    let second = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity: 0,
            delta_seconds: 0.016,
        },
        || {
            exports.call_with_capabilities(
                GAMEPLAY_MODULE,
                "request_scene_transition",
                vec![ScriptHostValue::String(
                    "res://scenes/next.scene.toml".to_string(),
                )],
                &capabilities,
            )
        },
    )
    .expect("second transition request");
    let ScriptHostValue::Int(second_id) = second else {
        panic!("scene transition should return a request id");
    };
    assert!(second_id > first_id);
    let current = level.with_world(|world| {
        world
            .get_resource::<ZrRuntimeProjectSceneTransitionRequestV1>()
            .cloned()
            .expect("latest request remains available to the project transition owner")
    });
    assert_eq!(current.request_id, second_id as u64);
    assert_eq!(current.scene_uri, "res://scenes/next.scene.toml");
}

#[test]
fn gameplay_pose_exports_update_entity_transform() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let target = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
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
fn gameplay_pose_exports_convert_world_trs_through_non_uniform_parent() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test parent spawn should succeed");
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test child spawn should succeed");
    let target = world
        .spawn_node(NodeKind::Empty)
        .expect("test target spawn should succeed");
    world
        .update_transform(
            parent,
            Transform {
                translation: Vec3::new(8.0, 2.0, -3.0),
                rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                scale: Vec3::new(2.0, 3.0, 4.0),
            },
        )
        .unwrap();
    world.set_parent_checked(entity, Some(parent)).unwrap();
    world
        .update_transform(
            target,
            Transform::from_translation(Vec3::new(-4.0, 6.0, 5.0)),
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

    with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || {
            for (name, args) in [
                (
                    "set_position",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(1.0),
                        ScriptHostValue::Float(2.0),
                        ScriptHostValue::Float(3.0),
                    ],
                ),
                (
                    "translate",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(2.0),
                        ScriptHostValue::Float(-1.0),
                        ScriptHostValue::Float(4.0),
                    ],
                ),
            ] {
                exports
                    .call_with_capabilities(GAMEPLAY_MODULE, name, args, &capabilities)
                    .expect("world-space pose update should succeed");
            }
        },
    );

    assert_vec3_close(
        level.with_world(|world| world.world_transform(entity).unwrap().translation),
        Vec3::new(3.0, 1.0, 7.0),
    );

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
                        ScriptHostValue::Float(1.0),
                        ScriptHostValue::Float(1.0),
                        ScriptHostValue::Float(1.0),
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
    assert_vec3_close(transform.translation, Vec3::new(-3.5, 7.25, 4.75));
    assert_vec3_close(transform.scale, Vec3::ONE);
    assert_quat_close(
        transform.rotation,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    );
}

#[test]
fn gameplay_pose_export_rejects_world_rotation_that_requires_local_shear() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test parent spawn should succeed");
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test child spawn should succeed");
    world
        .update_transform(
            parent,
            Transform {
                translation: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::new(2.0, 1.0, 4.0),
            },
        )
        .unwrap();
    world.set_parent_checked(entity, Some(parent)).unwrap();
    let level = LevelSystem::new(
        WorldHandle::new(45),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");

    let result = with_script_runtime_call_context(
        ScriptRuntimeCallContext {
            core: core.weak(),
            level: level.clone(),
            entity,
            delta_seconds: 0.016,
        },
        || {
            // A 45-degree yaw mixes axes with unequal parent scales and requires local shear.
            exports.call_with_capabilities(
                GAMEPLAY_MODULE,
                "face_direction",
                vec![
                    ScriptHostValue::Int(entity as i64),
                    ScriptHostValue::Float(1.0),
                    ScriptHostValue::Float(-1.0),
                ],
                &capabilities,
            )
        },
    );

    let error = result.expect_err("a sheared local matrix cannot be stored as local TRS");
    assert!(error
        .to_string()
        .contains("cannot represent world transform"));
    assert_eq!(
        level.with_world(|world| world.local_transform(entity).unwrap()),
        Transform::default(),
        "failed world-to-local conversion must leave authored transform unchanged"
    );
}

#[test]
fn gameplay_position_exports_preserve_existing_world_shear_under_parent_scale() {
    let core = CoreRuntime::new();
    let mut world = World::empty();
    let parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test parent spawn should succeed");
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test child spawn should succeed");
    world
        .update_transform(
            parent,
            Transform {
                translation: Vec3::new(3.0, 4.0, 5.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(2.0, 1.0, 4.0),
            },
        )
        .unwrap();
    let child_local = Transform {
        translation: Vec3::new(1.0, 2.0, -1.0),
        rotation: Quat::from_rotation_y(0.6),
        scale: Vec3::new(1.0, 2.0, 0.5),
    };
    world.update_transform(entity, child_local).unwrap();
    world.set_parent_checked(entity, Some(parent)).unwrap();
    let initial_world_matrix = world.world_matrix(entity).unwrap();

    let level = LevelSystem::new(
        WorldHandle::new(46),
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
                    "set_position",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(-2.0),
                        ScriptHostValue::Float(7.0),
                        ScriptHostValue::Float(1.5),
                    ],
                    &capabilities,
                )
                .expect("set position should preserve representable local TRS");
            exports
                .call_with_capabilities(
                    GAMEPLAY_MODULE,
                    "translate",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::Float(1.0),
                        ScriptHostValue::Float(-2.0),
                        ScriptHostValue::Float(3.0),
                    ],
                    &capabilities,
                )
                .expect("translate should preserve representable local TRS");
        },
    );

    let (world_matrix, local_transform) = level.with_world(|world| {
        (
            world.world_matrix(entity).unwrap(),
            world.local_transform(entity).unwrap(),
        )
    });
    let actual = world_matrix.to_cols_array();
    let initial = initial_world_matrix.to_cols_array();
    for index in 0..12 {
        assert!(
            (actual[index] - initial[index]).abs() <= 0.0001,
            "position writes must preserve world linear component {index}: before={}, after={}",
            initial[index],
            actual[index]
        );
    }
    assert_quat_close(local_transform.rotation, child_local.rotation);
    assert_vec3_close(local_transform.scale, child_local.scale);
    assert_vec3_close(world_matrix.w_axis.truncate(), Vec3::new(-1.0, 5.0, 4.5));
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
