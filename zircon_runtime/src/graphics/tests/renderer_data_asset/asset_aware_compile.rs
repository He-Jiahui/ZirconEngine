use crate::asset::{ShaderEntryPointAsset, ShaderSourceLanguage};
use crate::core::framework::render::{RenderMaterialValidationError, RenderShaderDefinitionValue};
use crate::graphics::{
    BuiltinRenderFeature, RenderPipelineCompileOptions, RendererFeatureContractDiagnostic,
};

use super::{
    assert_material_validation, asset_reference, material_with_contract_gaps,
    pipeline_with_mesh_feature, shader_contract, shader_with_validation_diagnostic, test_extract,
    InMemoryRenderPipelineAssetContext,
};

#[test]
fn asset_aware_compile_reports_missing_shader_and_material_without_blocking_graph() {
    let shader = asset_reference("missing-shader", "res://shaders/missing.zshader");
    let material = asset_reference("missing-material", "res://materials/missing.zmaterial");
    let pipeline = pipeline_with_mesh_feature(
        crate::graphics::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())
            .with_material_reference(material.clone()),
    );

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &InMemoryRenderPipelineAssetContext::default(),
        )
        .unwrap();

    assert!(report
        .pipeline
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "opaque-mesh"));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderMissing { feature, reference }
            if feature == "mesh" && reference == &shader
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialMissing { feature, reference }
            if feature == "mesh" && reference == &material
    )));
}

#[test]
fn asset_aware_compile_reports_shader_contract_expectation_gaps() {
    let shader = asset_reference("mesh-shader", "res://shaders/mesh.zshader");
    let pipeline = pipeline_with_mesh_feature(
        crate::graphics::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())
            .with_required_entry_point("missing_vs")
            .with_expected_property("roughness")
            .with_expected_texture_slot("normal"),
    );
    let context = InMemoryRenderPipelineAssetContext::default().with_shader(
        shader.clone(),
        shader_with_validation_diagnostic("capture missing"),
    );

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MissingEntryPoint { feature, shader: diagnostic_shader, entry_point }
            if feature == "mesh" && diagnostic_shader == &shader && entry_point == "missing_vs"
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MissingProperty { feature, shader: diagnostic_shader, property }
            if feature == "mesh" && diagnostic_shader == &shader && property == "roughness"
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MissingTextureSlot { feature, shader: diagnostic_shader, slot }
            if feature == "mesh" && diagnostic_shader == &shader && slot == "normal"
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation { feature, shader: diagnostic_shader, diagnostic }
            if feature == "mesh" && diagnostic_shader == &shader && diagnostic == "capture missing"
    )));
}

#[test]
fn asset_aware_compile_reports_shader_payload_readiness_gaps() {
    let shader = asset_reference("readiness-shader", "res://shaders/readiness.zshader");
    let pipeline = pipeline_with_mesh_feature(
        crate::graphics::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone()),
    );
    let mut shader_asset = shader_contract();
    shader_asset.uri = shader.locator.clone();
    shader_asset.source_language = ShaderSourceLanguage::Glsl;
    shader_asset.source = "void main() {}".to_string();
    shader_asset.wgsl_source.clear();
    shader_asset.entry_points.push(ShaderEntryPointAsset {
        name: "pixel_main".to_string(),
        stage: "pixel".to_string(),
    });
    shader_asset.shader_defs = vec![
        RenderShaderDefinitionValue::from("USE_FOG"),
        RenderShaderDefinitionValue::from(" "),
        RenderShaderDefinitionValue::bool(" USE_FOG ", false),
    ];
    let context =
        InMemoryRenderPipelineAssetContext::default().with_shader(shader.clone(), shader_asset);

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader: diagnostic_shader,
            diagnostic,
        } if feature == "mesh"
            && diagnostic_shader == &shader
            && diagnostic.contains("does not provide emitted WGSL")
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader: diagnostic_shader,
            diagnostic,
        } if feature == "mesh"
            && diagnostic_shader == &shader
            && diagnostic.contains("unsupported stage `pixel`")
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader: diagnostic_shader,
            diagnostic,
        } if feature == "mesh"
            && diagnostic_shader == &shader
            && diagnostic.contains("empty after trimming")
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader: diagnostic_shader,
            diagnostic,
        } if feature == "mesh"
            && diagnostic_shader == &shader
            && diagnostic.contains("duplicated")
    )));
}

#[test]
fn asset_aware_compile_reports_material_contract_diagnostics() {
    let feature_shader = asset_reference("feature-shader", "res://shaders/feature.zshader");
    let material_shader = asset_reference("material-shader", "res://shaders/material.zshader");
    let material = asset_reference("material", "res://materials/mismatch.zmaterial");
    let pipeline = pipeline_with_mesh_feature(
        crate::graphics::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(feature_shader.clone())
            .with_material_reference(material.clone()),
    );
    let context = InMemoryRenderPipelineAssetContext::default()
        .with_shader(feature_shader.clone(), shader_contract())
        .with_material(
            material.clone(),
            material_with_contract_gaps(material_shader.clone()),
        );

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert!(report
        .pipeline
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "opaque-mesh"));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialShaderMismatch {
            feature,
            material: diagnostic_material,
            feature_shader: diagnostic_feature_shader,
            material_shader: diagnostic_material_shader,
        } if feature == "mesh"
            && diagnostic_material == &material
            && diagnostic_feature_shader == &feature_shader
            && diagnostic_material_shader == &material_shader
    )));
    assert_material_validation(&report.diagnostics, &material, |error| {
        matches!(
            error,
            RenderMaterialValidationError::UnknownPropertyOverride { name, .. } if name == "unknown_scalar"
        )
    });
    assert_material_validation(&report.diagnostics, &material, |error| {
        matches!(
            error,
            RenderMaterialValidationError::PropertyOverrideTypeMismatch { name, expected, .. }
                if name == "base_color" && expected == "vec4"
        )
    });
    assert_material_validation(&report.diagnostics, &material, |error| {
        matches!(
            error,
            RenderMaterialValidationError::MissingRequiredProperty { name, .. } if name == "emissive_power"
        )
    });
    assert_material_validation(&report.diagnostics, &material, |error| {
        matches!(
            error,
            RenderMaterialValidationError::UnknownTextureSlot { slot, .. } if slot == "unknown_slot"
        )
    });
}

#[test]
fn asset_aware_compile_reports_material_local_validation_diagnostics() {
    let feature_shader = asset_reference("feature-shader", "res://shaders/feature.zshader");
    let material_reference = asset_reference("material", "res://materials/local-errors.zmaterial");
    let pipeline = pipeline_with_mesh_feature(
        crate::graphics::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(feature_shader.clone())
            .with_material_reference(material_reference.clone()),
    );
    let mut material = material_with_contract_gaps(feature_shader.clone());
    material.alpha_mode = crate::asset::AlphaMode::Mask { cutoff: 2.0 };
    material
        .validation_diagnostics
        .push("material importer note".to_string());
    let context = InMemoryRenderPipelineAssetContext::default()
        .with_shader(feature_shader, shader_contract())
        .with_material(material_reference.clone(), material);

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert_material_validation(&report.diagnostics, &material_reference, |error| {
        matches!(
            error,
            RenderMaterialValidationError::InvalidMaskCutoff { cutoff } if *cutoff == 2.0
        )
    });
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialDiagnostic { feature, material, diagnostic }
            if feature == "mesh"
                && material == &material_reference
                && diagnostic == "material importer note"
    )));
}
