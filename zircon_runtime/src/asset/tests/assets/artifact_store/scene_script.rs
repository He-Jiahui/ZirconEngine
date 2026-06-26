use super::*;

#[test]
fn artifact_store_roundtrips_scene_assets_with_script_binding_json_values() {
    let root = unique_temp_project_root("artifact_store_scene_dynamic_json");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 2,
            name: "Player".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: u32::MAX,
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
                module: "main".to_string(),
                enabled: true,
                update: true,
                fixed_update: true,
                properties: BTreeMap::from([
                    ("role".to_string(), serde_json::json!("player")),
                    ("hp".to_string(), serde_json::json!(120.0)),
                    (
                        "loadout".to_string(),
                        serde_json::json!({
                            "weapon": "blood_bolt",
                            "pierce": 1,
                            "tags": ["starter", true, null]
                        }),
                    ),
                ]),
            }],
        }],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Scene(scene.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Scene(scene));

    let _ = fs::remove_dir_all(root);
}
