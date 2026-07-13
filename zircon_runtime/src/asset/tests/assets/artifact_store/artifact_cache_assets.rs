use super::*;
use crate::core::framework::render::MaterialPropertyKind;

#[test]
fn artifact_store_roundtrips_physics_material_assets_in_artifact_cache() {
    let root = unique_temp_project_root("artifact_store_physics_material");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let physics_material = sample_physics_material_asset();
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::PhysicsMaterial,
        AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(
            &paths,
            &metadata,
            &ImportedAsset::PhysicsMaterial(physics_material.clone()),
        )
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_eq!(
        artifact_uri.to_string().contains("physics/materials/"),
        true
    );
    assert!(artifact_uri.to_string().ends_with(".zasset"));
    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::PhysicsMaterial(physics_material));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata() {
    let root = unique_temp_project_root("artifact_store_shader_toml_metadata");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let mut editor_metadata = toml::Table::new();
    editor_metadata.insert(
        "inspector_group".to_string(),
        toml::Value::String("PBR".into()),
    );
    editor_metadata.insert(
        "generated_at".to_string(),
        toml::Value::Datetime(
            "2026-06-11T12:30:00Z"
                .parse::<toml::value::Datetime>()
                .unwrap(),
        ),
    );
    let shader = ShaderAsset {
        uri: AssetUri::parse("res://shaders/pbr.zshader").unwrap(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }"
            .to_string(),
        wgsl_source: String::new(),
        import_path: Some("shaders/pbr.wgsl".to_string()),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: vec![
            ShaderImportRedirectAsset {
                source: "zircon::lighting".to_string(),
                redirect: Some(asset_reference("res://shaders/shared_lighting")),
            },
            ShaderImportRedirectAsset {
                source: "naga_oil::math".to_string(),
                redirect: None,
            },
        ],
        shader_defs: vec![
            RenderShaderDefinitionValue::uint("ALPHA_CLIP", 1),
            RenderShaderDefinitionValue::bool("USE_FOG", false),
        ],
        property_schema: vec![ShaderMaterialPropertyAsset {
            name: "tint".to_string(),
            kind: MaterialPropertyKind::Vec4,
            required: false,
            default: Some(toml::Value::Array(vec![
                toml::Value::Float(1.0),
                toml::Value::Float(0.8),
                toml::Value::Float(0.6),
                toml::Value::Float(1.0),
            ])),
            editor: BTreeMap::from([("widget".to_string(), "color".to_string())]),
        }],
        options: Vec::new(),
        texture_slots: vec![
            ShaderTextureSlotAsset {
                name: "base_color".to_string(),
                kind: "texture2d".to_string(),
                required: false,
                default: Some("white".to_string()),
                sampler: Some("linear_repeat".to_string()),
                group: Some("Surface".to_string()),
                label: Some("Base Color Texture".to_string()),
                option: None,
                st: false,
                editor: BTreeMap::from([("slot".to_string(), "base_color".to_string())]),
            },
            ShaderTextureSlotAsset {
                name: "mask".to_string(),
                kind: "texture2d".to_string(),
                required: false,
                default: None,
                sampler: None,
                group: None,
                label: None,
                option: None,
                st: false,
                editor: BTreeMap::new(),
            },
        ],
        shading_model: Some("standard_pbr".to_string()),
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: editor_metadata,
        pipeline_layout: Default::default(),
        validation_diagnostics: vec!["authoring note".to_string()],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Shader,
        AssetUri::parse("res://shaders/pbr.zshader").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Shader(shader.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert!(artifact_uri.to_string().contains("shaders/"));
    assert!(artifact_uri.to_string().ends_with(".zasset"));
    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Shader(shader));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_animation_sequence_assets_in_binary_artifact_cache() {
    let root = unique_temp_project_root("artifact_store_animation_sequence");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let sequence = sample_animation_sequence_asset();
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::AnimationSequence,
        AssetUri::parse("res://animation/hero.sequence.zranim").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(
            &paths,
            &metadata,
            &ImportedAsset::AnimationSequence(sequence.clone()),
        )
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert!(artifact_uri.to_string().contains("animation/sequences/"));
    assert!(artifact_uri.to_string().ends_with(".zasset"));
    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::AnimationSequence(sequence));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_text_artifact_cache_artifacts() {
    let root = unique_temp_project_root("artifact_store_text_reject");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let artifact_uri = AssetUri::parse("lib://materials/stale.json").unwrap();
    let artifact_path = paths.asset_artifact_root().join(artifact_uri.path());
    fs::create_dir_all(artifact_path.parent().unwrap()).unwrap();
    fs::write(&artifact_path, br#"{"Material":{"name":"Stale"}}"#).unwrap();

    let error = ArtifactStore::default()
        .read(&paths, &artifact_uri)
        .unwrap_err();

    assert!(format!("{error:?}").contains("expected .zasset"));

    let _ = fs::remove_dir_all(root);
}
