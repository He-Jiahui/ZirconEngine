use crate::ui::material_editor::RendererDataEditorProjection;
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};
use zircon_runtime::graphics::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererFeatureAsset,
    RendererFeatureContractDiagnostic, RendererFeatureContractDiagnosticSeverity,
};

#[test]
fn renderer_data_projection_surfaces_feature_contract_references() {
    let shader = asset_reference("res://shaders/pbr.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let renderer = RendererAsset {
        name: "Forward Renderer".to_string(),
        stages: vec![RenderPassStage::DepthPrepass, RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())
            .with_material_reference(material.clone())
            .with_required_entry_point("vs_main")
            .with_expected_property("base_color")
            .with_expected_texture_slot("albedo")],
    };

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &[]);

    assert_eq!(projection.renderer_name, "Forward Renderer");
    assert_eq!(projection.stages, vec!["DepthPrepass", "Opaque3d"]);
    assert_eq!(projection.features.len(), 1);
    let feature = &projection.features[0];
    assert_eq!(feature.name, "mesh");
    assert_eq!(feature.source, "Mesh");
    assert!(feature.enabled);
    assert_eq!(feature.quality_gate.as_deref(), Some("Mesh"));
    assert_eq!(feature.shader_reference.as_ref(), Some(&shader));
    assert_eq!(feature.material_reference.as_ref(), Some(&material));
    assert_eq!(feature.required_entry_points, vec!["vs_main"]);
    assert_eq!(feature.expected_properties, vec!["base_color"]);
    assert_eq!(feature.expected_texture_slots, vec!["albedo"]);
}

#[test]
fn renderer_data_projection_maps_diagnostics_to_feature_rows() {
    let shader = asset_reference("res://shaders/pbr.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let other_shader = asset_reference("res://shaders/other.zshader");
    let missing_material_shader = asset_reference("res://shaders/missing-material.zshader");
    let renderer = RendererAsset {
        name: "Forward Renderer".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())],
    };
    let diagnostics = vec![
        RendererFeatureContractDiagnostic::MissingEntryPoint {
            feature: "mesh".to_string(),
            shader: shader.clone(),
            entry_point: "vs_main".to_string(),
        },
        RendererFeatureContractDiagnostic::MaterialShaderMismatch {
            feature: "mesh".to_string(),
            material: material.clone(),
            feature_shader: shader.clone(),
            material_shader: other_shader.clone(),
        },
        RendererFeatureContractDiagnostic::MaterialShaderMissing {
            feature: "mesh".to_string(),
            material: material.clone(),
            shader: missing_material_shader.clone(),
        },
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material: material.clone(),
            shader: Some(shader.clone()),
            error: RenderMaterialValidationError::UnknownPropertyOverride {
                source: RenderMaterialDiagnosticSource::MaterialOverride,
                path: "overrides.base_colour".to_string(),
                name: "base_colour".to_string(),
            },
        },
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material: material.clone(),
            shader: None,
            error: RenderMaterialValidationError::InvalidLightingModel {
                path: "overrides.lighting_model".to_string(),
                value: "toon".to_string(),
            },
        },
        RendererFeatureContractDiagnostic::MaterialDiagnostic {
            feature: "mesh".to_string(),
            material: material.clone(),
            diagnostic: "material importer note".to_string(),
        },
    ];

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &diagnostics);

    let feature = &projection.features[0];
    assert_eq!(feature.diagnostic_count, 6);
    assert_eq!(projection.diagnostics.len(), 6);
    assert!(projection
        .diagnostics
        .iter()
        .all(|row| row.feature == "mesh"));
    assert!(projection.diagnostics.iter().any(|row| {
        row.shader_references == vec![shader.clone()]
            && row.path == "features.mesh.required_entry_points.vs_main"
            && row.message.contains("entry point `vs_main`")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.material_reference.as_ref() == Some(&material)
            && row.shader_references == vec![shader.clone(), other_shader.clone()]
            && row.path == "features.mesh.material.shader"
            && row.message.contains("does not match")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.material_reference.as_ref() == Some(&material)
            && row.shader_references == vec![missing_material_shader.clone()]
            && row.path == "features.mesh.material.shader"
            && row.message.contains("could not be resolved")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.material_reference.as_ref() == Some(&material)
            && row.shader_references == vec![shader.clone()]
            && row.source == Some(RenderMaterialDiagnosticSource::MaterialOverride)
            && row.path == "overrides.base_colour"
            && row.message.contains("base_colour")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.material_reference.as_ref() == Some(&material)
            && row.source.is_none()
            && row.path == "overrides.lighting_model"
            && row.message.contains("lighting model `toon`")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.material_reference.as_ref() == Some(&material)
            && row.source.is_none()
            && row.path == "features.mesh.material.validation_diagnostics"
            && row.message.contains("material importer note")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source == Some(RenderMaterialDiagnosticSource::MaterialOverride)
            && row.path == "overrides.base_colour"
            && row.message.contains("base_colour")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source.is_none()
            && row.path == "overrides.lighting_model"
            && row.message.contains("lighting model `toon`")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source.is_none()
            && row.path == "features.mesh.material.validation_diagnostics"
            && row.message.contains("material importer note")
    }));
}

#[test]
fn renderer_data_projection_groups_diagnostics_by_feature_name() {
    let mesh_shader = asset_reference("res://shaders/mesh.zshader");
    let ui_shader = asset_reference("res://shaders/ui.zshader");
    let renderer = RendererAsset {
        name: "Forward Renderer".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![
            RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
                .with_shader_reference(mesh_shader.clone()),
            RendererFeatureAsset::builtin(BuiltinRenderFeature::Ui)
                .with_shader_reference(ui_shader.clone()),
        ],
    };
    let diagnostics = vec![
        RendererFeatureContractDiagnostic::MissingEntryPoint {
            feature: "mesh".to_string(),
            shader: mesh_shader.clone(),
            entry_point: "vs_main".to_string(),
        },
        RendererFeatureContractDiagnostic::MissingProperty {
            feature: "mesh".to_string(),
            shader: mesh_shader,
            property: "base_color".to_string(),
        },
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature: "ui".to_string(),
            shader: ui_shader,
            diagnostic: "wgsl_capture missing ui_color".to_string(),
        },
    ];

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &diagnostics);
    let diagnostics_by_feature = projection.diagnostics_by_feature();

    assert_eq!(diagnostics_by_feature.len(), 2);
    let mesh_rows = diagnostics_by_feature.get("mesh").unwrap();
    assert_eq!(mesh_rows.len(), 2);
    assert!(mesh_rows.iter().all(|row| row.feature == "mesh"));
    assert_eq!(diagnostics_by_feature.get("ui").unwrap().len(), 1);
}

#[test]
fn renderer_data_projection_groups_diagnostics_by_source() {
    let shader = asset_reference("res://shaders/pbr.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let renderer = RendererAsset {
        name: "Forward Renderer".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())],
    };
    let diagnostics = vec![
        RendererFeatureContractDiagnostic::ShaderMissing {
            feature: "mesh".to_string(),
            reference: shader.clone(),
        },
        RendererFeatureContractDiagnostic::MissingProperty {
            feature: "mesh".to_string(),
            shader: shader.clone(),
            property: "base_color".to_string(),
        },
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material: material.clone(),
            shader: Some(shader.clone()),
            error: RenderMaterialValidationError::MissingRequiredProperty {
                source: RenderMaterialDiagnosticSource::ShaderSchema,
                path: "overrides.roughness".to_string(),
                name: "roughness".to_string(),
            },
        },
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material,
            shader: None,
            error: RenderMaterialValidationError::InvalidMaskCutoff { cutoff: 2.0 },
        },
    ];

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &diagnostics);
    let diagnostics_by_source = projection.diagnostics_by_source();

    assert_eq!(diagnostics_by_source.len(), 2);
    assert_eq!(
        diagnostics_by_source
            .get(&RenderMaterialDiagnosticSource::DependencyResolution)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        diagnostics_by_source
            .get(&RenderMaterialDiagnosticSource::ShaderSchema)
            .unwrap()
            .len(),
        2
    );
    assert!(diagnostics_by_source
        .get(&RenderMaterialDiagnosticSource::ShaderSchema)
        .unwrap()
        .iter()
        .any(|row| row.path == "overrides.roughness"));
    assert!(!diagnostics_by_source
        .values()
        .flatten()
        .any(|row| row.path == "overrides.alpha_mode.cutoff"));
}

#[test]
fn renderer_data_projection_groups_diagnostics_by_severity() {
    let shader = asset_reference("res://shaders/pbr.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let renderer = RendererAsset {
        name: "Forward Renderer".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())],
    };
    let diagnostics = vec![
        RendererFeatureContractDiagnostic::ShaderMissing {
            feature: "mesh".to_string(),
            reference: shader.clone(),
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
    ];

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &diagnostics);
    let diagnostics_by_severity = projection.diagnostics_by_severity();

    assert_eq!(diagnostics_by_severity.len(), 2);
    let error_rows = diagnostics_by_severity
        .get(&RendererFeatureContractDiagnosticSeverity::Error)
        .unwrap();
    assert_eq!(error_rows.len(), 1);
    assert_eq!(error_rows[0].path, "features.mesh.shader");
    assert!(error_rows[0].message.contains("could not be resolved"));
    assert_eq!(
        error_rows[0].severity,
        RendererFeatureContractDiagnosticSeverity::Error
    );

    let warning_rows = diagnostics_by_severity
        .get(&RendererFeatureContractDiagnosticSeverity::Warning)
        .unwrap();
    assert_eq!(warning_rows.len(), 2);
    assert!(warning_rows
        .iter()
        .all(|row| row.severity == RendererFeatureContractDiagnosticSeverity::Warning));
    assert!(warning_rows.iter().any(|row| {
        row.path == "features.mesh.material.validation_diagnostics"
            && row.message.contains("material importer note")
    }));
    assert!(warning_rows.iter().any(|row| {
        row.path == "features.mesh.shader.validation_diagnostics"
            && row.source == Some(RenderMaterialDiagnosticSource::WgslCapture)
    }));
}

#[test]
fn renderer_data_projection_groups_diagnostics_by_material_reference() {
    let material = asset_reference("res://materials/pbr.zmaterial");
    let shader = asset_reference("res://shaders/pbr.zshader");
    let material_shader = asset_reference("res://shaders/material.zshader");
    let renderer = RendererAsset {
        name: "Forward Renderer".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_material_reference(material.clone())],
    };
    let diagnostics = vec![
        RendererFeatureContractDiagnostic::ShaderMissing {
            feature: "mesh".to_string(),
            reference: shader.clone(),
        },
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material: material.clone(),
            shader: None,
            error: RenderMaterialValidationError::InvalidLightingModel {
                path: "overrides.lighting_model".to_string(),
                value: "toon".to_string(),
            },
        },
        RendererFeatureContractDiagnostic::MaterialDiagnostic {
            feature: "mesh".to_string(),
            material: material.clone(),
            diagnostic: "material importer note".to_string(),
        },
        RendererFeatureContractDiagnostic::MaterialShaderMismatch {
            feature: "mesh".to_string(),
            material: material.clone(),
            feature_shader: shader.clone(),
            material_shader,
        },
    ];

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &diagnostics);
    let diagnostics_by_material = projection.diagnostics_by_material();

    assert_eq!(diagnostics_by_material.len(), 1);
    let material_rows = diagnostics_by_material.get(&material).unwrap();
    assert_eq!(material_rows.len(), 3);
    assert!(material_rows.iter().any(|row| {
        row.path == "overrides.lighting_model" && row.message.contains("lighting model `toon`")
    }));
    assert!(material_rows.iter().any(|row| {
        row.path == "features.mesh.material.validation_diagnostics"
            && row.message.contains("material importer note")
    }));
    assert!(material_rows.iter().any(|row| {
        row.path == "features.mesh.material.shader" && row.message.contains("does not match")
    }));
    assert!(!material_rows
        .iter()
        .any(|row| row.path == "features.mesh.shader"));
}

#[test]
fn renderer_data_projection_groups_diagnostics_by_shader_reference() {
    let feature_shader = asset_reference("res://shaders/feature.zshader");
    let material_shader = asset_reference("res://shaders/material.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let renderer = RendererAsset {
        name: "Forward Renderer".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(feature_shader.clone())],
    };
    let diagnostics = vec![
        RendererFeatureContractDiagnostic::MissingProperty {
            feature: "mesh".to_string(),
            shader: feature_shader.clone(),
            property: "base_color".to_string(),
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
            error: RenderMaterialValidationError::MissingRequiredProperty {
                source: RenderMaterialDiagnosticSource::ShaderSchema,
                path: "overrides.emissive_power".to_string(),
                name: "emissive_power".to_string(),
            },
        },
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material,
            shader: None,
            error: RenderMaterialValidationError::InvalidMaskCutoff { cutoff: 2.0 },
        },
    ];

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &diagnostics);
    let diagnostics_by_shader = projection.diagnostics_by_shader();

    let feature_shader_rows = diagnostics_by_shader.get(&feature_shader).unwrap();
    assert_eq!(feature_shader_rows.len(), 2);
    assert!(feature_shader_rows.iter().any(|row| {
        row.path == "features.mesh.expected_properties.base_color"
            && row.message.contains("base_color")
    }));
    assert!(feature_shader_rows.iter().any(|row| {
        row.path == "features.mesh.material.shader" && row.message.contains("does not match")
    }));

    let material_shader_rows = diagnostics_by_shader.get(&material_shader).unwrap();
    assert_eq!(material_shader_rows.len(), 2);
    assert!(material_shader_rows.iter().any(|row| {
        row.path == "features.mesh.material.shader" && row.message.contains("does not match")
    }));
    assert!(material_shader_rows.iter().any(|row| {
        row.path == "overrides.emissive_power" && row.message.contains("emissive_power")
    }));
    assert!(!material_shader_rows
        .iter()
        .any(|row| row.path == "overrides.alpha_mode.cutoff"));
}

#[test]
fn renderer_data_projection_uses_runtime_diagnostic_ownership_without_shader_duplicates() {
    let shader = asset_reference("res://shaders/shared.zshader");
    let material = asset_reference("res://materials/pbr.zmaterial");
    let renderer = RendererAsset {
        name: "Forward Renderer".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())],
    };
    let diagnostics = vec![
        RendererFeatureContractDiagnostic::MaterialShaderMismatch {
            feature: "mesh".to_string(),
            material: material.clone(),
            feature_shader: shader.clone(),
            material_shader: shader.clone(),
        },
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            material: material.clone(),
            shader: Some(shader.clone()),
            error: RenderMaterialValidationError::UnresolvedShaderReference {
                reference: shader.clone(),
            },
        },
    ];

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &diagnostics);

    assert_eq!(projection.diagnostics.len(), 2);
    assert!(projection.diagnostics.iter().all(|row| {
        row.material_reference.as_ref() == Some(&material)
            && row.shader_references == vec![shader.clone()]
    }));
    let diagnostics_by_shader = projection.diagnostics_by_shader();
    assert_eq!(diagnostics_by_shader.get(&shader).unwrap().len(), 2);
}

fn asset_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}
