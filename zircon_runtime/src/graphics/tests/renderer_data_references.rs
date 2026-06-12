use crate::asset::{AssetReference, AssetUri, AssetUuid};
use crate::{
    BuiltinRenderFeature, RenderPassStage, RendererAsset, RendererDataDocument,
    RendererDataDocumentError, RendererFeatureAsset, RendererFeatureReferenceListKind,
};

#[test]
fn renderer_data_document_rejects_duplicate_required_entry_points() {
    let error = reference_error(
        r#"
required_entry_points = ["vs_main", "fs_main", "vs_main"]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::DuplicateRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::RequiredEntryPoints,
            value: "vs_main".to_string()
        }
    );
}

#[test]
fn renderer_data_document_rejects_duplicate_expected_properties() {
    let error = reference_error(
        r#"
expected_properties = ["base_color", "roughness", "base_color"]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::DuplicateRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedProperties,
            value: "base_color".to_string()
        }
    );
}

#[test]
fn renderer_data_document_rejects_duplicate_expected_texture_slots() {
    let error = reference_error(
        r#"
expected_texture_slots = ["base_color", "normal", "normal"]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::DuplicateRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedTextureSlots,
            value: "normal".to_string()
        }
    );
}

#[test]
fn renderer_data_document_rejects_empty_required_entry_points() {
    let error = reference_error(
        r#"
required_entry_points = ["vs_main", ""]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::EmptyRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::RequiredEntryPoints,
        }
    );
}

#[test]
fn renderer_data_document_rejects_blank_expected_properties() {
    let error = reference_error(
        r#"
expected_properties = ["base_color", "   "]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::EmptyRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedProperties,
        }
    );
}

#[test]
fn renderer_data_document_rejects_empty_expected_texture_slots() {
    let error = reference_error(
        r#"
expected_texture_slots = ["base_color", ""]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::EmptyRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedTextureSlots,
        }
    );
}

#[test]
fn renderer_data_document_rejects_padded_required_entry_points() {
    let error = reference_error(
        r#"
required_entry_points = [" vs_main"]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::RequiredEntryPoints,
            value: " vs_main".to_string(),
        }
    );
}

#[test]
fn renderer_data_document_rejects_padded_expected_properties() {
    let error = reference_error(
        r#"
expected_properties = ["base_color "]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedProperties,
            value: "base_color ".to_string(),
        }
    );
}

#[test]
fn renderer_data_document_rejects_padded_expected_texture_slots() {
    let error = reference_error(
        r#"
expected_texture_slots = [" normal "]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedTextureSlots,
            value: " normal ".to_string(),
        }
    );
}

#[test]
fn renderer_data_document_rejects_required_entry_points_without_shader_reference() {
    let error = reference_error(
        r#"
required_entry_points = ["vs_main"]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::MissingRenderFeatureShaderReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::RequiredEntryPoints,
        }
    );
}

#[test]
fn renderer_data_document_rejects_expected_properties_without_shader_reference() {
    let error = reference_error(
        r#"
expected_properties = ["base_color"]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::MissingRenderFeatureShaderReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedProperties,
        }
    );
}

#[test]
fn renderer_data_document_rejects_expected_texture_slots_without_shader_reference() {
    let error = reference_error(
        r#"
expected_texture_slots = ["normal"]
"#,
    );

    assert_eq!(
        error,
        RendererDataDocumentError::MissingRenderFeatureShaderReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedTextureSlots,
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_duplicate_required_entry_points() {
    let error = runtime_reference_error(
        RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_required_entry_point("vs_main")
            .with_required_entry_point("vs_main"),
    );

    assert_eq!(
        error,
        RendererDataDocumentError::DuplicateRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::RequiredEntryPoints,
            value: "vs_main".to_string(),
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_empty_expected_properties() {
    let error = runtime_reference_error(
        RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh).with_expected_property(""),
    );

    assert_eq!(
        error,
        RendererDataDocumentError::EmptyRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedProperties,
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_padded_expected_texture_slots() {
    let error = runtime_reference_error(
        RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_expected_texture_slot(" normal "),
    );

    assert_eq!(
        error,
        RendererDataDocumentError::PaddedRenderFeatureReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedTextureSlots,
            value: " normal ".to_string(),
        }
    );
}

#[test]
fn renderer_asset_projection_rejects_contract_references_without_shader_reference() {
    let error = runtime_reference_error(
        RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_expected_texture_slot("normal"),
    );

    assert_eq!(
        error,
        RendererDataDocumentError::MissingRenderFeatureShaderReference {
            feature: "Mesh".to_string(),
            list: RendererFeatureReferenceListKind::ExpectedTextureSlots,
        }
    );
}

#[test]
fn renderer_asset_projection_accepts_contract_references_with_shader_reference() {
    let renderer = RendererAsset {
        name: "valid-runtime-references".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader_reference())
            .with_expected_texture_slot("normal")],
    };

    let document = RendererDataDocument::from_renderer_asset(&renderer).unwrap();

    assert_eq!(
        document.features[0].expected_texture_slots,
        vec!["normal".to_string()]
    );
    assert_eq!(document.features[0].shader, Some(shader_reference()));
}

fn reference_error(reference_list: &str) -> RendererDataDocumentError {
    let document = RendererDataDocument::from_toml_str(&format!(
        r#"
version = 1
name = "duplicate-feature-references"
stages = ["Opaque3d"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
{reference_list}
"#
    ))
    .unwrap();

    document.to_renderer_asset().unwrap_err()
}

fn shader_reference() -> AssetReference {
    AssetReference::new(
        AssetUuid::from_stable_label("mesh-shader"),
        AssetUri::parse("res://shaders/mesh.zshader").unwrap(),
    )
}

fn runtime_reference_error(feature: RendererFeatureAsset) -> RendererDataDocumentError {
    let renderer = RendererAsset {
        name: "invalid-runtime-references".to_string(),
        stages: vec![RenderPassStage::Opaque3d],
        features: vec![feature],
    };

    RendererDataDocument::from_renderer_asset(&renderer).unwrap_err()
}
