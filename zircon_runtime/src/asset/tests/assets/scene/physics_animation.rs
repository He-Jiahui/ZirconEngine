use super::*;

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
                mesh: None,
                material: AssetReference::new(
                    AssetUuid::from_stable_label("hero-material"),
                    AssetUri::parse("res://materials/hero.zmaterial").unwrap(),
                ),
                render_queue: 0,
                material_queue: 0,
                order_in_layer: 0,
                depth_bias: 0.0,
                morph_weights: Vec::new(),
                primitives: Vec::new(),
                lods: Vec::new(),
            }),
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: Some(SceneRigidBodyAsset {
                body_type: SceneRigidBodyTypeAsset::Dynamic,
                mass: 2.5,
                mass_properties: Default::default(),
                linear_velocity: [0.25, 0.0, 0.0],
                angular_velocity: [0.0, 0.25, 0.0],
                linear_damping: 0.15,
                angular_damping: 0.05,
                gravity_scale: 1.0,
                ccd_mode: Default::default(),
                sleep_policy: Default::default(),
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
                constraint: Default::default(),
                skeleton_binding: None,
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
            terrain: None,
            tilemap: None,
            prefab_instance: None,
            script_bindings: Vec::new(),
        }],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert!(document.contains("rigid_body"));
    assert!(document.contains("animation_state_machine_player"));
}
