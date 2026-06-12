use std::collections::BTreeMap;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, AssetUuid, MaterialAsset,
    MaterialAssetManagementRecordSet, MaterialTextureSlotValue, ShaderAsset, ShaderEntryPointAsset,
    ShaderMaterialPropertyAsset, ShaderSourceLanguage, ShaderTextureSlotAsset,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialLightingModel, RenderMaterialTextureTransform,
    RenderMaterialValidationError, RenderShaderDefinitionValue,
};
use crate::core::resource::ResourceId;

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
fn material_asset_roundtrip_preserves_standard_texture_transforms() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Tiled Grid"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.wgsl"

[textures.base_color]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://textures/tiled.png"
uv_channel = 1

[textures.base_color.transform]
scale = [2.0, 3.0]
offset = [0.25, 0.5]
"#,
    )
    .unwrap();

    let transform = RenderMaterialTextureTransform {
        scale: [2.0, 3.0],
        offset: [0.25, 0.5],
    };
    assert_eq!(
        material.texture_slots["base_color"].texture_transform(),
        transform
    );
    assert_eq!(material.texture_slots["base_color"].texture_uv_channel(), 1);
    assert_eq!(
        material
            .standard_material_descriptor()
            .base_color_texture_transform,
        transform
    );
    assert_eq!(
        material
            .standard_material_descriptor()
            .base_color_texture_uv_channel,
        1
    );

    let encoded = material.to_toml_string().unwrap();
    let loaded = MaterialAsset::from_toml_str(&encoded).unwrap();

    assert_eq!(
        loaded.texture_slots["base_color"].texture_transform(),
        transform
    );
    assert_eq!(loaded.texture_slots["base_color"].texture_uv_channel(), 1);
    assert_eq!(
        loaded
            .standard_material_descriptor()
            .base_color_texture_transform,
        transform
    );
    assert_eq!(
        loaded
            .standard_material_descriptor()
            .base_color_texture_uv_channel,
        1
    );
}

#[test]
fn material_owned_lighting_model_drives_standard_descriptor_without_shader_override() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Unlit Grid"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
lighting_model = "unlit"
custom_gain = 2.0
"#,
    )
    .unwrap();

    let descriptor = material.standard_material_descriptor();

    assert_eq!(
        material.lighting_model(),
        RenderMaterialLightingModel::Unlit
    );
    assert_eq!(
        descriptor.lighting_model,
        RenderMaterialLightingModel::Unlit
    );
    assert!(descriptor.unlit);
    assert!(material
        .shader_property_override("lighting_model")
        .is_none());
    assert!(material
        .shader_property_overrides()
        .all(|(name, _)| name != "lighting_model"));
    assert_eq!(
        material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );
}

#[test]
fn material_owned_receive_shadows_defaults_on_and_can_opt_out_without_shader_override() {
    let default_material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Default Receiver"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .unwrap();
    let mut no_receive_material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "No Receiver"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
receive_shadows = false
custom_gain = 2.0
"#,
    )
    .unwrap();

    assert!(default_material.receive_shadows());
    assert!(
        default_material
            .standard_material_descriptor()
            .receive_shadows
    );
    assert!(no_receive_material
        .shader_property_override("receive_shadows")
        .is_none());
    assert!(no_receive_material
        .shader_property_overrides()
        .all(|(name, _)| name != "receive_shadows"));
    assert!(!no_receive_material.receive_shadows());
    assert!(
        !no_receive_material
            .standard_material_descriptor()
            .receive_shadows
    );
    assert_eq!(
        no_receive_material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = no_receive_material.to_toml_string().unwrap();
    assert!(encoded.contains("receive_shadows = false"));
    no_receive_material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(true));
    let encoded = no_receive_material.to_toml_string().unwrap();
    assert!(!encoded.contains("receive_shadows"));
}

#[test]
fn material_owned_cast_shadows_defaults_on_and_can_opt_out_without_shader_override() {
    let default_material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Default Caster"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .unwrap();
    let mut no_cast_material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "No Caster"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
cast_shadows = false
custom_gain = 2.0
"#,
    )
    .unwrap();

    assert!(default_material.cast_shadows());
    assert!(default_material.standard_material_descriptor().cast_shadows);
    assert!(no_cast_material
        .shader_property_override("cast_shadows")
        .is_none());
    assert!(no_cast_material
        .shader_property_overrides()
        .all(|(name, _)| name != "cast_shadows"));
    assert!(!no_cast_material.cast_shadows());
    assert!(!no_cast_material.standard_material_descriptor().cast_shadows);
    assert_eq!(
        no_cast_material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = no_cast_material.to_toml_string().unwrap();
    assert!(encoded.contains("cast_shadows = false"));
    no_cast_material
        .property_values
        .insert("cast_shadows".to_string(), toml::Value::Boolean(true));
    let encoded = no_cast_material.to_toml_string().unwrap();
    assert!(!encoded.contains("cast_shadows"));
}

#[test]
fn material_owned_receive_shadows_reports_non_bool_override() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Receiver"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
receive_shadows = "no"
"#,
    )
    .unwrap();

    assert!(material.receive_shadows());
    assert!(material.validation_errors().iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.receive_shadows"
            && name == "receive_shadows"
            && expected == "bool"
    )));
}

#[test]
fn material_owned_cast_shadows_reports_non_bool_override() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Caster"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
cast_shadows = "no"
"#,
    )
    .unwrap();

    assert!(material.cast_shadows());
    assert!(material.validation_errors().iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.cast_shadows"
            && name == "cast_shadows"
            && expected == "bool"
    )));
}

#[test]
fn material_owned_sort_fields_drive_standard_descriptor_without_shader_override() {
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Queue Shifted"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
render_queue = -20
material_queue = 7
depth_bias = -0.25
custom_gain = 2.0
"#,
    )
    .unwrap();

    let descriptor = material.standard_material_descriptor();

    assert_eq!(material.render_queue(), -20);
    assert_eq!(material.material_queue(), 7);
    assert_eq!(material.depth_bias(), -0.25);
    assert_eq!(descriptor.render_queue, -20);
    assert_eq!(descriptor.material_queue, 7);
    assert_eq!(descriptor.depth_bias, -0.25);
    for name in ["render_queue", "material_queue", "depth_bias"] {
        assert!(material.shader_property_override(name).is_none());
        assert!(material
            .shader_property_overrides()
            .all(|(property, _)| property != name));
    }
    assert_eq!(
        material.shader_property_override("custom_gain"),
        Some(&toml::Value::Float(2.0))
    );

    let encoded = material.to_toml_string().unwrap();
    assert!(encoded.contains("render_queue = -20"));
    assert!(encoded.contains("material_queue = 7"));
    assert!(encoded.contains("depth_bias = -0.25"));

    material
        .property_values
        .insert("render_queue".to_string(), toml::Value::Integer(0));
    material
        .property_values
        .insert("material_queue".to_string(), toml::Value::Integer(0));
    material
        .property_values
        .insert("depth_bias".to_string(), toml::Value::Float(0.0));
    let encoded = material.to_toml_string().unwrap();
    assert!(!encoded.contains("render_queue"));
    assert!(!encoded.contains("material_queue"));
    assert!(!encoded.contains("depth_bias"));
}

#[test]
fn material_owned_sort_fields_report_invalid_override_types() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Queue Fields"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
render_queue = 2.5
material_queue = "front"
depth_bias = "near"
"#,
    )
    .unwrap();

    let errors = material.validation_errors();

    assert_eq!(material.render_queue(), 0);
    assert_eq!(material.material_queue(), 0);
    assert_eq!(material.depth_bias(), 0.0);
    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.render_queue"
            && name == "render_queue"
            && expected == "i32"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.material_queue"
            && name == "material_queue"
            && expected == "i32"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } if *source == RenderMaterialDiagnosticSource::MaterialOverride
            && path == "overrides.depth_bias"
            && name == "depth_bias"
            && expected == "number"
    )));
}

#[test]
fn material_asset_reports_invalid_lighting_model_as_material_validation_error() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Invalid Lighting"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
lighting_model = "toonish"
"#,
    )
    .unwrap();

    let errors = material.validation_errors();

    assert!(errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::InvalidLightingModel { path, value }
            if path == "overrides.lighting_model" && value == "toonish"
    )));
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
    material.texture_slots.insert("albedo".to_string(), {
        let mut slot = MaterialTextureSlotValue::new(shader_driven.clone());
        slot.transform = Some(RenderMaterialTextureTransform {
            scale: [4.0, 4.0],
            offset: [0.125, 0.25],
        });
        slot.uv_channel = 1;
        slot
    });
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
    assert_eq!(
        shader_descriptor.base_color_texture_transform,
        RenderMaterialTextureTransform {
            scale: [4.0, 4.0],
            offset: [0.125, 0.25],
        }
    );
    assert_eq!(shader_descriptor.base_color_texture_uv_channel, 1);
}

#[test]
fn material_asset_management_record_set_sorts_and_summarizes_records() {
    let shader = asset_reference("management-shader", "res://shaders/managed.zshader");
    let albedo = asset_reference("management-albedo", "res://textures/albedo.png");
    let material_with_issues = MaterialAsset {
        name: Some("ManagedGrid".to_string()),
        shader: shader.clone(),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: BTreeMap::from([("roughness".to_string(), toml::Value::Float(0.5))]),
        texture_slots: BTreeMap::from([
            (
                "albedo".to_string(),
                MaterialTextureSlotValue::new(albedo.clone()),
            ),
            (
                "normal".to_string(),
                MaterialTextureSlotValue {
                    reference: None,
                    fallback: Some("normal".to_string()),
                    transform: None,
                    uv_channel: 0,
                },
            ),
        ]),
        validation_diagnostics: vec!["imported with generated material defaults".to_string()],
    };
    let ready_material = MaterialAsset {
        name: Some("ManagedReady".to_string()),
        shader: shader.clone(),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: BTreeMap::new(),
        texture_slots: BTreeMap::new(),
        validation_diagnostics: Vec::new(),
    };
    let first_id = ResourceId::from_stable_label("material:first");
    let second_id = ResourceId::from_stable_label("material:second");

    let overview = material_with_issues.overview();
    assert_eq!(overview.name.as_deref(), Some("ManagedGrid"));
    assert_eq!(overview.shader, shader);
    assert_eq!(overview.property_override_count, 1);
    assert_eq!(overview.texture_slot_count, 2);
    assert_eq!(overview.texture_reference_count, 1);
    assert_eq!(overview.fallback_texture_slot_count, 1);
    assert_eq!(overview.validation_error_count, 0);
    assert_eq!(overview.validation_diagnostic_count, 1);
    assert_eq!(overview.direct_reference_count, 2);

    let record_set = MaterialAssetManagementRecordSet::from_records(vec![
        ready_material.management_record(second_id),
        material_with_issues.management_record(first_id),
    ]);
    let mut expected_ids = vec![first_id, second_id];
    expected_ids.sort();

    assert_eq!(
        record_set
            .records
            .iter()
            .map(|record| record.material_id)
            .collect::<Vec<_>>(),
        expected_ids
    );
    assert_eq!(record_set.summary.material_count, 2);
    assert_eq!(record_set.summary.ready_count, 1);
    assert_eq!(record_set.summary.issue_material_count, 1);
    assert_eq!(record_set.summary.degraded_count(), 1);
    assert_eq!(record_set.summary.property_override_count, 1);
    assert_eq!(record_set.summary.texture_slot_count, 2);
    assert_eq!(record_set.summary.texture_reference_count, 1);
    assert_eq!(record_set.summary.fallback_texture_slot_count, 1);
    assert_eq!(record_set.summary.validation_error_count, 0);
    assert_eq!(record_set.summary.validation_diagnostic_count, 1);
    assert_eq!(record_set.summary.issue_row_count(), 1);
    assert_eq!(record_set.summary.direct_reference_count, 3);
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

fn asset_reference(label: &str, uri: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_stable_label(label),
        AssetUri::parse(uri).unwrap(),
    )
}
