use super::*;

#[test]
fn artifact_store_roundtrips_material_assets_in_library() {
    let root = unique_temp_project_root("artifact_store");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let material = MaterialAsset {
        name: Some("Grid".to_string()),
        shader: asset_reference("res://shaders/pbr.wgsl"),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.8, 0.7, 0.6, 1.0],
        base_color_texture: Some(asset_reference("res://textures/grid.png")),
        normal_texture: None,
        metallic: 0.2,
        roughness: 0.7,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Material,
        AssetUri::parse("res://materials/grid.zmaterial").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(
            &paths,
            &metadata,
            &ImportedAsset::Material(material.clone()),
        )
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert!(artifact_uri.to_string().ends_with(".zasset"));
    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Material(material));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_material_assets_with_dynamic_property_values() {
    let root = unique_temp_project_root("artifact_store_material_dynamic_values");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Ghost Mist"

[shader]
uuid = "00000000-0000-0000-0000-000000000143"
url = "res://shaders/vampire_effect"

[overrides]
base_color = [0.42, 0.72, 0.86, 0.98]
metallic = 0.0
roughness = 0.9
emissive = [0.08, 0.18, 0.24]
double_sided = true

[overrides.alpha_mode]
mode = "opaque"
"#,
    )
    .unwrap();
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Material,
        AssetUri::parse("res://materials/ghost_mist.zmaterial").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(
            &paths,
            &metadata,
            &ImportedAsset::Material(material.clone()),
        )
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Material(material));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_data_assets_with_dynamic_json_values() {
    let root = unique_temp_project_root("artifact_store_data_dynamic_json");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let data = DataAsset {
        uri: AssetUri::parse("res://data/balance.json").unwrap(),
        format: DataAssetFormat::Json,
        text: r#"{"player":{"hp":120,"speed":5.5},"tags":["vampire",true,null]}"#.to_string(),
        canonical_json: serde_json::json!({
            "player": { "hp": 120, "speed": 5.5 },
            "tags": ["vampire", true, null],
            "spawn_count": 12_u64
        }),
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Data,
        AssetUri::parse("res://data/balance.json").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Data(data.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Data(data));

    let _ = fs::remove_dir_all(root);
}
