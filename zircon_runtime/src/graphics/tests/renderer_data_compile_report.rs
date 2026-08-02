use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract,
    RenderMaterialDiagnosticSource, RenderMaterialValidationError, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::graphics::{
    RenderPipelineAsset, RenderPipelineCompileReport, RendererFeatureContractDiagnostic,
    RendererFeatureContractDiagnosticSeverity,
};

#[test]
fn render_pipeline_compile_report_groups_diagnostics_by_feature_material_and_shader() {
    let feature_shader = asset_reference("res://shaders/feature.zshader");
    let material_shader = asset_reference("res://shaders/material.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let report = RenderPipelineCompileReport {
        pipeline: empty_compiled_pipeline(),
        diagnostics: vec![
            RendererFeatureContractDiagnostic::ShaderMissing {
                feature: "mesh".to_string(),
                reference: feature_shader.clone(),
            },
            RendererFeatureContractDiagnostic::MaterialMissing {
                feature: "mesh".to_string(),
                reference: material.clone(),
            },
            RendererFeatureContractDiagnostic::MaterialShaderMismatch {
                feature: "mesh".to_string(),
                material: material.clone(),
                feature_shader: feature_shader.clone(),
                material_shader: material_shader.clone(),
            },
            RendererFeatureContractDiagnostic::MaterialValidation {
                feature: "mesh".to_string(),
                material: material.clone(),
                shader: Some(material_shader.clone()),
                error: RenderMaterialValidationError::UnresolvedShaderReference {
                    reference: material_shader.clone(),
                },
            },
            RendererFeatureContractDiagnostic::MaterialValidation {
                feature: "mesh".to_string(),
                material: material.clone(),
                shader: None,
                error: RenderMaterialValidationError::InvalidMaskCutoff { cutoff: 2.0 },
            },
            RendererFeatureContractDiagnostic::ShaderValidation {
                feature: "ui".to_string(),
                shader: feature_shader.clone(),
                diagnostic: "wgsl_capture missing ui_color".to_string(),
            },
        ],
    };

    assert!(!report.pipeline.graph().passes().is_empty());

    let diagnostics_by_feature = report.diagnostics_by_feature();
    assert_eq!(diagnostics_by_feature["mesh"].len(), 5);
    assert_eq!(diagnostics_by_feature["ui"].len(), 1);

    let diagnostics_by_material = report.diagnostics_by_material();
    let material_rows = diagnostics_by_material.get(&material).unwrap();
    assert_eq!(material_rows.len(), 4);
    assert!(material_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialMissing { .. }
    )));
    assert!(material_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialShaderMismatch { .. }
    )));
    assert!(material_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialValidation { .. }
    )));

    let diagnostics_by_source = report.diagnostics_by_source();
    let dependency_rows = diagnostics_by_source
        .get(&RenderMaterialDiagnosticSource::DependencyResolution)
        .unwrap();
    assert_eq!(dependency_rows.len(), 4);
    assert!(dependency_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderMissing { .. }
    )));
    assert!(dependency_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialMissing { .. }
    )));
    assert!(dependency_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialShaderMismatch { .. }
    )));
    assert!(dependency_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialValidation {
            error: RenderMaterialValidationError::UnresolvedShaderReference { .. },
            ..
        }
    )));
    assert_eq!(
        diagnostics_by_source
            .get(&RenderMaterialDiagnosticSource::WgslCapture)
            .unwrap()
            .len(),
        1
    );
    assert!(!diagnostics_by_source
        .values()
        .flatten()
        .any(|diagnostic| matches!(
            diagnostic,
            RendererFeatureContractDiagnostic::MaterialValidation {
                error: RenderMaterialValidationError::InvalidMaskCutoff { .. },
                ..
            }
        )));

    let diagnostics_by_shader = report.diagnostics_by_shader();
    let feature_shader_rows = diagnostics_by_shader.get(&feature_shader).unwrap();
    assert_eq!(feature_shader_rows.len(), 3);
    assert!(feature_shader_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderMissing { .. }
    )));
    assert!(feature_shader_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialShaderMismatch { .. }
    )));
    assert!(feature_shader_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation { .. }
    )));

    let material_shader_rows = diagnostics_by_shader.get(&material_shader).unwrap();
    assert_eq!(material_shader_rows.len(), 2);
    assert!(material_shader_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialShaderMismatch { .. }
    )));
    assert!(material_shader_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialValidation {
            error:
                RenderMaterialValidationError::UnresolvedShaderReference {
                    reference,
                },
            ..
        } if reference == &material_shader
    )));
}

#[test]
fn render_pipeline_compile_report_groups_diagnostics_by_severity() {
    let shader = asset_reference("res://shaders/pbr.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let report = RenderPipelineCompileReport {
        pipeline: empty_compiled_pipeline(),
        diagnostics: vec![
            RendererFeatureContractDiagnostic::ShaderMissing {
                feature: "mesh".to_string(),
                reference: shader.clone(),
            },
            RendererFeatureContractDiagnostic::MaterialValidation {
                feature: "mesh".to_string(),
                material: material.clone(),
                shader: Some(shader.clone()),
                error: RenderMaterialValidationError::MissingRequiredProperty {
                    source: RenderMaterialDiagnosticSource::ShaderSchema,
                    path: "overrides.base_color".to_string(),
                    name: "base_color".to_string(),
                },
            },
            RendererFeatureContractDiagnostic::MaterialDiagnostic {
                feature: "mesh".to_string(),
                material,
                diagnostic: "material importer note".to_string(),
            },
            RendererFeatureContractDiagnostic::ShaderValidation {
                feature: "mesh".to_string(),
                shader,
                diagnostic: "wgsl_capture missing base_color".to_string(),
            },
        ],
    };

    let diagnostics_by_severity = report.diagnostics_by_severity();

    assert_eq!(diagnostics_by_severity.len(), 2);
    let error_rows = diagnostics_by_severity
        .get(&RendererFeatureContractDiagnosticSeverity::Error)
        .unwrap();
    assert_eq!(error_rows.len(), 2);
    assert!(error_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderMissing { .. }
    )));
    assert!(error_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialValidation { .. }
    )));
    let warning_rows = diagnostics_by_severity
        .get(&RendererFeatureContractDiagnosticSeverity::Warning)
        .unwrap();
    assert_eq!(warning_rows.len(), 2);
    assert!(warning_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialDiagnostic { .. }
    )));
    assert!(warning_rows.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation { .. }
    )));
}

#[test]
fn renderer_feature_contract_diagnostic_exposes_deduplicated_shader_references() {
    let shader = asset_reference("res://shaders/material.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let diagnostic = RendererFeatureContractDiagnostic::MaterialValidation {
        feature: "mesh".to_string(),
        material,
        shader: Some(shader.clone()),
        error: RenderMaterialValidationError::UnresolvedShaderReference {
            reference: shader.clone(),
        },
    };

    assert_eq!(diagnostic.shader_references(), vec![&shader]);
}

#[test]
fn renderer_feature_contract_diagnostic_exposes_canonical_severity() {
    let shader = asset_reference("res://shaders/material.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");

    assert_eq!(
        RendererFeatureContractDiagnostic::ShaderMissing {
            feature: "mesh".to_string(),
            reference: shader.clone(),
        }
        .severity(),
        RendererFeatureContractDiagnosticSeverity::Error
    );
    assert_eq!(
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material: material.clone(),
            shader: Some(shader.clone()),
            error: RenderMaterialValidationError::InvalidMaskCutoff { cutoff: 2.0 },
        }
        .severity(),
        RendererFeatureContractDiagnosticSeverity::Error
    );
    assert_eq!(
        RendererFeatureContractDiagnostic::MaterialDiagnostic {
            feature: "mesh".to_string(),
            material,
            diagnostic: "material importer note".to_string(),
        }
        .severity(),
        RendererFeatureContractDiagnosticSeverity::Warning
    );
    assert_eq!(
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature: "mesh".to_string(),
            shader,
            diagnostic: "wgsl_capture missing base_color".to_string(),
        }
        .severity(),
        RendererFeatureContractDiagnosticSeverity::Warning
    );
}

#[test]
fn renderer_feature_contract_diagnostic_exposes_canonical_sources() {
    let shader = asset_reference("res://shaders/material.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");

    assert_eq!(
        RendererFeatureContractDiagnostic::ShaderMissing {
            feature: "mesh".to_string(),
            reference: shader.clone(),
        }
        .source(),
        Some(RenderMaterialDiagnosticSource::DependencyResolution)
    );
    assert_eq!(
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material: material.clone(),
            shader: Some(shader.clone()),
            error: RenderMaterialValidationError::MissingRequiredProperty {
                source: RenderMaterialDiagnosticSource::ShaderSchema,
                path: "overrides.base_color".to_string(),
                name: "base_color".to_string(),
            },
        }
        .source(),
        Some(RenderMaterialDiagnosticSource::ShaderSchema)
    );
    assert_eq!(
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material,
            shader: None,
            error: RenderMaterialValidationError::InvalidLightingModel {
                path: "overrides.lighting_model".to_string(),
                value: "toon".to_string(),
            },
        }
        .source(),
        None
    );
    assert_eq!(
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature: "mesh".to_string(),
            shader,
            diagnostic: "wgsl_capture missing base_color".to_string(),
        }
        .source(),
        Some(RenderMaterialDiagnosticSource::WgslCapture)
    );
}

fn asset_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}

fn empty_compiled_pipeline() -> crate::graphics::CompiledRenderPipeline {
    RenderPipelineAsset::default_forward_plus()
        .compile(&test_extract())
        .unwrap()
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}
