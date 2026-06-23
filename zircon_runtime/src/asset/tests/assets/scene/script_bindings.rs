use super::*;

#[test]
fn scene_asset_toml_roundtrip_preserves_script_bindings() {
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 80,
            name: "Player".to_string(),
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
            script_bindings: vec![SceneScriptBindingAsset {
                package: "vampire_game".to_string(),
                module: "player".to_string(),
                enabled: true,
                update: true,
                fixed_update: false,
                properties: std::collections::BTreeMap::from([(
                    "move_speed".to_string(),
                    serde_json::json!(5.5),
                )]),
            }],
        }],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert!(document.contains("script_bindings"));
    assert!(document.contains("vampire_game"));
    assert!(document.contains("fixed_update = false"));
}
