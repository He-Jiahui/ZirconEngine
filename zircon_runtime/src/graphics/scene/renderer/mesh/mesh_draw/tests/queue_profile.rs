use crate::core::framework::render::{
    GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use crate::core::framework::scene::Mobility;

use super::{MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile};

#[test]
fn queue_profile_maps_prepared_gpu_skinning_to_skinned_shader_geometry() {
    let profile = MeshDrawQueueProfile::new(
        MeshDrawQueuePhase::Opaque,
        MeshDrawGeometrySource::Prepared,
        Mobility::Dynamic,
        false,
        true,
        false,
    );

    assert_eq!(profile.geometry_source(), MeshDrawGeometrySource::Prepared);
    assert_eq!(
        profile.shader_geometry_source_id(),
        GEOMETRY_SOURCE_ID_SKINNED_MESH
    );
}

#[test]
fn queue_profile_keeps_cpu_fallback_dynamic_on_static_shader_geometry() {
    let profile = MeshDrawQueueProfile::new(
        MeshDrawQueuePhase::Opaque,
        MeshDrawGeometrySource::Dynamic,
        Mobility::Dynamic,
        false,
        false,
        false,
    );

    assert_eq!(
        profile.shader_geometry_source_id(),
        GEOMETRY_SOURCE_ID_STATIC_MESH
    );
}

#[test]
fn queue_profile_preserves_cpu_morphed_gpu_skinning_source_metadata() {
    let profile = MeshDrawQueueProfile::new(
        MeshDrawQueuePhase::Opaque,
        MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource,
        Mobility::Dynamic,
        false,
        true,
        false,
    );

    assert!(profile
        .geometry_source()
        .uses_cpu_morphed_gpu_skinning_source());
    assert_eq!(
        profile.shader_geometry_source_id(),
        GEOMETRY_SOURCE_ID_SKINNED_MESH
    );
}

#[test]
fn queue_profile_preserves_direct_cpu_morphed_source_metadata() {
    let profile = MeshDrawQueueProfile::new(
        MeshDrawQueuePhase::Opaque,
        MeshDrawGeometrySource::DynamicCpuMorphedSource,
        Mobility::Dynamic,
        false,
        false,
        false,
    );

    assert!(profile.geometry_source().uses_cpu_morphed_source());
    assert!(!profile
        .geometry_source()
        .uses_cpu_morphed_gpu_skinning_source());
    assert_eq!(
        profile.shader_geometry_source_id(),
        GEOMETRY_SOURCE_ID_STATIC_MESH
    );
}

#[test]
fn queue_profile_maps_gpu_morphed_source_to_morphed_shader_geometry() {
    let profile = MeshDrawQueueProfile::new(
        MeshDrawQueuePhase::Opaque,
        MeshDrawGeometrySource::DynamicGpuMorphedSource,
        Mobility::Dynamic,
        false,
        false,
        false,
    );

    assert!(profile.geometry_source().uses_gpu_morph_payload_source());
    assert_eq!(
        profile.shader_geometry_source_id(),
        GEOMETRY_SOURCE_ID_MORPHED_MESH
    );
}

#[test]
fn queue_profile_maps_gpu_skinned_morphed_source_to_skinned_morphed_shader_geometry() {
    let profile = MeshDrawQueueProfile::new(
        MeshDrawQueuePhase::Opaque,
        MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource,
        Mobility::Dynamic,
        false,
        true,
        false,
    );

    assert!(profile.geometry_source().uses_gpu_morph_payload_source());
    assert_eq!(
        profile.shader_geometry_source_id(),
        GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH
    );
}
