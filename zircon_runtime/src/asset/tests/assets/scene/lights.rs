use super::*;

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
    assert!(document.contains("ambient_light"));
    assert!(document.contains("rect_light"));
}
