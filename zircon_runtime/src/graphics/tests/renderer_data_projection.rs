use std::str::FromStr;

use crate::asset::{AssetReference, AssetUri, AssetUuid};
use crate::graphics::feature::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
};
use crate::render_graph::QueueLane;
use crate::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererDataDocument,
    RendererDataDocumentError, RendererFeatureAsset,
};

#[test]
fn renderer_asset_projects_to_renderer_data_document_with_authoring_names() {
    let shader = asset_reference(
        "renderer-data-projection-shader",
        "res://shaders/mesh.zshader",
    );
    let material = asset_reference(
        "renderer-data-projection-material",
        "res://materials/mesh.zmaterial",
    );
    let renderer = RendererAsset {
        name: "forward-authoring".to_string(),
        stages: vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Opaque3d,
            RenderPassStage::PostProcess,
            RenderPassStage::Ui,
        ],
        features: vec![
            RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
                .with_shader_reference(shader.clone())
                .with_material_reference(material.clone())
                .with_required_entry_point("vs_main")
                .with_required_entry_point("fs_main")
                .with_expected_property("base_color")
                .with_expected_texture_slot("base_color")
                .with_local_config("variant", "lit"),
            RendererFeatureAsset::disabled(BuiltinRenderFeature::Ui).without_quality_gate(),
        ],
    };

    let document = RendererDataDocument::from_renderer_asset(&renderer).unwrap();

    assert_eq!(document.version, 1);
    assert_eq!(document.name, "forward-authoring");
    assert_eq!(
        document.stages,
        vec!["DepthPrepass", "Opaque3d", "PostProcess", "Ui"]
    );
    assert_eq!(document.features.len(), 2);

    let mesh = &document.features[0];
    assert_eq!(mesh.name, "Mesh");
    assert_eq!(mesh.source, "Mesh");
    assert!(mesh.enabled);
    assert_eq!(mesh.quality_gate.as_deref(), Some("Mesh"));
    assert_eq!(mesh.shader, Some(shader));
    assert_eq!(mesh.material, Some(material));
    assert_eq!(mesh.required_entry_points, vec!["vs_main", "fs_main"]);
    assert_eq!(mesh.expected_properties, vec!["base_color"]);
    assert_eq!(mesh.expected_texture_slots, vec!["base_color"]);
    assert_eq!(mesh.local_config.get("variant").unwrap(), "lit");

    let ui = &document.features[1];
    assert_eq!(ui.name, "Ui");
    assert_eq!(ui.source, "Ui");
    assert!(!ui.enabled);
    assert_eq!(ui.quality_gate, None);

    let restored = document.to_renderer_asset().unwrap();
    assert_eq!(restored, renderer);
}

#[test]
fn renderer_asset_projection_rejects_non_renderer_data_stage() {
    let renderer = RendererAsset {
        name: "internal-stage".to_string(),
        stages: vec![RenderPassStage::Opaque],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)],
    };

    let error = RendererDataDocument::from_renderer_asset(&renderer).unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::UnsupportedRendererAssetStage {
            stage: RenderPassStage::Opaque
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_plugin_feature_sources() {
    let renderer = RendererAsset {
        name: "plugin-feature".to_string(),
        stages: vec![RenderPassStage::PostProcess],
        features: vec![RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
            "plugin.custom",
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ))],
    };

    let error = RendererDataDocument::from_renderer_asset(&renderer).unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::UnsupportedRendererAssetFeatureSource {
            value: "plugin.custom".to_string()
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_descriptor_overrides() {
    let descriptor = RenderFeatureDescriptor::new(
        "mesh.override",
        vec!["scene".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Opaque3d,
            "mesh.override",
            QueueLane::Graphics,
        )],
    );
    let renderer = RendererAsset {
        name: "override-feature".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_descriptor_override(descriptor)],
    };

    let error = RendererDataDocument::from_renderer_asset(&renderer).unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::UnsupportedRendererAssetDescriptorOverride {
            feature: "Mesh".to_string()
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_runtime_only_capability_requirements() {
    let renderer = RendererAsset {
        name: "capability-feature".to_string(),
        stages: vec![RenderPassStage::PostProcess],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_capability_requirement(RenderFeatureCapabilityRequirement::SparseTexture)],
    };

    let error = RendererDataDocument::from_renderer_asset(&renderer).unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::UnsupportedRendererAssetCapabilityRequirements {
            feature: "Mesh".to_string()
        }
    );
}

fn asset_reference(label: &str, uri: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_str(label).unwrap_or_else(|_| AssetUuid::from_stable_label(label)),
        AssetUri::parse(uri).unwrap(),
    )
}
