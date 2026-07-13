use std::collections::{BTreeMap, BTreeSet};

use crate::ui::material_editor::MaterialEditorProjection;
use zircon_runtime::asset::assets::generate_material_artifact;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialAsset, MaterialTextureSlotValue, ShaderAsset,
    ShaderMaterialPropertyAsset, ShaderSourceLanguage, ShaderTextureSlotAsset, ZMaterialDocument,
};
use zircon_runtime::core::framework::render::{
    MaterialPropertyKind, RenderMaterialDiagnosticSource, ShaderAssetKind,
};

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

#[test]
fn material_editor_projection_maps_missing_required_shader_property() {
    let material = material_from_document(
        Some("Incomplete Material"),
        BTreeMap::new(),
        BTreeMap::new(),
    );
    let shader = shader_asset();

    let projection = MaterialEditorProjection::from_material(&material, Some(&shader));

    assert!(projection.diagnostics.iter().any(|row| {
        row.source == Some(RenderMaterialDiagnosticSource::ShaderSchema)
            && row.path == "overrides.base_color"
            && row.message.contains("base_color")
            && row.message.contains("required")
    }));
}

fn material_asset() -> MaterialAsset {
    let overrides = BTreeMap::from([
        (
            "base_color".to_string(),
            toml::Value::Array(vec![
                toml::Value::Float(0.8),
                toml::Value::Float(0.7),
                toml::Value::Float(0.6),
                toml::Value::Float(1.0),
            ]),
        ),
        ("unknown_scalar".to_string(), toml::Value::Float(3.0)),
    ]);
    let textures = BTreeMap::from([
        (
            "albedo".to_string(),
            MaterialTextureSlotValue {
                reference: None,
                fallback: Some("white".to_string()),
                transform: None,
                uv_channel: 0,
            },
        ),
        (
            "unknown_slot".to_string(),
            MaterialTextureSlotValue::new(material_reference("res://textures/extra.png")),
        ),
    ]);
    material_from_document(Some("Preview Material"), overrides, textures)
}

fn material_from_document(
    name: Option<&str>,
    overrides: BTreeMap<String, toml::Value>,
    textures: BTreeMap<String, MaterialTextureSlotValue>,
) -> MaterialAsset {
    MaterialAsset::from_zmaterial_document(ZMaterialDocument {
        version: 2,
        name: name.map(str::to_string),
        shader: material_reference("res://shaders/pbr.zshader"),
        parent: None,
        options: BTreeMap::new(),
        overrides,
        textures,
        queue: None,
        editor: toml::Table::new(),
        validation_diagnostics: Vec::new(),
    })
}

fn material_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn shader_asset() -> ShaderAsset {
    let property_schema = vec![
        ShaderMaterialPropertyAsset {
            name: "base_color".to_string(),
            kind: MaterialPropertyKind::Vec4,
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
            kind: MaterialPropertyKind::Float,
            required: false,
            default: Some(toml::Value::Float(0.5)),
            editor: editor_hints("Surface", "Roughness"),
        },
    ];
    let options = Vec::new();
    let texture_slots = vec![
        ShaderTextureSlotAsset {
            name: "albedo".to_string(),
            kind: "texture2d".to_string(),
            required: false,
            default: Some("white".to_string()),
            sampler: Some("linear_repeat".to_string()),
            group: Some("Surface".to_string()),
            label: Some("Albedo".to_string()),
            option: None,
            st: false,
            editor: BTreeMap::new(),
        },
        ShaderTextureSlotAsset {
            name: "normal".to_string(),
            kind: "texture2d".to_string(),
            required: false,
            default: Some("normal".to_string()),
            sampler: Some("linear_repeat".to_string()),
            group: Some("Surface".to_string()),
            label: Some("Normal".to_string()),
            option: None,
            st: false,
            editor: BTreeMap::new(),
        },
    ];
    let generated_material = generate_material_artifact(&property_schema, &options, &texture_slots);

    ShaderAsset {
        uri: AssetUri::parse("res://shaders/pbr.zshader").unwrap(),
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
        property_schema,
        options,
        texture_slots,
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: generated_material.property_layout,
        material_option_table: generated_material.option_table,
        generated_material_wgsl: generated_material.wgsl_source,
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
    let mut alpha_mode = toml::Table::new();
    alpha_mode.insert("mode".to_string(), toml::Value::String("mask".to_string()));
    alpha_mode.insert("cutoff".to_string(), toml::Value::Float(2.0));
    let material = material_from_document(
        None,
        BTreeMap::from([
            ("alpha_mode".to_string(), toml::Value::Table(alpha_mode)),
            (
                "lighting_model".to_string(),
                toml::Value::String("toon".to_string()),
            ),
        ]),
        BTreeMap::new(),
    );
    let projection = MaterialEditorProjection::from_material(&material, None);

    assert!(projection.diagnostics.iter().any(|row| {
        row.source.is_none()
            && row.path == "overrides.alpha_mode.cutoff"
            && row.message.contains("0.0..=1.0")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source.is_none()
            && row.path == "overrides.lighting_model"
            && row.message.contains("lighting model `toon`")
    }));
}
