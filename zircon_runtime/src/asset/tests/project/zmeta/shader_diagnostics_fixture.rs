use super::*;

#[test]
fn project_manager_imports_zshader_with_wgsl_capture_diagnostics() {
    let root = unique_temp_project_root("project_manager_zshader_capture_diagnostics");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "ShaderCaptureSandbox",
        AssetUri::parse("res://shaders/capture_shader").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_uri = AssetUri::parse("res://shaders/capture_shader").unwrap();
    let shader_meta_path = paths
        .assets_root()
        .join("shaders")
        .join("capture_shader.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths.assets_root().join("shaders").join("capture_shader");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("capture.zshader"),
        r#"
version = 1
wgsl_files = ["capture.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"

[[properties]]
name = "base_color"
kind = "vec4"

[[texture_slots]]
name = "albedo"
kind = "texture2d"
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("capture.wgsl"),
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    return vec4f(f32(vertex_index), 0.0, 0.0, 1.0);
}
"#,
    )
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    match manager.load_artifact(&shader_uri).unwrap() {
        ImportedAsset::Shader(shader) => {
            assert!(shader
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic
                    .contains("wgsl_capture property `base_color` was not found")));
            assert!(shader
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic
                    .contains("wgsl_capture texture slot `albedo` was not found")));
            let readiness = shader.readiness_report();
            assert!(!readiness.is_ready());
            assert!(readiness.uses_runtime_wgsl());
            assert_eq!(
                readiness.validation_diagnostics,
                shader.validation_diagnostics
            );
            assert!(readiness
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("wgsl_capture property `base_color`")));
            assert!(readiness
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("wgsl_capture texture slot `albedo`")));
        }
        other => panic!("unexpected compound shader artifact: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn zshader_typed_shader_definition_rows_validate_kind_and_value() {
    let document = ZShaderDocument::from_toml_str(
        r#"
version = 1
shader_defs = ["LEGACY_FLAG"]

[[shader_def_values]]
name = "ENABLE_FOG"
kind = "bool"
value = false

[[shader_def_values]]
name = "BINDING_INDEX"
kind = "uint"
value = 4

[[shader_def_values]]
name = "DEBUG_MODE"
kind = "int"
value = -1
"#,
    )
    .unwrap();

    assert_eq!(
        document.shader_definition_values().unwrap(),
        vec![
            RenderShaderDefinitionValue::from("LEGACY_FLAG"),
            RenderShaderDefinitionValue::bool("ENABLE_FOG", false),
            RenderShaderDefinitionValue::uint("BINDING_INDEX", 4),
            RenderShaderDefinitionValue::int("DEBUG_MODE", -1),
        ]
    );

    let unknown_kind = ZShaderDocument::from_toml_str(
        r#"
[[shader_def_values]]
name = "BAD_KIND"
kind = "float"
value = 1.0
"#,
    )
    .unwrap();
    assert!(unknown_kind
        .shader_definition_values()
        .unwrap_err()
        .contains("unsupported kind `float`"));

    let non_bool = ZShaderDocument::from_toml_str(
        r#"
[[shader_def_values]]
name = "ENABLE_FOG"
kind = "bool"
value = 1
"#,
    )
    .unwrap();
    assert!(non_bool
        .shader_definition_values()
        .unwrap_err()
        .contains("value is not a boolean"));

    let negative_uint = ZShaderDocument::from_toml_str(
        r#"
[[shader_def_values]]
name = "BINDING_INDEX"
kind = "uint"
value = -1
"#,
    )
    .unwrap();
    assert!(negative_uint
        .shader_definition_values()
        .unwrap_err()
        .contains("value is not a u32 integer"));
}

#[test]
fn documented_zmeta_shader_material_fixture_parses() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have repo parent")
        .join("docs")
        .join("assets-and-rendering")
        .join("fixtures")
        .join("zmeta-shader-material")
        .join("assets");

    let shader_meta =
        AssetMetaDocument::load(fixture_root.join("shaders").join("unlit_shader.zmeta")).unwrap();
    let shader_uri = AssetUri::parse("res://shaders/unlit_shader").unwrap();
    let zshader_uri = AssetUri::parse("res://shaders/unlit_shader#zshader:unlit.zshader").unwrap();
    let wgsl_uri = AssetUri::parse("res://shaders/unlit_shader#wgsl:unlit.wgsl").unwrap();

    assert_eq!(shader_meta.url, shader_uri);
    assert_eq!(shader_meta.asset_kind, AssetKind::Shader);
    assert_eq!(shader_meta.unit, AssetSourceUnit::Compound);
    assert_eq!(shader_meta.entries.len(), 3);
    assert!(shader_meta
        .entries
        .iter()
        .any(|entry| entry.url == zshader_uri && entry.asset_kind == AssetKind::Data));
    assert!(shader_meta
        .entries
        .iter()
        .any(|entry| entry.url == wgsl_uri && entry.asset_kind == AssetKind::Data));

    let zshader = ZShaderDocument::from_toml_str(
        &fs::read_to_string(
            fixture_root
                .join("shaders")
                .join("unlit_shader")
                .join("unlit.zshader"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(zshader.wgsl_files, vec!["unlit.wgsl"]);
    assert_eq!(zshader.import_path.as_deref(), Some("zircon::unlit"));
    assert_eq!(zshader.shader_defs, vec!["USE_UNLIT"]);
    assert_eq!(
        zshader.shader_definition_values().unwrap(),
        vec![RenderShaderDefinitionValue::from("USE_UNLIT")]
    );
    assert_eq!(zshader.entry_points.len(), 2);
    assert_eq!(zshader.properties[0].name, "base_color");
    let fixture_wgsl = fs::read_to_string(
        fixture_root
            .join("shaders")
            .join("unlit_shader")
            .join("unlit.wgsl"),
    )
    .unwrap();
    let shader_asset = ShaderAsset {
        uri: shader_uri.clone(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: fixture_wgsl.clone(),
        wgsl_source: fixture_wgsl,
        import_path: zshader.import_path.clone(),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: vec![ShaderSourceFileAsset {
            path: "unlit.wgsl".to_string(),
            url: AssetUri::parse("res://shaders/unlit_shader/unlit.wgsl").unwrap(),
        }],
        imports: Vec::new(),
        shader_defs: zshader.shader_definition_values().unwrap(),
        property_schema: zshader.properties.clone(),
        texture_slots: zshader
            .texture_slots
            .iter()
            .map(ShaderTextureSlotAsset::from)
            .collect(),
        editor: zshader.editor.clone(),
        pipeline_layout: zshader.pipeline_layout.clone(),
        validation_diagnostics: Vec::new(),
    };
    assert!(
        crate::asset::validate_wgsl_captures(&shader_asset).is_empty(),
        "documented fixture WGSL should reference every declared shader property and texture slot"
    );

    let material = MaterialAsset::from_toml_str(
        &fs::read_to_string(fixture_root.join("materials").join("hero_unlit.zmaterial")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        material.shader.uuid,
        "11111111-2222-4333-8444-555555555555".parse().unwrap()
    );
    assert_eq!(material.shader.locator, shader_uri);
    assert!(material.property_values.contains_key("base_color"));
    assert!(material.texture_slots.contains_key("base_color"));

    let material_meta = AssetMetaDocument::load(
        fixture_root
            .join("materials")
            .join("hero_unlit.zmaterial.zmeta"),
    )
    .unwrap();
    assert_eq!(material_meta.asset_kind, AssetKind::Material);
    assert_eq!(material_meta.dependencies.len(), 2);
    assert!(material_meta.dependencies.contains(&shader_uri));
    assert!(material_meta
        .dependencies
        .contains(&AssetUri::parse("res://textures/hero_albedo.png").unwrap()));
}
