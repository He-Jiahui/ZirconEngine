use super::*;

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
                    post_process_settings: None,
                    ..SceneCameraAsset::default()
                }),
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
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
                script_bindings: Vec::new(),
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
                    render_queue: 0,
                    material_queue: 0,
                    order_in_layer: 0,
                    depth_bias: 0.0,
                    morph_weights: vec![0.5, 1.0],
                    primitives: Vec::new(),
                    lods: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
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
                script_bindings: Vec::new(),
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
                    volumetric: false,
                }),
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
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
                script_bindings: Vec::new(),
            },
        ],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
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
