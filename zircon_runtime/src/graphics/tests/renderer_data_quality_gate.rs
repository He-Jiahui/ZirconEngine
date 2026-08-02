use crate::graphics::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererDataDocument,
    RendererDataDocumentError, RendererFeatureAsset, RendererFeatureDocument,
};

#[test]
fn renderer_data_document_rejects_empty_quality_gate_before_runtime_projection() {
    let error = document_quality_gate_error(r#""""#);

    assert_eq!(
        error,
        RendererDataDocumentError::EmptyRenderFeatureQualityGate {
            feature: "Mesh".to_string(),
        }
    );
}

#[test]
fn renderer_data_document_rejects_padded_quality_gate_before_runtime_projection() {
    let error = document_quality_gate_error(r#"" Bloom ""#);

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRenderFeatureQualityGate {
            feature: "Mesh".to_string(),
            gate: " Bloom ".to_string(),
        }
    );
}

#[test]
fn renderer_data_document_rejects_padded_quality_gate_names() {
    let document = RendererDataDocument {
        version: 1,
        name: "quality-gate-keys".to_string(),
        stages: vec!["PostProcess".to_string()],
        features: vec![RendererFeatureDocument {
            name: "Bloom".to_string(),
            source: "Bloom".to_string(),
            enabled: true,
            quality_gate: Some(" Bloom ".to_string()),
            shader: None,
            material: None,
            required_entry_points: Vec::new(),
            expected_properties: Vec::new(),
            expected_texture_slots: Vec::new(),
            local_config: Default::default(),
        }],
    };

    let error = document.to_renderer_asset().unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRenderFeatureQualityGate {
            feature: "Bloom".to_string(),
            gate: " Bloom ".to_string(),
        }
    );
}

#[test]
fn renderer_asset_projection_preserves_cross_feature_quality_gate() {
    let renderer = RendererAsset {
        name: "quality-gate-projection".to_string(),
        stages: vec![RenderPassStage::PostProcess],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Bloom)
            .with_quality_gate(BuiltinRenderFeature::PostProcess)],
    };

    let document = RendererDataDocument::from_renderer_asset(&renderer).unwrap();

    assert_eq!(
        document.features[0].quality_gate.as_deref(),
        Some("PostProcess")
    );
    assert_eq!(document.to_renderer_asset().unwrap(), renderer);
}

fn document_quality_gate_error(gate: &str) -> RendererDataDocumentError {
    let document = RendererDataDocument::from_toml_str(&format!(
        r#"
version = 1
name = "quality-gate-keys"
stages = ["PostProcess"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
quality_gate = {gate}
"#
    ))
    .unwrap();

    document.to_renderer_asset().unwrap_err()
}
