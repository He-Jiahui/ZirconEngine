use crate::asset::{
    AssetReference, AssetUri, AssetUuid, SceneAnimationGraphPlayerAsset, SceneAnimationPlayerAsset,
    SceneAnimationSequencePlayerAsset, SceneAnimationSkeletonAsset,
    SceneAnimationStateMachinePlayerAsset, SceneAsset, SceneCameraAsset, SceneColliderAsset,
    SceneColliderShapeAsset, SceneDirectionalLightAsset, SceneEntityAsset, SceneJointAsset,
    SceneJointKindAsset, SceneMeshInstanceAsset, SceneMobilityAsset, ScenePointLightAsset,
    SceneRigidBodyAsset, SceneRigidBodyTypeAsset, SceneSpotLightAsset, TransformAsset,
};
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};

#[test]
fn scene_asset_toml_roundtrip_preserves_entities_and_bindings() {
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 1,
                name: "Camera".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 2.0, 5.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    fov_y_radians: 1.0471976,
                    z_near: 0.1,
                    z_far: 200.0,
                }),
                mesh: None,
                directional_light: None,
                point_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
            },
            SceneEntityAsset {
                entity: 2,
                name: "Model".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: AssetReference::new(
                        AssetUuid::from_stable_label("robot-model"),
                        AssetUri::parse("res://models/robot.gltf").unwrap(),
                    ),
                    material: AssetReference::new(
                        AssetUuid::from_stable_label("robot-material"),
                        AssetUri::parse("res://materials/robot.material.toml").unwrap(),
                    ),
                }),
                directional_light: None,
                point_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
            },
            SceneEntityAsset {
                entity: 3,
                name: "Sun".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 4.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0004,
                mobility: SceneMobilityAsset::Static,
                camera: None,
                mesh: None,
                directional_light: Some(SceneDirectionalLightAsset {
                    direction: [-0.4, -1.0, -0.25],
                    color: [1.0, 1.0, 1.0],
                    intensity: 3.0,
                }),
                point_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
            },
        ],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
}

#[test]
fn scene_asset_toml_roundtrip_preserves_physics_and_animation_components() {
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 12,
            name: "AnimatedRigidBody".to_string(),
            parent: Some(1),
            transform: TransformAsset {
                translation: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
            active: true,
            render_layer_mask: 0x0000_0003,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: Some(SceneMeshInstanceAsset {
                model: AssetReference::new(
                    AssetUuid::from_stable_label("hero-model"),
                    AssetUri::parse("res://models/hero.gltf").unwrap(),
                ),
                material: AssetReference::new(
                    AssetUuid::from_stable_label("hero-material"),
                    AssetUri::parse("res://materials/hero.material.toml").unwrap(),
                ),
            }),
            directional_light: None,
            point_light: None,
            spot_light: None,
            rigid_body: Some(SceneRigidBodyAsset {
                body_type: SceneRigidBodyTypeAsset::Dynamic,
                mass: 2.5,
                linear_damping: 0.15,
                angular_damping: 0.05,
                gravity_scale: 1.0,
                can_sleep: true,
                lock_translation: [false, false, false],
                lock_rotation: [false, true, false],
            }),
            collider: Some(SceneColliderAsset {
                shape: SceneColliderShapeAsset::Box {
                    half_extents: [0.5, 1.0, 0.5],
                },
                sensor: false,
                layer: 2,
                collision_group: 4,
                collision_mask: 0x0000_00ff,
                material: Some(AssetReference::new(
                    AssetUuid::from_stable_label("hero-physics-material"),
                    AssetUri::parse("res://physics/hero.physics_material.toml").unwrap(),
                )),
                material_override: Some(PhysicsMaterialMetadata {
                    static_friction: 0.7,
                    dynamic_friction: 0.4,
                    restitution: 0.2,
                    friction_combine: PhysicsCombineRule::Maximum,
                    restitution_combine: PhysicsCombineRule::Average,
                }),
                local_transform: TransformAsset {
                    translation: [0.0, 0.5, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
            }),
            joint: Some(SceneJointAsset {
                joint_type: SceneJointKindAsset::Hinge,
                connected_entity: Some(1),
                anchor: [0.0, 1.0, 0.0],
                axis: [0.0, 1.0, 0.0],
                limits: Some([-0.5, 0.5]),
                collide_connected: false,
            }),
            animation_skeleton: Some(SceneAnimationSkeletonAsset {
                skeleton: AssetReference::new(
                    AssetUuid::from_stable_label("hero-skeleton"),
                    AssetUri::parse("res://animation/hero.skeleton.zranim").unwrap(),
                ),
            }),
            animation_player: Some(SceneAnimationPlayerAsset {
                clip: AssetReference::new(
                    AssetUuid::from_stable_label("hero-clip"),
                    AssetUri::parse("res://animation/hero.clip.zranim").unwrap(),
                ),
                playback_speed: 1.25,
                time_seconds: 0.5,
                weight: 0.8,
                looping: true,
                playing: true,
            }),
            animation_sequence_player: Some(SceneAnimationSequencePlayerAsset {
                sequence: AssetReference::new(
                    AssetUuid::from_stable_label("hero-sequence"),
                    AssetUri::parse("res://animation/hero.sequence.zranim").unwrap(),
                ),
                playback_speed: 1.0,
                time_seconds: 0.25,
                looping: false,
                playing: true,
            }),
            animation_graph_player: Some(SceneAnimationGraphPlayerAsset {
                graph: AssetReference::new(
                    AssetUuid::from_stable_label("hero-graph"),
                    AssetUri::parse("res://animation/hero.graph.zranim").unwrap(),
                ),
                parameters: std::collections::BTreeMap::from([
                    ("grounded".to_string(), AnimationParameterValue::Bool(true)),
                    ("speed".to_string(), AnimationParameterValue::Scalar(1.5)),
                ]),
                playing: true,
            }),
            animation_state_machine_player: Some(SceneAnimationStateMachinePlayerAsset {
                state_machine: AssetReference::new(
                    AssetUuid::from_stable_label("hero-state-machine"),
                    AssetUri::parse("res://animation/hero.state_machine.zranim").unwrap(),
                ),
                parameters: std::collections::BTreeMap::from([
                    ("grounded".to_string(), AnimationParameterValue::Bool(true)),
                    ("speed".to_string(), AnimationParameterValue::Scalar(1.5)),
                ]),
                active_state: Some("Locomotion".to_string()),
                playing: true,
            }),
        }],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert!(document.contains("rigid_body"));
    assert!(document.contains("animation_state_machine_player"));
}

#[test]
fn scene_asset_parses_legacy_locator_only_mesh_bindings() {
    let document = r#"
[[entities]]
entity = 2
name = "Model"
parent = 0
active = true
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
mesh = { model = "res://models/robot.gltf", material = "res://materials/robot.material.toml" }
"#;

    let loaded = SceneAsset::from_toml_str(document).unwrap();
    let mesh = loaded.entities[0].mesh.as_ref().unwrap();

    assert_eq!(
        mesh.model.locator,
        AssetUri::parse("res://models/robot.gltf").unwrap()
    );
    assert_eq!(
        mesh.material.locator,
        AssetUri::parse("res://materials/robot.material.toml").unwrap()
    );
}

#[test]
fn scene_asset_defaults_new_runtime_foundation_fields_when_omitted() {
    let document = r#"
[[entities]]
entity = 7
name = "Legacy"
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
active = true
"#;

    let loaded = SceneAsset::from_toml_str(document).unwrap();
    let entity = &loaded.entities[0];

    assert!(entity.active);
    assert_eq!(entity.render_layer_mask, 0x0000_0001);
    assert_eq!(entity.mobility, SceneMobilityAsset::Dynamic);
    assert!(entity.rigid_body.is_none());
    assert!(entity.collider.is_none());
    assert!(entity.joint.is_none());
    assert!(entity.animation_skeleton.is_none());
    assert!(entity.animation_player.is_none());
    assert!(entity.animation_sequence_player.is_none());
    assert!(entity.animation_graph_player.is_none());
    assert!(entity.animation_state_machine_player.is_none());
}

#[test]
fn scene_asset_toml_roundtrip_preserves_point_and_spot_lights() {
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 40,
                name: "Lamp".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [2.0, 3.0, 4.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: None,
                directional_light: None,
                point_light: Some(ScenePointLightAsset {
                    color: [0.4, 0.7, 1.0],
                    intensity: 5.5,
                    range: 11.0,
                }),
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
            },
            SceneEntityAsset {
                entity: 41,
                name: "StageSpot".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [-3.0, 6.0, 2.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0002,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: None,
                directional_light: None,
                point_light: None,
                spot_light: Some(SceneSpotLightAsset {
                    direction: [0.0, -1.0, 0.25],
                    color: [1.0, 0.8, 0.3],
                    intensity: 9.0,
                    range: 14.0,
                    inner_angle_radians: 0.2,
                    outer_angle_radians: 0.45,
                }),
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
            },
        ],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert!(document.contains("point_light"));
    assert!(document.contains("spot_light"));
}
