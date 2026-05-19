use std::collections::{BTreeMap, BTreeSet};

use crate::ui::material_editor::MaterialEditorProjection;
use zircon_runtime::asset::{
    AssetUri, MaterialAsset, ShaderAsset, ShaderMaterialPropertyAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset,
};
use zircon_runtime::core::framework::render::RenderMaterialDiagnosticSource;

#[test]
fn material_editor_projection_groups_shader_properties_and_material_overrides() {
    let material = material_asset();
    let shader = shader_asset();

    let projection = MaterialEditorProjection::from_material(&material, Some(&shader));

    assert_eq!(
        projection.material_name.as_deref(),
        Some("Preview Material")
    );
    assert_eq!(projection.properties.len(), 3);

    let base_color = projection
        .properties
        .iter()
        .find(|row| row.name == "base_color")
        .expect("base_color property row");
    assert_eq!(base_color.kind.as_deref(), Some("vec4"));
    assert_eq!(base_color.group.as_deref(), Some("Surface"));
    assert_eq!(base_color.label.as_deref(), Some("Base Color"));
    assert!(base_color.is_overridden);
    assert_eq!(
        base_color.override_value.as_ref(),
        material.property_overrides().get("base_color")
    );

    let roughness = projection
        .properties
        .iter()
        .find(|row| row.name == "roughness")
        .expect("roughness property row");
    assert_eq!(roughness.kind.as_deref(), Some("float"));
    assert!(!roughness.is_overridden);
    assert_eq!(roughness.default_value, Some(toml::Value::Float(0.5)));

    let unknown = projection
        .properties
        .iter()
        .find(|row| row.name == "unknown_scalar")
        .expect("unknown material override row");
    assert_eq!(unknown.kind, None);
    assert!(unknown.is_overridden);
}

#[test]
fn material_editor_projection_surfaces_texture_slots_and_diagnostics() {
    let material = material_asset();
    let shader = shader_asset();

    let projection = MaterialEditorProjection::from_material(&material, Some(&shader));

    assert_eq!(projection.texture_slots.len(), 3);

    let albedo = projection
        .texture_slots
        .iter()
        .find(|row| row.name == "albedo")
        .expect("albedo texture slot row");
    assert_eq!(albedo.kind.as_deref(), Some("texture2d"));
    assert_eq!(albedo.group.as_deref(), Some("Surface"));
    assert_eq!(albedo.default_fallback.as_deref(), Some("white"));
    assert_eq!(albedo.fallback.as_deref(), Some("white"));
    assert!(albedo.reference.is_none());
    assert!(albedo.is_overridden);

    let normal = projection
        .texture_slots
        .iter()
        .find(|row| row.name == "normal")
        .expect("normal texture slot row");
    assert!(!normal.is_overridden);
    assert_eq!(normal.default_fallback.as_deref(), Some("normal"));

    let unknown = projection
        .texture_slots
        .iter()
        .find(|row| row.name == "unknown_slot")
        .expect("unknown texture slot row");
    assert_eq!(unknown.kind, None);
    assert!(unknown.reference.is_some());

    assert!(projection.diagnostics.iter().any(|row| {
        row.source == Some(RenderMaterialDiagnosticSource::MaterialOverride)
            && row.path == "overrides.unknown_scalar"
            && row.message.contains("not declared")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source == Some(RenderMaterialDiagnosticSource::TextureSlot)
            && row.path == "textures.unknown_slot"
            && row.message.contains("not declared")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source == Some(RenderMaterialDiagnosticSource::WgslCapture)
            && row.path == "shader.validation_diagnostics"
            && row.message.contains("base_color")
    }));
}

#[test]
fn material_editor_projection_can_open_without_loaded_shader_contract() {
    let material = material_asset();

    let projection = MaterialEditorProjection::from_material(&material, None);

    assert_eq!(projection.properties.len(), 2);
    assert_eq!(projection.texture_slots.len(), 2);
    assert!(projection
        .properties
        .iter()
        .all(|row| row.kind.is_none() && row.is_overridden));
    assert_eq!(
        projection
            .texture_slots
            .iter()
            .map(|row| row.name.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["albedo", "unknown_slot"])
    );
    assert!(projection.diagnostics.is_empty());
}

#[test]
fn material_editor_projection_preserves_material_and_generic_shader_diagnostics() {
    let mut material = material_asset();
    material
        .validation_diagnostics
        .push("material importer note".to_string());
    let mut shader = shader_asset();
    shader
        .validation_diagnostics
        .push("wgsl validation failed before entry point inference".to_string());

    let projection = MaterialEditorProjection::from_material(&material, Some(&shader));

    assert!(projection.diagnostics.iter().any(|row| {
        row.source.is_none()
            && row.path == "material.validation_diagnostics"
            && row.message == "material importer note"
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source.is_none()
            && row.path == "shader.validation_diagnostics"
            && row.message == "wgsl validation failed before entry point inference"
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source == Some(RenderMaterialDiagnosticSource::WgslCapture)
            && row.path == "shader.validation_diagnostics"
            && row.message.contains("base_color")
    }));
}

fn material_asset() -> MaterialAsset {
    MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Preview Material"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
base_color = [0.8, 0.7, 0.6, 1.0]
unknown_scalar = 3.0

[textures.albedo]
fallback = "white"

[textures.unknown_slot]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://textures/extra.png"
"#,
    )
    .unwrap()
}

fn shader_asset() -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/pbr.zshader").unwrap(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: String::new(),
        wgsl_source: String::new(),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        property_schema: vec![
            ShaderMaterialPropertyAsset {
                name: "base_color".to_string(),
                kind: "vec4".to_string(),
                required: true,
                default: Some(toml::Value::Array(vec![
                    toml::Value::Float(1.0),
                    toml::Value::Float(1.0),
                    toml::Value::Float(1.0),
                    toml::Value::Float(1.0),
                ])),
                editor: editor_hints("Surface", "Base Color"),
            },
            ShaderMaterialPropertyAsset {
                name: "roughness".to_string(),
                kind: "float".to_string(),
                required: false,
                default: Some(toml::Value::Float(0.5)),
                editor: editor_hints("Surface", "Roughness"),
            },
        ],
        texture_slots: vec![
            ShaderTextureSlotAsset {
                name: "albedo".to_string(),
                kind: "texture2d".to_string(),
                default: Some("white".to_string()),
                sampler: Some("linear_repeat".to_string()),
                group: Some("Surface".to_string()),
                label: Some("Albedo".to_string()),
                editor: BTreeMap::new(),
            },
            ShaderTextureSlotAsset {
                name: "normal".to_string(),
                kind: "texture2d".to_string(),
                default: Some("normal".to_string()),
                sampler: Some("linear_repeat".to_string()),
                group: Some("Surface".to_string()),
                label: Some("Normal".to_string()),
                editor: BTreeMap::new(),
            },
        ],
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: vec![
            "wgsl_capture property `base_color` was not found at properties.base_color".to_string(),
        ],
    }
}

fn editor_hints(group: &str, label: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("group".to_string(), group.to_string()),
        ("label".to_string(), label.to_string()),
    ])
}

#[test]
fn material_editor_projection_maps_runtime_validation_errors_to_rows() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
[overrides]
alpha_mode = { mode = "mask", cutoff = 2.0 }
"#,
    )
    .unwrap();
    let projection = MaterialEditorProjection::from_material(&material, None);

    assert!(projection.diagnostics.iter().any(|row| {
        row.source.is_none()
            && row.path == "overrides.alpha_mode.cutoff"
            && row.message.contains("0.0..=1.0")
    }));
}
