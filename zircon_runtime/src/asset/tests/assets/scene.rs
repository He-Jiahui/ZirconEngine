use crate::asset::{
    AssetReference, AssetUri, AssetUuid, PrefabInstanceAsset, SceneAmbientLightAsset,
    SceneAnimationGraphPlayerAsset, SceneAnimationPlayerAsset, SceneAnimationSequencePlayerAsset,
    SceneAnimationSkeletonAsset, SceneAnimationStateMachinePlayerAsset, SceneAsset,
    SceneAssetManagementRecord, SceneAssetManagementRecordSet, SceneAssetOverview,
    SceneCameraAsset, SceneCameraTargetAsset, SceneColliderAsset, SceneColliderShapeAsset,
    SceneDirectionalLightAsset, SceneEntityAsset, SceneEntityManagementRecordSet, SceneJointAsset,
    SceneJointKindAsset, SceneMeshInstanceAsset, SceneMobilityAsset, ScenePointLightAsset,
    SceneRectLightAsset, SceneRigidBodyAsset, SceneRigidBodyTypeAsset, SceneSpotLightAsset,
    SceneTerrainAsset, SceneTileMapAsset, SceneViewportRectAsset, TransformAsset,
};
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};
use crate::core::framework::render::{ProjectionMode, RenderCameraClearColor};
use crate::core::resource::ResourceId;

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
                    ..SceneCameraAsset::default()
                }),
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
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
                    mesh: None,
                    material: AssetReference::new(
                        AssetUuid::from_stable_label("robot-material"),
                        AssetUri::parse("res://materials/robot.zmaterial").unwrap(),
                    ),
                    primitives: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
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
                ambient_light: None,
                directional_light: Some(SceneDirectionalLightAsset {
                    direction: [-0.4, -1.0, -0.25],
                    color: [1.0, 1.0, 1.0],
                    intensity: 3.0,
                }),
                point_light: None,
                rect_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
            },
        ],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
}

#[test]
fn scene_camera_asset_roundtrip_preserves_bevy_style_camera_fields() {
    let camera_target = AssetReference::new(
        AssetUuid::from_stable_label("camera-target"),
        AssetUri::parse("res://textures/camera-target.png").unwrap(),
    );
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 9,
            name: "RenderCamera".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 0x0000_0002,
            mobility: SceneMobilityAsset::Dynamic,
            camera: Some(SceneCameraAsset {
                projection_mode: ProjectionMode::Orthographic,
                fov_y_radians: 0.75,
                ortho_size: 12.0,
                z_near: 0.05,
                z_far: 500.0,
                target: SceneCameraTargetAsset::Texture {
                    texture: camera_target.clone(),
                },
                viewport: Some(SceneViewportRectAsset {
                    physical_position: [32, 48],
                    physical_size: [640, 360],
                    depth_min: 0.1,
                    depth_max: 0.9,
                }),
                order: 3,
                active: false,
                hdr: true,
                exposure_ev100: 11.0,
                clear_color: RenderCameraClearColor::None,
                msaa_samples: 4,
            }),
            mesh: None,
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: None,
        }],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert_eq!(loaded.direct_references(), vec![camera_target]);
    assert!(document.contains("projection_mode"));
    assert!(document.contains("camera-target.png"));
}

#[test]
fn scene_camera_asset_defaults_bevy_camera_fields_when_omitted() {
    let document = r#"
[[entities]]
entity = 9
name = "LegacyCamera"
parent = 0
active = true
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
camera = { fov_y_radians = 1.0, z_near = 0.25, z_far = 900.0 }
"#;

    let loaded = SceneAsset::from_toml_str(document).unwrap();
    let camera = loaded.entities[0].camera.as_ref().unwrap();

    assert_eq!(camera.projection_mode, ProjectionMode::Perspective);
    assert_eq!(camera.ortho_size, 5.0);
    assert!(matches!(
        &camera.target,
        SceneCameraTargetAsset::PrimarySurface
    ));
    assert_eq!(camera.viewport, None);
    assert_eq!(camera.order, 0);
    assert!(camera.active);
    assert!(!camera.hdr);
    assert_eq!(camera.clear_color, RenderCameraClearColor::Default);
    assert_eq!(camera.msaa_samples, 1);
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
                mesh: None,
                material: AssetReference::new(
                    AssetUuid::from_stable_label("hero-material"),
                    AssetUri::parse("res://materials/hero.zmaterial").unwrap(),
                ),
                primitives: Vec::new(),
            }),
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            rigid_body: Some(SceneRigidBodyAsset {
                body_type: SceneRigidBodyTypeAsset::Dynamic,
                mass: 2.5,
                linear_velocity: [0.25, 0.0, 0.0],
                angular_velocity: [0.0, 0.25, 0.0],
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
            terrain: None,
            tilemap: None,
            prefab_instance: None,
        }],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert!(document.contains("rigid_body"));
    assert!(document.contains("animation_state_machine_player"));
}

#[test]
fn scene_asset_parses_uuid_url_mesh_bindings() {
    let document = r#"
[[entities]]
entity = 2
name = "Model"
parent = 0
active = true
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }

[entities.mesh.model]
uuid = "00000000-0000-0000-0000-000000000011"
url = "res://models/robot.gltf"

[entities.mesh.material]
uuid = "00000000-0000-0000-0000-000000000012"
url = "res://materials/robot.zmaterial"
"#;

    let loaded = SceneAsset::from_toml_str(document).unwrap();
    let mesh = loaded.entities[0].mesh.as_ref().unwrap();

    assert_eq!(
        mesh.model.locator,
        AssetUri::parse("res://models/robot.gltf").unwrap()
    );
    assert_eq!(
        mesh.material.locator,
        AssetUri::parse("res://materials/robot.zmaterial").unwrap()
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
    assert!(entity.ambient_light.is_none());
    assert!(entity.rect_light.is_none());
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
                ambient_light: None,
                directional_light: None,
                point_light: Some(ScenePointLightAsset {
                    color: [0.4, 0.7, 1.0],
                    intensity: 5.5,
                    range: 11.0,
                }),
                rect_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
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
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
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
                terrain: None,
                tilemap: None,
                prefab_instance: None,
            },
        ],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert!(document.contains("point_light"));
    assert!(document.contains("spot_light"));
}

#[test]
fn scene_asset_toml_roundtrip_preserves_ambient_and_rect_lights() {
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 50,
                name: "Ambient".to_string(),
                parent: None,
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: None,
                ambient_light: Some(SceneAmbientLightAsset {
                    color: [0.15, 0.2, 0.35],
                    intensity: 120.0,
                    affects_lightmapped_meshes: false,
                }),
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
            },
            SceneEntityAsset {
                entity: 51,
                name: "Softbox".to_string(),
                parent: None,
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: Some(SceneRectLightAsset {
                    color: [1.0, 0.75, 0.45],
                    intensity: 80_000.0,
                    range: 16.0,
                    size: [4.0, 2.0],
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
                terrain: None,
                tilemap: None,
                prefab_instance: None,
            },
        ],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert!(document.contains("ambient_light"));
    assert!(document.contains("rect_light"));
}

#[test]
fn scene_asset_overview_reports_entity_component_and_reference_counts() {
    let camera_target = asset_ref(
        "camera-target-overview",
        "res://textures/camera-target-overview.png",
    );
    let model = asset_ref("hero-model-overview", "res://models/hero-overview.gltf");
    let mesh = asset_ref(
        "hero-mesh-overview",
        "res://models/hero-overview.gltf#Mesh0/Primitive0",
    );
    let material = asset_ref(
        "hero-material-overview",
        "res://materials/hero-overview.zmaterial",
    );
    let physics_material = asset_ref(
        "hero-physics-material-overview",
        "res://physics/hero-overview.physics_material.toml",
    );
    let animation_graph = asset_ref(
        "hero-graph-overview",
        "res://animation/hero-overview.graph.zranim",
    );
    let terrain = asset_ref(
        "hero-terrain-overview",
        "res://terrain/hero-overview.zterrain",
    );
    let tilemap = asset_ref(
        "hero-tilemap-overview",
        "res://tilemaps/hero-overview.ztilemap",
    );
    let prefab = asset_ref(
        "hero-prefab-overview",
        "res://prefabs/hero-overview.zprefab",
    );

    let mut camera_entity = empty_scene_entity(10, "Camera");
    camera_entity.camera = Some(SceneCameraAsset {
        target: SceneCameraTargetAsset::Texture {
            texture: camera_target.clone(),
        },
        ..SceneCameraAsset::default()
    });

    let mut hero_entity = empty_scene_entity(11, "Hero");
    hero_entity.parent = Some(10);
    hero_entity.active = false;
    hero_entity.render_layer_mask = 0x0000_0010;
    hero_entity.mobility = SceneMobilityAsset::Static;
    hero_entity.mesh = Some(SceneMeshInstanceAsset {
        model: model.clone(),
        mesh: Some(mesh.clone()),
        material: material.clone(),
        primitives: Vec::new(),
    });
    hero_entity.point_light = Some(ScenePointLightAsset {
        color: [1.0, 0.8, 0.6],
        intensity: 4.0,
        range: 12.0,
    });
    hero_entity.rigid_body = Some(SceneRigidBodyAsset {
        body_type: SceneRigidBodyTypeAsset::Dynamic,
        mass: 3.0,
        linear_velocity: [0.0, 0.0, 0.0],
        angular_velocity: [0.0, 0.0, 0.0],
        linear_damping: 0.0,
        angular_damping: 0.0,
        gravity_scale: 1.0,
        can_sleep: true,
        lock_translation: [false, false, false],
        lock_rotation: [false, false, false],
    });
    hero_entity.collider = Some(SceneColliderAsset {
        shape: SceneColliderShapeAsset::Sphere { radius: 0.5 },
        sensor: false,
        layer: 1,
        collision_group: 1,
        collision_mask: u32::MAX,
        material: Some(physics_material.clone()),
        material_override: None,
        local_transform: TransformAsset::default(),
    });
    hero_entity.joint = Some(SceneJointAsset {
        joint_type: SceneJointKindAsset::Fixed,
        connected_entity: Some(10),
        anchor: [0.0, 0.0, 0.0],
        axis: [0.0, 1.0, 0.0],
        limits: None,
        collide_connected: false,
    });
    hero_entity.animation_graph_player = Some(SceneAnimationGraphPlayerAsset {
        graph: animation_graph.clone(),
        parameters: std::collections::BTreeMap::new(),
        playing: true,
    });
    hero_entity.terrain = Some(SceneTerrainAsset {
        terrain: terrain.clone(),
    });
    hero_entity.tilemap = Some(SceneTileMapAsset {
        tilemap: tilemap.clone(),
    });
    hero_entity.prefab_instance = Some(PrefabInstanceAsset {
        prefab: prefab.clone(),
        local_transform: TransformAsset::default(),
        overrides: Vec::new(),
    });

    let scene = SceneAsset {
        entities: vec![camera_entity, hero_entity],
    };

    assert_eq!(
        scene.direct_references(),
        vec![
            camera_target.clone(),
            model.clone(),
            mesh.clone(),
            material.clone(),
            physics_material.clone(),
            animation_graph.clone(),
            terrain.clone(),
            tilemap.clone(),
            prefab.clone(),
        ]
    );
    assert_eq!(
        scene.entities[1].direct_references(),
        vec![
            model,
            mesh,
            material,
            physics_material,
            animation_graph,
            terrain,
            tilemap,
            prefab
        ]
    );

    let overview: SceneAssetOverview = scene.overview();

    assert_eq!(overview.entity_count, 2);
    assert_eq!(overview.active_entity_count, 1);
    assert_eq!(overview.root_entity_count, 1);
    assert_eq!(overview.camera_count, 1);
    assert_eq!(overview.mesh_instance_count, 1);
    assert_eq!(overview.direct_mesh_reference_count, 1);
    assert_eq!(overview.mesh_primitive_binding_count, 0);
    assert_eq!(overview.mesh_material_binding_count, 1);
    assert_eq!(overview.collider_material_binding_count, 1);
    assert_eq!(overview.light_count, 1);
    assert_eq!(overview.physics_component_count, 3);
    assert_eq!(overview.animation_binding_count, 1);
    assert_eq!(overview.terrain_count, 1);
    assert_eq!(overview.tilemap_count, 1);
    assert_eq!(overview.prefab_instance_count, 1);
    assert_eq!(overview.direct_reference_count, 9);

    let camera_overview = &overview.entities[0];
    assert_eq!(camera_overview.entity, 10);
    assert_eq!(camera_overview.name, "Camera");
    assert_eq!(camera_overview.direct_reference_count, 1);
    assert!(camera_overview.has_camera);
    assert!(!camera_overview.has_mesh);

    let hero_overview = &overview.entities[1];
    assert_eq!(hero_overview.entity, 11);
    assert_eq!(hero_overview.parent, Some(10));
    assert!(!hero_overview.active);
    assert_eq!(hero_overview.render_layer_mask, 0x0000_0010);
    assert_eq!(hero_overview.mobility, SceneMobilityAsset::Static);
    assert_eq!(hero_overview.direct_reference_count, 8);
    assert!(hero_overview.has_mesh);
    assert!(hero_overview.has_direct_mesh_reference);
    assert_eq!(hero_overview.direct_mesh_reference_count, 1);
    assert_eq!(hero_overview.mesh_primitive_binding_count, 0);
    assert!(hero_overview.has_collider_material);
    assert_eq!(hero_overview.light_count(), 1);
    assert_eq!(hero_overview.physics_component_count(), 3);
    assert_eq!(hero_overview.animation_binding_count(), 1);

    let scene_id = ResourceId::from_stable_label("res://scenes/overview.scene.toml");
    let record: SceneAssetManagementRecord = scene.management_record(scene_id);

    assert_eq!(record.scene_id, scene_id);
    assert_eq!(record.overview, overview);
}

#[test]
fn scene_asset_overview_handles_empty_scenes() {
    let scene = SceneAsset {
        entities: Vec::new(),
    };

    let overview = scene.overview();

    assert_eq!(overview.entity_count, 0);
    assert_eq!(overview.direct_reference_count, 0);
    assert!(overview.entities.is_empty());
    assert!(scene.direct_references().is_empty());
}

#[test]
fn scene_asset_management_record_set_sorts_and_summarizes_records() {
    let camera_target = asset_ref(
        "camera-target-record-set",
        "res://textures/camera-target-record-set.png",
    );
    let model = asset_ref("scene-record-set-model", "res://models/record-set.gltf");
    let material = asset_ref(
        "scene-record-set-material",
        "res://materials/record-set.zmaterial",
    );
    let clip = asset_ref(
        "scene-record-set-clip",
        "res://animation/record-set.clip.zranim",
    );

    let mut camera_entity = empty_scene_entity(20, "RecordSetCamera");
    camera_entity.camera = Some(SceneCameraAsset {
        target: SceneCameraTargetAsset::Texture {
            texture: camera_target,
        },
        ..SceneCameraAsset::default()
    });

    let mut actor_entity = empty_scene_entity(21, "RecordSetActor");
    actor_entity.parent = Some(20);
    actor_entity.active = false;
    actor_entity.mesh = Some(SceneMeshInstanceAsset {
        model,
        mesh: None,
        material,
        primitives: Vec::new(),
    });
    actor_entity.directional_light = Some(SceneDirectionalLightAsset {
        direction: [0.0, -1.0, 0.0],
        color: [1.0, 1.0, 1.0],
        intensity: 2.0,
    });
    actor_entity.rigid_body = Some(SceneRigidBodyAsset {
        body_type: SceneRigidBodyTypeAsset::Dynamic,
        mass: 1.0,
        linear_velocity: [0.0, 0.0, 0.0],
        angular_velocity: [0.0, 0.0, 0.0],
        linear_damping: 0.0,
        angular_damping: 0.0,
        gravity_scale: 1.0,
        can_sleep: true,
        lock_translation: [false, false, false],
        lock_rotation: [false, false, false],
    });
    actor_entity.animation_player = Some(SceneAnimationPlayerAsset {
        clip,
        playback_speed: 1.0,
        time_seconds: 0.0,
        weight: 1.0,
        looping: true,
        playing: true,
    });

    let populated_scene = SceneAsset {
        entities: vec![camera_entity, actor_entity],
    };
    let empty_scene = SceneAsset {
        entities: Vec::new(),
    };
    let populated_id = ResourceId::from_stable_label("scene:record-set-populated");
    let empty_id = ResourceId::from_stable_label("scene:record-set-empty");

    let record_set = SceneAssetManagementRecordSet::from_records(vec![
        populated_scene.management_record(populated_id),
        empty_scene.management_record(empty_id),
    ]);

    let mut expected_ids = vec![empty_id, populated_id];
    expected_ids.sort();
    let record_ids = record_set
        .records
        .iter()
        .map(|record| record.scene_id)
        .collect::<Vec<_>>();
    assert_eq!(record_ids, expected_ids);
    assert_eq!(record_set.records.len(), 2);
    let summary = &record_set.summary;
    assert_eq!(summary.scene_count, 2);
    assert_eq!(summary.entity_count, 2);
    assert_eq!(summary.active_entity_count, 1);
    assert_eq!(summary.root_entity_count, 1);
    assert_eq!(summary.direct_reference_count, 4);
    assert_eq!(summary.camera_count, 1);
    assert_eq!(summary.mesh_instance_count, 1);
    assert_eq!(summary.direct_mesh_reference_count, 0);
    assert_eq!(summary.mesh_primitive_binding_count, 0);
    assert_eq!(summary.mesh_material_binding_count, 1);
    assert_eq!(summary.collider_material_binding_count, 0);
    assert_eq!(summary.light_count, 1);
    assert_eq!(summary.physics_component_count, 1);
    assert_eq!(summary.animation_binding_count, 1);
    assert_eq!(summary.terrain_count, 0);
    assert_eq!(summary.tilemap_count, 0);
    assert_eq!(summary.prefab_instance_count, 0);

    let entity_record_set = SceneEntityManagementRecordSet::from_records(
        record_set
            .records
            .iter()
            .flat_map(SceneAssetManagementRecord::entity_management_records)
            .collect(),
    );

    assert_eq!(
        populated_scene
            .entity_management_records(populated_id)
            .iter()
            .map(|record| record.entity.entity)
            .collect::<Vec<_>>(),
        vec![20, 21]
    );
    assert_eq!(
        entity_record_set
            .records
            .iter()
            .map(|record| (record.scene_id, record.entity.entity))
            .collect::<Vec<_>>(),
        vec![(populated_id, 20), (populated_id, 21)]
    );
    assert_eq!(entity_record_set.summary.scene_count, 1);
    assert_eq!(entity_record_set.summary.entity_count, 2);
    assert_eq!(entity_record_set.summary.active_entity_count, 1);
    assert_eq!(entity_record_set.summary.root_entity_count, 1);
    assert_eq!(entity_record_set.summary.direct_reference_count, 4);
    assert_eq!(entity_record_set.summary.camera_count, 1);
    assert_eq!(entity_record_set.summary.mesh_instance_count, 1);
    assert_eq!(entity_record_set.summary.direct_mesh_reference_count, 0);
    assert_eq!(entity_record_set.summary.mesh_primitive_binding_count, 0);
    assert_eq!(entity_record_set.summary.mesh_material_binding_count, 1);
    assert_eq!(entity_record_set.summary.collider_material_binding_count, 0);
    assert_eq!(entity_record_set.summary.light_count, 1);
    assert_eq!(entity_record_set.summary.physics_component_count, 1);
    assert_eq!(entity_record_set.summary.animation_binding_count, 1);
    assert_eq!(entity_record_set.summary.terrain_count, 0);
    assert_eq!(entity_record_set.summary.tilemap_count, 0);
    assert_eq!(entity_record_set.summary.prefab_instance_count, 0);
}

fn empty_scene_entity(entity: u64, name: &str) -> SceneEntityAsset {
    SceneEntityAsset {
        entity,
        name: name.to_string(),
        parent: None,
        transform: TransformAsset::default(),
        active: true,
        render_layer_mask: 0x0000_0001,
        mobility: SceneMobilityAsset::Dynamic,
        camera: None,
        mesh: None,
        ambient_light: None,
        directional_light: None,
        point_light: None,
        rect_light: None,
        spot_light: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
    }
}

fn asset_ref(label: &str, uri: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_stable_label(label),
        AssetUri::parse(uri).unwrap(),
    )
}
