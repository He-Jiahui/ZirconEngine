use std::collections::BTreeMap;

use super::{project_authored_material_document, MaterialDocumentProjectionError};
use zircon_editor::ui::material_editor::MaterialEditorProjection;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialAsset, ShaderAsset, ShaderMaterialPropertyAsset,
    ShaderSourceLanguage, ShaderTextureSlotAsset, ZMaterialDocument,
};
use zircon_runtime::core::framework::render::{
    MaterialPropertyKind, RenderMaterialDiagnosticSource, ShaderAssetKind,
};
use zircon_runtime_interface::project::PersistedAssetReference;

#[test]
fn material_graph_instance_projection_does_not_mark_inherited_values_as_local_overrides() {
    let mut child = document("", "");
    child.parent = Some(reference("builtin://material/base"));
    let before = child.clone();
    let parent = MaterialAsset::from_zmaterial_document(document(
        "roughness = 0.9",
        "base_color = { kind = 'builtin', locator = 'builtin://texture/white' }",
    ));
    let mut effective = MaterialAsset::from_zmaterial_document(child.clone());
    effective.inherit_parent_values_from(&parent);
    assert_eq!(effective.roughness, 0.9);
    assert!(effective.base_color_texture.is_some());
    assert!(effective.texture_slots.contains_key("base_color"));

    let schema = shader();
    let flattened = MaterialEditorProjection::from_material(&effective, Some(&schema));
    assert!(flattened.properties[1].is_overridden);
    assert!(flattened.texture_slots[0].is_overridden);

    let authored = project_authored_material_document(&child, Some(&schema)).unwrap();
    assert!(!authored.properties[1].is_overridden);
    assert!(authored.properties[1].override_value.is_none());
    assert!(!authored.texture_slots[0].is_overridden);
    assert!(authored.texture_slots[0].reference.is_none());
    // The shader default is not asserted to be the inherited effective 0.9.
    assert_eq!(
        authored.properties[1]
            .default_value
            .as_ref()
            .unwrap()
            .as_float(),
        Some(0.5)
    );
    assert_eq!(child, before);
}

#[test]
fn material_graph_instance_projection_preserves_default_override_and_empty_texture_presence() {
    let source = document("roughness = 0.5", "base_color = {}");
    let before = source.clone();
    let projected = project_authored_material_document(&source, Some(&shader())).unwrap();
    let roughness = &projected.properties[1];
    assert!(roughness.is_overridden);
    assert_eq!(roughness.override_value, roughness.default_value);
    let cleared = &projected.texture_slots[0];
    assert!(cleared.is_overridden);
    assert!(cleared.reference.is_none());
    assert!(cleared.fallback.is_none());
    assert_eq!(cleared.default_fallback.as_deref(), Some("white"));
    assert!(!projected.texture_slots[1].is_overridden);
    assert_eq!(source, before);
}

#[test]
fn material_graph_instance_projection_keeps_schema_order_groups_exact_keys_and_unknown_rows() {
    let source = document(
        "roughness = 0.75\n' roughness ' = 0.4\n'未知 参数' = 3",
        "extra = { fallback = 'black', uv_channel = 3 }",
    );
    let before = source.clone();
    let schema = shader();
    let schema_before = schema.clone();
    let projected = project_authored_material_document(&source, Some(&schema)).unwrap();
    assert_eq!(
        projected
            .properties
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>(),
        ["color", "roughness", " roughness ", "未知 参数"]
    );
    assert_eq!(
        projected.properties[1].group.as_deref(),
        Some("表面参数 / Surface")
    );
    assert_eq!(
        projected.properties[1].label.as_deref(),
        Some("粗糙度：保留作者显示名")
    );
    assert!(projected.properties[2].kind.is_none());
    assert!(projected.properties[2].default_value.is_none());
    assert!(projected.properties[2].is_overridden);
    assert_eq!(projected.texture_slots[2].name, "extra");
    assert!(projected.texture_slots[2].kind.is_none());
    assert_eq!(source.textures["extra"].uv_channel, 3);
    assert_eq!(source, before);
    assert_eq!(schema, schema_before);
}

#[test]
fn material_graph_instance_projection_without_shader_does_not_invent_schema_or_values() {
    let source = document("roughness = 0.25", "base_color = {}");
    let projected = project_authored_material_document(&source, None).unwrap();
    assert_eq!(projected.shader_reference, source.shader);
    assert_eq!(projected.properties.len(), 1);
    assert!(projected.properties[0].kind.is_none());
    assert!(projected.properties[0].group.is_none());
    assert!(projected.properties[0].default_value.is_none());
    assert_eq!(
        projected.properties[0].override_value.as_ref(),
        source.overrides.get("roughness")
    );
    assert_eq!(projected.texture_slots.len(), 1);
    assert!(projected.texture_slots[0].is_overridden);
    assert!(projected.diagnostics.iter().any(|diagnostic| {
        diagnostic.path == "shader"
            && diagnostic.source == Some(RenderMaterialDiagnosticSource::DependencyResolution)
    }));
}

#[test]
fn material_graph_instance_projection_rejects_an_unrelated_shader_without_source_mutation() {
    let source = document("roughness = 0.25", "");
    let before = source.clone();
    let mut wrong = shader();
    wrong.uri = AssetUri::parse("builtin://shader/unrelated").unwrap();
    let wrong_before = wrong.clone();
    assert_eq!(
        project_authored_material_document(&source, Some(&wrong)),
        Err(MaterialDocumentProjectionError::ShaderLocatorMismatch {
            expected: source.shader.clone(),
            actual: wrong.uri.clone(),
        })
    );
    assert_eq!(source, before);
    assert_eq!(wrong, wrong_before);
}

#[test]
fn material_graph_instance_projection_rejects_ambiguous_schema_keys_in_their_own_namespace() {
    let source = document("roughness = 0.25", "");
    let mut duplicate_property = shader();
    duplicate_property
        .property_schema
        .push(duplicate_property.property_schema[1].clone());
    assert_eq!(
        project_authored_material_document(&source, Some(&duplicate_property)),
        Err(MaterialDocumentProjectionError::DuplicatePropertyName(
            "roughness".into()
        ))
    );
    let mut duplicate_texture = shader();
    duplicate_texture
        .texture_slots
        .push(duplicate_texture.texture_slots[0].clone());
    assert_eq!(
        project_authored_material_document(&source, Some(&duplicate_texture)),
        Err(MaterialDocumentProjectionError::DuplicateTextureSlotName(
            "base_color".into()
        ))
    );
    let mut distinct_namespaces = shader();
    distinct_namespaces.texture_slots[0].name = "roughness".into();
    assert!(project_authored_material_document(&source, Some(&distinct_namespaces)).is_ok());
}

#[test]
fn material_graph_instance_projection_preserves_author_codec_metadata_and_repair_diagnostics() {
    let mut source = document("roughness = inf\ncolor = 'wrong-kind'", "base_color = {}");
    source.parent = Some(reference("builtin://material/base"));
    source.validation_diagnostics = vec!["原作者诊断，未由投影静默清除".into()];
    let slot = source.textures.get_mut("base_color").unwrap();
    slot.uv_channel = 2;
    slot.transform = Some(Default::default());
    let before = source.clone();
    let mut schema = shader();
    schema.validation_diagnostics = vec![
        "wgsl_capture 原着色器诊断".into(),
        "other shader import diagnostic".into(),
    ];
    let projected = project_authored_material_document(&source, Some(&schema)).unwrap();
    assert_eq!(
        projected.properties[0]
            .override_value
            .as_ref()
            .unwrap()
            .as_str(),
        Some("wrong-kind")
    );
    assert!(projected.properties[1]
        .override_value
        .as_ref()
        .unwrap()
        .as_float()
        .unwrap()
        .is_infinite());
    assert_eq!(projected.diagnostics.len(), 3);
    assert_eq!(
        projected.diagnostics[0].message,
        source.validation_diagnostics[0]
    );
    assert_eq!(
        projected.diagnostics[1].source,
        Some(RenderMaterialDiagnosticSource::WgslCapture)
    );
    assert_eq!(
        projected.diagnostics[2].message,
        schema.validation_diagnostics[1]
    );
    assert_eq!(source, before);

    let text = source
        .to_project_toml_string(|value| Ok(PersistedAssetReference::builtin(value.locator.clone())))
        .unwrap();
    let reopened = ZMaterialDocument::from_project_toml_str(&text, |value| {
        Ok(AssetReference::from_locator(
            value.builtin_locator().unwrap().clone(),
        ))
    })
    .unwrap();
    assert_eq!(reopened, source);
    assert_eq!(
        project_authored_material_document(&reopened, Some(&schema)).unwrap(),
        projected
    );
}

// Unit inputs for the real authoring codec/projection, never a preview producer,
// published ResourceManager handle or substitute for an Engine-rendered asset.
fn document(overrides: &str, textures: &str) -> ZMaterialDocument {
    let text = format!(
        r#"version = 2
name = "实例作者参数"
[shader]
kind = "builtin"
locator = "builtin://shader/pbr"
[overrides]
{overrides}
[textures]
{textures}
[options]
quality = 2
[editor]
note = "保留中文和长作者元数据"
expanded = ["表面参数 / Surface"]
"#
    );
    ZMaterialDocument::from_project_toml_str(&text, |value| {
        Ok(AssetReference::from_locator(
            value.builtin_locator().unwrap().clone(),
        ))
    })
    .unwrap()
}

fn reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}

fn shader() -> ShaderAsset {
    let defaults = document("roughness = 0.5\ncolor = [1.0, 1.0, 1.0, 1.0]", "");
    ShaderAsset {
        uri: AssetUri::parse("builtin://shader/pbr").unwrap(),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: String::new(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: [
            ("color", MaterialPropertyKind::Vec4, "颜色"),
            (
                "roughness",
                MaterialPropertyKind::Float,
                "粗糙度：保留作者显示名",
            ),
        ]
        .into_iter()
        .map(|(name, kind, label)| ShaderMaterialPropertyAsset {
            name: name.into(),
            kind,
            required: false,
            default: defaults.overrides.get(name).cloned(),
            editor: BTreeMap::from([
                ("group".into(), "表面参数 / Surface".into()),
                ("label".into(), label.into()),
            ]),
        })
        .collect(),
        options: Vec::new(),
        texture_slots: ["base_color", "normal"]
            .into_iter()
            .map(|name| ShaderTextureSlotAsset {
                name: name.into(),
                kind: "texture2d".into(),
                required: false,
                default: Some(
                    if name == "base_color" {
                        "white"
                    } else {
                        "normal"
                    }
                    .into(),
                ),
                sampler: None,
                group: Some("纹理".into()),
                label: Some(name.into()),
                option: None,
                st: false,
                editor: BTreeMap::new(),
            })
            .collect(),
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}
