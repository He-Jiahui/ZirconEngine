use crate::asset::{
    AlphaMode, AssetReference, AssetUri, AssetUuid, MaterialAsset, MaterialTextureSlotValue,
    ShaderAsset, ShaderEntryPointAsset, ShaderMaterialPropertyAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError, RenderShaderDefinitionValue,
};

#[test]
fn material_asset_zmaterial_roundtrip_maps_pbr_fields_to_shader_overrides() {
    let material = MaterialAsset {
        name: Some("Grid".to_string()),
        shader: AssetReference::new(
            AssetUuid::from_stable_label("shader"),
            AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        ),
        base_color: [0.9, 0.8, 0.7, 1.0],
        base_color_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("albedo"),
            AssetUri::parse("res://textures/albedo.png").unwrap(),
        )),
        normal_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("normal"),
            AssetUri::parse("res://textures/normal.png").unwrap(),
        )),
        metallic: 0.3,
        roughness: 0.6,
        metallic_roughness_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("metal_rough"),
            AssetUri::parse("res://textures/metal_rough.png").unwrap(),
        )),
        occlusion_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("occlusion"),
            AssetUri::parse("res://textures/occlusion.png").unwrap(),
        )),
        emissive: [0.1, 0.2, 0.3],
        emissive_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("emissive"),
            AssetUri::parse("res://textures/emissive.png").unwrap(),
        )),
        alpha_mode: AlphaMode::Mask { cutoff: 0.5 },
        double_sided: true,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };

    let document = material.to_toml_string().unwrap();
    let loaded = MaterialAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded.name, material.name);
    assert_eq!(loaded.shader, material.shader);
    assert_eq!(loaded.base_color, material.base_color);
    assert_eq!(loaded.base_color_texture, material.base_color_texture);
    assert_eq!(loaded.normal_texture, material.normal_texture);
    assert_eq!(loaded.metallic, material.metallic);
    assert_eq!(loaded.roughness, material.roughness);
    assert_eq!(
        loaded.metallic_roughness_texture,
        material.metallic_roughness_texture
    );
    assert_eq!(loaded.occlusion_texture, material.occlusion_texture);
    assert_eq!(loaded.emissive, material.emissive);
    assert_eq!(loaded.emissive_texture, material.emissive_texture);
    assert_eq!(loaded.alpha_mode, material.alpha_mode);
    assert_eq!(loaded.double_sided, material.double_sided);
    assert!(loaded.property_overrides().contains_key("base_color"));
    assert!(loaded.property_overrides().contains_key("roughness"));
    assert!(loaded.texture_slots.contains_key("base_color"));
    assert!(loaded.texture_slots.contains_key("normal"));
}

#[test]
fn material_asset_parses_uuid_url_references() {
    let document = r#"
version = 1
name = "Grid"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.wgsl"

[overrides]
base_color = [0.9, 0.8, 0.7, 1.0]
metallic = 0.3
roughness = 0.6
emissive = [0.1, 0.2, 0.3]
double_sided = true

[overrides.alpha_mode]
mode = "opaque"

[textures.base_color]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://textures/albedo.png"

[textures.normal]
fallback = "normal"
"#;

    let loaded = MaterialAsset::from_toml_str(document).unwrap();

    assert_eq!(
        loaded.shader.locator,
        AssetUri::parse("res://shaders/pbr.wgsl").unwrap()
    );
    assert_eq!(
        loaded.base_color_texture.as_ref().unwrap().locator,
        AssetUri::parse("res://textures/albedo.png").unwrap()
    );
    assert_eq!(loaded.base_color, [0.9, 0.8, 0.7, 1.0]);
    assert!(loaded.double_sided);
    assert_eq!(
        loaded.texture_slots["normal"].fallback.as_deref(),
        Some("normal")
    );
    assert!(loaded.texture_slots["normal"].reference.is_none());
}

#[test]
fn material_asset_serialization_rewrites_stale_canonical_overrides() {
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Grid"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.wgsl"

[overrides]
base_color = [0.8, 0.8, 0.8, 1.0]
metallic = 0.1
roughness = 0.8
emissive = [0.4, 0.3, 0.2]
double_sided = true

[overrides.alpha_mode]
mode = "mask"
cutoff = 0.5

[textures.base_color]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://textures/old.png"
fallback = "white"
"#,
    )
    .unwrap();

    material.base_color = [0.2, 0.7, 0.9, 1.0];
    material.metallic = 0.6;
    material.roughness = 0.25;
    material.emissive = [0.0, 0.1, 0.2];
    material.alpha_mode = AlphaMode::Opaque;
    material.double_sided = false;
    material.base_color_texture = Some(AssetReference::new(
        AssetUuid::from_stable_label("new-base-color"),
        AssetUri::parse("res://textures/new.png").unwrap(),
    ));
    material
        .property_values
        .insert("custom_gain".to_string(), toml::Value::Float(2.0));

    let encoded = material.to_toml_string().unwrap();
    let loaded = MaterialAsset::from_toml_str(&encoded).unwrap();

    assert_eq!(loaded.base_color, [0.2, 0.7, 0.9, 1.0]);
    assert_eq!(loaded.metallic, 0.6);
    assert_eq!(loaded.roughness, 0.25);
    assert_eq!(loaded.emissive, [0.0, 0.1, 0.2]);
    assert_eq!(loaded.alpha_mode, AlphaMode::Opaque);
    assert!(!loaded.double_sided);
    assert_eq!(
        loaded.base_color_texture.as_ref().unwrap().locator,
        AssetUri::parse("res://textures/new.png").unwrap()
    );
    assert_eq!(
        loaded.texture_slots["base_color"].fallback.as_deref(),
        Some("white")
    );
    assert_eq!(
        loaded.property_values.get("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );
    assert!(!loaded.property_values.contains_key("alpha_mode"));
    assert!(!loaded.property_values.contains_key("double_sided"));
}

#[test]
fn material_asset_rejects_legacy_material_toml_shape() {
    let document = r#"
name = "Grid"
base_color = [0.9, 0.8, 0.7, 1.0]

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.wgsl"
"#;

    let error = MaterialAsset::from_toml_str(document).unwrap_err();

    assert!(
        error.to_string().contains("unknown field `base_color`"),
        "unexpected error: {error}"
    );
}

#[test]
fn material_asset_reports_shader_contract_diagnostics_without_blocking_import() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Mismatch"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/mismatch.zshader"

[overrides]
base_color = true
unknown_scalar = 3.0

[textures.base_color]
fallback = "white"

[textures.unknown_slot]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://textures/extra.png"
"#,
    )
    .unwrap();
    let shader = shader_contract();

    let diagnostics = material.shader_contract_diagnostics(&shader);
    let report = material.readiness_report_with_shader_contract(&shader, |_| true, |_| true);

    assert!(diagnostics.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnknownPropertyOverride { source, path, name }
            if *source == RenderMaterialDiagnosticSource::MaterialOverride
                && path == "overrides.unknown_scalar"
                && name == "unknown_scalar"
    )));
    assert!(diagnostics.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::ShaderSchema
            && path == "overrides.base_color"
            && name == "base_color"
            && expected == "vec4"
    )));
    assert!(diagnostics.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnknownTextureSlot { source, path, slot }
            if *source == RenderMaterialDiagnosticSource::TextureSlot
                && path == "textures.unknown_slot"
                && slot == "unknown_slot"
    )));
    assert!(diagnostics.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::MissingRequiredProperty { source, path, name }
            if *source == RenderMaterialDiagnosticSource::ShaderSchema
                && path == "overrides.emissive_power"
                && name == "emissive_power"
    )));
    assert!(!diagnostics.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedTextureReference { slot, .. }
            if slot == "base_color"
    )));
    assert!(!report.is_ready());
    assert_eq!(report.validation_errors.len(), diagnostics.len());
    assert!(report.fallback_usages.is_empty());
}

#[test]
fn material_asset_reports_missing_required_shader_texture_slot() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "MissingTexture"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/missing_texture.zshader"

[textures.base_color]
fallback = "white"
"#,
    )
    .unwrap();
    let mut shader = shader_contract();
    shader.property_schema.clear();
    shader.texture_slots[0].required = true;

    let diagnostics = material.shader_contract_diagnostics(&shader);
    let report = material.readiness_report_with_shader_contract(&shader, |_| true, |_| true);

    assert!(diagnostics.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::MissingRequiredTextureSlot { source, path, slot }
            if *source == RenderMaterialDiagnosticSource::ShaderSchema
                && path == "textures.base_color"
                && slot == "base_color"
    )));
    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::MissingRequiredTextureSlot { slot, .. }
            if slot == "base_color"
    )));
    assert_eq!(
        material.shader_property_diagnostics(&shader),
        vec!["material texture slot base_color requires a concrete texture reference"]
    );
}

#[test]
fn material_asset_readiness_includes_shader_payload_readiness_diagnostics() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "ShaderReadiness"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/readiness.zshader"
"#,
    )
    .unwrap();
    let mut shader = shader_contract();
    shader.property_schema.clear();
    shader.texture_slots.clear();
    shader.entry_points = vec![ShaderEntryPointAsset {
        name: "fs_main".to_string(),
        stage: "pixel".to_string(),
    }];
    shader.shader_defs = vec![
        RenderShaderDefinitionValue::from("USE_UNLIT"),
        RenderShaderDefinitionValue::from(" USE_UNLIT "),
    ];

    let report = material.readiness_report_with_shader_contract(&shader, |_| true, |_| true);

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::ShaderReadiness
            && path == "entry_points.fs_main"
            && diagnostic.contains("unsupported stage `pixel`")
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::ShaderReadiness
            && path == "shader_defs.USE_UNLIT"
            && diagnostic.contains("duplicated")
    )));
}

#[test]
fn material_asset_readiness_reports_material_local_diagnostics_without_blocking() {
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "ImportedMaterial"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/imported.zshader"
"#,
    )
    .unwrap();
    material
        .validation_diagnostics
        .push("glTF Material0 imported with generated defaults".to_string());

    let report = material.readiness_report();

    assert!(report.is_ready());
    assert!(report.has_diagnostics());
    assert!(report.validation_errors.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].source,
        RenderMaterialDiagnosticSource::MaterialAsset
    );
    assert_eq!(
        report.diagnostics[0].path,
        "material.validation_diagnostics[0]"
    );
    assert_eq!(
        report.diagnostics[0].diagnostic,
        "glTF Material0 imported with generated defaults"
    );
}

#[test]
fn shader_declared_texture_slot_overrides_standard_material_bridge() {
    let legacy = AssetReference::new(
        AssetUuid::from_stable_label("legacy-base-color"),
        AssetUri::parse("res://textures/legacy.png").unwrap(),
    );
    let shader_driven = AssetReference::new(
        AssetUuid::from_stable_label("shader-albedo"),
        AssetUri::parse("res://textures/albedo.png").unwrap(),
    );
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 1

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/custom.zshader"
"#,
    )
    .unwrap();
    material.base_color_texture = Some(legacy.clone());
    material.texture_slots.insert(
        "albedo".to_string(),
        MaterialTextureSlotValue::new(shader_driven.clone()),
    );
    let mut shader = shader_contract();
    shader.texture_slots = vec![ShaderTextureSlotAsset {
        name: "albedo".to_string(),
        kind: "texture2d".to_string(),
        required: false,
        default: None,
        sampler: None,
        group: None,
        label: None,
        editor: Default::default(),
    }];

    let legacy_descriptor = material.standard_material_descriptor();
    let shader_descriptor = material.standard_material_descriptor_for_shader(&shader);

    assert_eq!(legacy_descriptor.base_color_texture, Some(legacy));
    assert_eq!(shader_descriptor.base_color_texture, Some(shader_driven));
}

fn shader_contract() -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/mismatch.zshader").unwrap(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(1.0); }".to_string(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: vec![
            ShaderMaterialPropertyAsset {
                name: "base_color".to_string(),
                kind: "vec4".to_string(),
                required: true,
                default: None,
                editor: Default::default(),
            },
            ShaderMaterialPropertyAsset {
                name: "emissive_power".to_string(),
                kind: "float".to_string(),
                required: true,
                default: None,
                editor: Default::default(),
            },
        ],
        texture_slots: vec![ShaderTextureSlotAsset {
            name: "base_color".to_string(),
            kind: "texture2d".to_string(),
            required: false,
            default: Some("white".to_string()),
            sampler: Some("linear_repeat".to_string()),
            group: Some("Surface".to_string()),
            label: Some("Base Color".to_string()),
            editor: Default::default(),
        }],
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}
