use super::*;

use crate::asset::ShaderDependencyAsset;
use crate::core::framework::render::RenderMaterialFallbackReason;
use crate::core::resource::ResourceKind;

#[test]
fn material_asset_reports_shader_contract_diagnostics_without_blocking_import() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
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
fn persisted_standard_pbr_overrides_are_ready_with_builtin_shader_contract() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Persisted Mirror"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "builtin://shader/pbr.wgsl"

[overrides]
base_color = [0.92, 0.92, 0.92, 1.0]
lighting_model = "pbr"
metallic = 1.0
roughness = 0.08
"#,
    )
    .unwrap();
    let mut shader = shader_contract();
    shader.property_schema.clear();
    shader.texture_slots.clear();

    let report = material.readiness_report_with_shader_contract(&shader, |_| true, |_| true);

    assert!(report.is_ready(), "standard PBR controls: {report:?}");
    assert!(report.validation_errors.is_empty());
}

#[test]
fn material_asset_rejects_a_ready_generic_shader_module() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Wrong Shader Domain"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/generic.wgsl"
"#,
    )
    .unwrap();
    let mut shader = shader_contract();
    shader.kind = crate::core::framework::render::ShaderAssetKind::Module;
    shader.shading_model = None;

    assert!(shader.readiness_report().is_ready());
    let report = material.readiness_report_with_shader_contract(&shader, |_| true, |_| true);

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } if *source == RenderMaterialDiagnosticSource::ShaderReadiness
            && path == "shader.kind"
            && diagnostic.contains("requires a surface shader, found module")
    )));
}

#[test]
fn material_asset_reports_missing_required_shader_texture_slot() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
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
version = 2
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
fn material_asset_readiness_reports_unresolved_shader_import_redirect_dependency() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "RedirectImport"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/redirect_surface.zshader"
"#,
    )
    .unwrap();
    let mut shader = shader_contract();
    shader.property_schema.clear();
    shader.texture_slots.clear();
    let redirected_module = asset_reference("missing-shared", "res://shaders/missing_shared");
    shader.dependencies = vec![ShaderDependencyAsset {
        kind: ResourceKind::Shader,
        reference: redirected_module.clone(),
    }];

    let report = material.readiness_report_with_shader_contract(
        &shader,
        |reference| reference != &redirected_module,
        |_| true,
    );

    assert!(!report.is_ready());
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { reference }
            if reference == &redirected_module
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference == &redirected_module
    )));
}

#[test]
fn material_asset_readiness_reports_material_local_diagnostics_without_blocking() {
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 2
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
version = 2

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
            rotation: 0.5,
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
        option: None,
        st: false,
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
            rotation: 0.5,
        }
    );
    assert_eq!(shader_descriptor.base_color_texture_uv_channel, 1);
}

#[test]
fn standard_pbr_readiness_rejects_texture_coordinates_outside_the_vertex_abi() {
    let texture = AssetReference::new(
        AssetUuid::from_stable_label("unsupported-uv-set"),
        AssetUri::parse("res://textures/unsupported-uv.png").unwrap(),
    );
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .unwrap();
    material.base_color_texture = Some(texture.clone());
    material.texture_slots.insert("base_color".to_string(), {
        let mut slot = MaterialTextureSlotValue::new(texture);
        slot.uv_channel = 2;
        slot
    });

    let report = material.readiness_report_with_resolution(|_| true, |_| true);

    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnsupportedTextureUvChannel {
            slot,
            channel: 2,
            supported_channel_count: 2,
        } if slot == "base_color"
    )));
}
