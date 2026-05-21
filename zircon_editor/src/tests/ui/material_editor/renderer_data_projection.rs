use crate::ui::material_editor::RendererDataEditorProjection;
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};
use zircon_runtime::graphics::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererFeatureAsset,
    RendererFeatureContractDiagnostic,
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
    let material = asset_reference("res://materials/other.zmaterial");
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
            feature_shader: shader,
            material_shader: material,
        },
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature: "mesh".to_string(),
            error: RenderMaterialValidationError::UnknownPropertyOverride {
                source: RenderMaterialDiagnosticSource::MaterialOverride,
                path: "overrides.base_colour".to_string(),
                name: "base_colour".to_string(),
            },
        },
        RendererFeatureContractDiagnostic::MaterialDiagnostic {
            feature: "mesh".to_string(),
            material: asset_reference("res://materials/pbr.zmaterial"),
            diagnostic: "material importer note".to_string(),
        },
    ];

    let projection = RendererDataEditorProjection::from_renderer_asset(&renderer, &diagnostics);

    let feature = &projection.features[0];
    assert_eq!(feature.diagnostic_count, 4);
    assert_eq!(projection.diagnostics.len(), 4);
    assert!(projection
        .diagnostics
        .iter()
        .all(|row| row.feature == "mesh"));
    assert!(projection.diagnostics.iter().any(|row| {
        row.path == "features.mesh.required_entry_points.vs_main"
            && row.message.contains("entry point `vs_main`")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.path == "features.mesh.material.shader" && row.message.contains("does not match")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source == Some(RenderMaterialDiagnosticSource::MaterialOverride)
            && row.path == "overrides.base_colour"
            && row.message.contains("base_colour")
    }));
    assert!(projection.diagnostics.iter().any(|row| {
        row.source.is_none()
            && row.path == "features.mesh.material.validation_diagnostics"
            && row.message.contains("material importer note")
    }));
}

fn asset_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}
