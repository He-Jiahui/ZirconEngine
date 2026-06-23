use super::*;

#[test]
fn project_manager_imports_compound_zshader_package_with_subassets() {
    let root = unique_temp_project_root("project_manager_compound_zshader");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "ShaderPackageSandbox",
        AssetUri::parse("res://shaders/unlit_shader").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_uri = AssetUri::parse("res://shaders/unlit_shader").unwrap();
    let shader_meta_path = paths
        .assets_root()
        .join("shaders")
        .join("unlit_shader.zmeta");
    let mut shader_meta =
        AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
    shader_meta.unit = AssetSourceUnit::Compound;
    shader_meta.save(&shader_meta_path).unwrap();

    let shader_dir = paths.assets_root().join("shaders").join("unlit_shader");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(
        shader_dir.join("unlit.zshader"),
        r#"
version = 1
import_path = "zircon::unlit"
wgsl_files = ["unlit.wgsl"]
shader_defs = ["USE_UNLIT", "ALPHA_CLIP"]

[[shader_def_values]]
name = "TONEMAPPING_LUT_TEXTURE_BINDING_INDEX"
kind = "uint"
value = 2

[[shader_def_values]]
name = "ENABLE_FOG"
kind = "bool"
value = false

[[shader_def_values]]
name = "DEBUG_MODE"
kind = "int"
value = -1

[pipeline_layout]
push_constant_ranges = ["draw_index:0..4"]

[[pipeline_layout.bind_groups]]
group = 3
label = "material"

[[pipeline_layout.bind_groups.bindings]]
binding = 0
label = "material_uniforms"
resource_type = "uniform_buffer"
visibility = ["vertex", "fragment"]

[[imports]]
source = "zircon::lighting"
redirect = { uuid = "22222222-2222-4222-8222-222222222222", url = "res://shaders/shared_lighting" }

[[imports]]
source = "naga_oil::math"

[[entry_points]]
name = "vs_main"
stage = "vertex"
file = "unlit.wgsl"

[[entry_points]]
name = "fs_main"
stage = "fragment"
file = "unlit.wgsl"

[[properties]]
name = "base_color"
kind = "vec4"
required = true
default = [1.0, 1.0, 1.0, 1.0]
editor = { label = "Base Color", group = "Surface" }

[[texture_slots]]
name = "base_color"
kind = "texture2d"
default = "white"
sampler = "linear_repeat"
group = "Surface"
label = "Base Color Texture"
"#,
    )
    .unwrap();
    fs::write(
        shader_dir.join("unlit.wgsl"),
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    let base_color = vec4f(1.0, 1.0, 1.0, 1.0);
    return base_color;
}
"#,
    )
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let imported_meta = AssetMetaDocument::load(&shader_meta_path).unwrap();
    let shader_record = manager
        .registry()
        .get_by_locator(&shader_uri)
        .expect("compound shader root record");
    let zshader_subasset = AssetUri::parse("res://shaders/unlit_shader#zshader:unlit.zshader")
        .expect("zshader subasset uri");
    let wgsl_subasset =
        AssetUri::parse("res://shaders/unlit_shader#wgsl:unlit.wgsl").expect("wgsl subasset uri");

    assert_eq!(imported_meta.unit, AssetSourceUnit::Compound);
    assert_eq!(imported_meta.asset_kind, AssetKind::Shader);
    assert!(imported_meta
        .included_files
        .contains(&AssetUri::parse("res://shaders/unlit_shader/unlit.zshader").unwrap()));
    assert!(imported_meta
        .included_files
        .contains(&AssetUri::parse("res://shaders/unlit_shader/unlit.wgsl").unwrap()));
    assert!(imported_meta
        .entries
        .iter()
        .any(|entry| entry.url == shader_uri));
    assert!(imported_meta
        .entries
        .iter()
        .any(|entry| entry.url == zshader_subasset && entry.asset_kind == AssetKind::Data));
    assert!(imported_meta
        .entries
        .iter()
        .any(|entry| entry.url == wgsl_subasset && entry.asset_kind == AssetKind::Data));
    assert_eq!(
        shader_record.id(),
        AssetId::from_asset_uuid(imported_meta.uuid)
    );

    match manager.load_artifact(&shader_uri).unwrap() {
        ImportedAsset::Shader(shader) => {
            assert_eq!(shader.source_files.len(), 1);
            assert_eq!(shader.source_files[0].path, "unlit.wgsl");
            assert_eq!(shader.import_path.as_deref(), Some("zircon::unlit"));
            assert_eq!(shader.imports.len(), 2);
            assert_eq!(shader.imports[0].source, "zircon::lighting");
            assert_eq!(
                shader.imports[0]
                    .redirect
                    .as_ref()
                    .expect("redirected shader import")
                    .locator,
                AssetUri::parse("res://shaders/shared_lighting").unwrap()
            );
            assert_eq!(shader.imports[1].source, "naga_oil::math");
            assert!(shader.imports[1].redirect.is_none());
            assert_eq!(shader.dependencies.len(), 1);
            assert_eq!(
                shader.dependencies()[0].reference.locator,
                AssetUri::parse("res://shaders/shared_lighting").unwrap()
            );
            assert_eq!(shader.entry_points.len(), 2);
            assert_eq!(
                shader.shader_defs,
                vec![
                    RenderShaderDefinitionValue::from("USE_UNLIT"),
                    RenderShaderDefinitionValue::from("ALPHA_CLIP"),
                    RenderShaderDefinitionValue::uint("TONEMAPPING_LUT_TEXTURE_BINDING_INDEX", 2),
                    RenderShaderDefinitionValue::bool("ENABLE_FOG", false),
                    RenderShaderDefinitionValue::int("DEBUG_MODE", -1),
                ]
            );
            assert_eq!(shader.variant_keys()[0].defines, shader.shader_defs);
            assert_eq!(shader.property_schema.len(), 1);
            assert_eq!(shader.property_schema[0].name, "base_color");
            assert_eq!(shader.texture_slots.len(), 1);
            assert_eq!(shader.texture_slots[0].name, "base_color");
            assert_eq!(shader.texture_slots[0].default.as_deref(), Some("white"));
            assert_eq!(
                shader.pipeline_layout,
                RenderShaderPipelineLayoutDescriptor {
                    bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
                        group: 3,
                        label: Some("material".to_string()),
                        bindings: vec![RenderShaderBindingDescriptor {
                            binding: 0,
                            label: Some("material_uniforms".to_string()),
                            resource_type: RenderShaderBindingResourceType::UniformBuffer,
                            visibility: vec![
                                RenderShaderStage::Vertex,
                                RenderShaderStage::Fragment,
                            ],
                        }],
                    }],
                    push_constant_ranges: vec!["draw_index:0..4".to_string()],
                }
            );
            assert!(shader.validation_diagnostics.is_empty());

            let readiness = shader.readiness_report();
            assert!(readiness.is_ready());
            assert!(readiness.uses_runtime_wgsl());
            assert!(readiness.has_pipeline_layout);
            assert!(readiness.has_redirected_import_dependencies());
            assert_eq!(readiness.dependency_count, 1);
            assert_eq!(readiness.imports.len(), 2);
            assert_eq!(readiness.imports[0].source, "zircon::lighting");
            assert!(readiness.imports[0].contributes_dependency);
            assert_eq!(readiness.imports[1].source, "naga_oil::math");
            assert!(!readiness.imports[1].contributes_dependency);
            assert_eq!(readiness.entry_points.len(), 2);
            assert!(readiness
                .entry_points
                .iter()
                .all(|entry| entry.diagnostic.is_none()));
            assert_eq!(readiness.shader_defs.len(), 5);
            assert!(readiness
                .shader_defs
                .iter()
                .all(|definition| definition.diagnostic.is_none()));
            assert_eq!(readiness.shader_defs[2].value.value_as_string(), "2");
            assert_eq!(readiness.shader_defs[3].value.value_as_string(), "false");
            assert_eq!(readiness.shader_defs[4].value.value_as_string(), "-1");
            assert!(readiness.validation_diagnostics.is_empty());

            let mut material = material_for_shader(&shader_uri);
            material.property_values.insert(
                "base_color".to_string(),
                toml::Value::Array(vec![
                    toml::Value::Float(1.0),
                    toml::Value::Float(0.8),
                    toml::Value::Float(0.2),
                    toml::Value::Float(1.0),
                ]),
            );
            assert!(material.shader_property_diagnostics(&shader).is_empty());
            material
                .property_values
                .insert("unknown".to_string(), toml::Value::Boolean(true));
            assert!(material
                .shader_property_diagnostics(&shader)
                .iter()
                .any(|diagnostic| diagnostic.contains("not declared")));
        }
        other => panic!("unexpected compound shader artifact: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}
