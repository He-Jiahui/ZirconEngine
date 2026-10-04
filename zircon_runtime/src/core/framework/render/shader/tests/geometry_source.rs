use std::collections::HashSet;

use super::{
    builtin_geometry_source_descriptor, builtin_geometry_source_descriptors,
    GeometrySourceBindingKind, GeometrySourceId, GeometrySourceVertexAttribute,
    GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    GEOMETRY_SOURCE_PLUGIN_ID_START, GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH, GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
};

#[test]
fn render_shader_geometry_source_ids_reserve_builtin_segment() {
    assert_eq!(GEOMETRY_SOURCE_ID_STATIC_MESH.value(), 0);
    assert_eq!(GEOMETRY_SOURCE_ID_SKINNED_MESH.value(), 1);
    assert_eq!(GEOMETRY_SOURCE_ID_MORPHED_MESH.value(), 2);
    assert_eq!(GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH.value(), 3);
    assert!(!GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH.is_plugin_range());
    assert!(GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START).is_plugin_range());
}

#[test]
fn render_shader_geometry_source_descriptors_cover_builtin_segment() {
    let descriptors = builtin_geometry_source_descriptors();
    let ids = descriptors
        .iter()
        .map(|descriptor| descriptor.id.value())
        .collect::<HashSet<_>>();
    let tokens = descriptors
        .iter()
        .map(|descriptor| descriptor.token.as_str())
        .collect::<HashSet<_>>();

    assert_eq!(descriptors.len(), 4);
    assert_eq!(ids.len(), 4);
    assert_eq!(tokens.len(), 4);
    assert_eq!(
        descriptors
            .iter()
            .map(|descriptor| descriptor.wgsl_include.as_str())
            .collect::<Vec<_>>(),
        vec![
            GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
            GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
            GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
            GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
        ]
    );
}

#[test]
fn render_shader_geometry_source_descriptors_report_shape_requirements() {
    let static_mesh = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static mesh descriptor");
    let skinned = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_SKINNED_MESH)
        .expect("skinned mesh descriptor");
    let morphed = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_MORPHED_MESH)
        .expect("morphed mesh descriptor");
    let skinned_morphed =
        builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH)
            .expect("skinned morphed mesh descriptor");

    assert!(static_mesh.requires_binding(GeometrySourceBindingKind::GpuSceneInstance));
    assert!(!static_mesh.requires_binding(GeometrySourceBindingKind::SkinningPaletteStorage));
    assert!(!static_mesh.requires_binding(GeometrySourceBindingKind::MorphWeightsStorage));

    assert!(skinned.has_vertex_attribute(GeometrySourceVertexAttribute::JointIndices));
    assert!(skinned.requires_binding(GeometrySourceBindingKind::SkinningPaletteStorage));
    assert!(!skinned.requires_binding(GeometrySourceBindingKind::MorphTargetStorage));

    assert!(morphed.has_vertex_attribute(GeometrySourceVertexAttribute::MorphPositionDelta));
    assert!(morphed.requires_binding(GeometrySourceBindingKind::MorphWeightsStorage));
    assert!(morphed.requires_binding(GeometrySourceBindingKind::MorphTargetStorage));
    assert!(!morphed.requires_binding(GeometrySourceBindingKind::SkinningPaletteStorage));

    assert!(skinned_morphed.has_vertex_attribute(GeometrySourceVertexAttribute::JointWeights));
    assert!(skinned_morphed.has_vertex_attribute(GeometrySourceVertexAttribute::MorphNormalDelta));
    assert!(skinned_morphed.requires_binding(GeometrySourceBindingKind::SkinningPaletteStorage));
    assert!(skinned_morphed.requires_binding(GeometrySourceBindingKind::MorphTargetStorage));
}
