use std::collections::BTreeMap;
use std::fs;

use zircon_runtime::core::framework::render::{
    GeometrySourceBindingKind, GeometrySourceBindingRequirement, GeometrySourceDescriptor,
    GeometrySourceId, GeometrySourceVertexAttribute, RenderShaderDefinitionValue,
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_STATIC_MESH, GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_ID_STANDARD_PBR,
};

use super::super::{
    asset_root_manifest_for_quality_tiers_and_geometry_sources,
    builtin_fallback_manifest_for_quality_tiers_and_geometry_sources,
    builtin_fallback_manifest_for_quality_tiers_geometry_sources_and_descriptors,
};
use super::{source_for, BUILTIN_MATERIAL_PASS_TYPES};

#[test]
fn shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources() {
    let manifest = builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(
        &[ShaderQualityTier::Medium, ShaderQualityTier::High],
        &[
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ],
        &BTreeMap::new(),
    );

    assert_eq!(manifest.variants.len(), 24);
    assert_eq!(
        manifest
            .variants
            .iter()
            .filter(|request| request.key.geometry_source == GEOMETRY_SOURCE_ID_STATIC_MESH)
            .count(),
        12
    );
    assert_eq!(
        manifest
            .variants
            .iter()
            .filter(|request| request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH)
            .count(),
        12
    );
    for pass_type in BUILTIN_MATERIAL_PASS_TYPES {
        assert_eq!(
            manifest
                .variants
                .iter()
                .filter(|request| request.key.pass_type == pass_type)
                .count(),
            4
        );
    }
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.shading_model == SHADING_MODEL_ID_STANDARD_PBR));
    assert!(manifest.variants.iter().all(|request| request
        .key
        .features
        .contains(ShaderFeatureBits::RECEIVE_SHADOWS)));
    assert!(manifest
        .variants
        .iter()
        .any(|request| request.key.quality == ShaderQualityTier::High
            && source_for(&manifest, request)
                .wgsl_source
                .contains("// include: zr_geometry_skinned.wgsl")));
    let skinned_depth_request = manifest
        .variants
        .iter()
        .find(|request| {
            request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH
                && request.key.quality == ShaderQualityTier::High
                && request.key.pass_type == ShaderPassType::DepthPrepass
        })
        .expect("high-quality skinned depth-only builtin fallback request");
    let skinned_depth_source = source_for(&manifest, skinned_depth_request);
    assert!(skinned_depth_source
        .wgsl_source
        .contains("// include: zr_template_depth.wgsl"));
    assert!(!skinned_depth_source
        .wgsl_source
        .contains("zr_material_surface"));
    assert!(!skinned_depth_source
        .wgsl_source
        .contains("surface.normal_ws * 0.5"));
    assert!(!skinned_depth_source
        .wgsl_source
        .contains("// include: zr_template_gbuffer.wgsl"));
}

#[test]
fn shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_custom_geometry_source_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("procedural.zshader"),
        r#"version = 2
kind = "surface"
wgsl_files = ["procedural.wgsl"]
shading_model = "standard_pbr"
"#,
    )
    .unwrap();
    fs::write(root.join("procedural.wgsl"), "fn procedural() {}\n").unwrap();

    let custom_geometry_source = GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START);
    let manifest = asset_root_manifest_for_quality_tiers_and_geometry_sources(
        &root,
        &[ShaderQualityTier::Medium],
        &[custom_geometry_source],
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest.variants.iter().all(|request| {
        request.key.geometry_source == custom_geometry_source
            && request.key.geometry_source.is_plugin_range()
    }));
    for pass_type in BUILTIN_MATERIAL_PASS_TYPES {
        assert!(manifest
            .variants
            .iter()
            .any(|request| request.key.pass_type == pass_type));
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_builtin_fallback_manifest_uses_custom_geometry_source_descriptor() {
    let custom_geometry_source = GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START);
    let mut descriptors = BTreeMap::new();
    descriptors.insert(custom_geometry_source, virtual_geometry_source_descriptor());

    let manifest = builtin_fallback_manifest_for_quality_tiers_geometry_sources_and_descriptors(
        &[ShaderQualityTier::Medium],
        &[custom_geometry_source],
        &descriptors,
    );

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest.variants.iter().all(|request| {
        let source = source_for(&manifest, request);
        request.key.geometry_source == custom_geometry_source
            && request.key.geometry_source.is_plugin_range()
            && source
                .wgsl_source
                .contains("// include: zr_geometry_virtual_geometry.wgsl")
            && source
                .wgsl_source
                .contains("const ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY: bool = true;")
            && source.template_revision == "zr-material-template-v1"
    }));
    for pass_type in BUILTIN_MATERIAL_PASS_TYPES {
        assert!(manifest
            .variants
            .iter()
            .any(|request| request.key.pass_type == pass_type));
    }
}

#[test]
fn shader_prewarm_asset_root_manifest_expands_requested_geometry_sources() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_geometry_scan_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("simple.zshader"),
        r#"version = 2
kind = "surface"
wgsl_files = ["simple.wgsl"]
shading_model = "standard_pbr"
"#,
    )
    .unwrap();
    fs::write(root.join("simple.wgsl"), "fn simple() {}\n").unwrap();

    let manifest = asset_root_manifest_for_quality_tiers_and_geometry_sources(
        &root,
        &[ShaderQualityTier::Medium],
        &[
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ],
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 12);
    assert_eq!(
        manifest
            .variants
            .iter()
            .filter(|request| request.key.geometry_source == GEOMETRY_SOURCE_ID_STATIC_MESH)
            .count(),
        6
    );
    assert_eq!(
        manifest
            .variants
            .iter()
            .filter(|request| request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH)
            .count(),
        6
    );
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.quality == ShaderQualityTier::Medium));

    let _ = fs::remove_dir_all(root);
}

fn virtual_geometry_source_descriptor() -> GeometrySourceDescriptor {
    GeometrySourceDescriptor {
        id: GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START),
        token: "custom:virtual_geometry".to_string(),
        wgsl_include: "zr_geometry_virtual_geometry.wgsl".to_string(),
        vertex_attributes: vec![
            GeometrySourceVertexAttribute::Position,
            GeometrySourceVertexAttribute::Normal,
            GeometrySourceVertexAttribute::Tangent,
            GeometrySourceVertexAttribute::Uv0,
        ],
        required_bindings: vec![
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryPages,
                "virtual_geometry.pages",
            ),
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryClusters,
                "virtual_geometry.clusters",
            ),
        ],
        shader_defines: vec![RenderShaderDefinitionValue::bool(
            "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY",
            true,
        )],
    }
}
