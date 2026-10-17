use std::sync::Arc;

use crate::graphics::scene::gpu_scene::GpuMorphWeight;

use crate::asset::ModelPrimitiveAsset;

use super::{
    hzb_geometry_bounds_are_temporally_stable, morph_bounds_are_temporally_stable,
    PendingMeshGeometry, PendingMorphPayload,
};

#[test]
fn equal_current_and_previous_morph_weights_are_temporally_stable() {
    let weights = vec![GpuMorphWeight::new(0.5)];
    let payload = Arc::new(PendingMorphPayload {
        vertex_count: 1,
        target_count: 1,
        deltas: Vec::new(),
        weights: weights.clone(),
        previous_weights: weights,
    });

    assert!(morph_bounds_are_temporally_stable(Some(&payload)));
}

#[test]
fn changed_morph_weights_force_temporal_hzb_visibility() {
    let payload = PendingMorphPayload {
        vertex_count: 1,
        target_count: 1,
        deltas: Vec::new(),
        weights: vec![GpuMorphWeight::new(0.75)],
        previous_weights: vec![GpuMorphWeight::new(0.25)],
    };

    assert!(!morph_bounds_are_temporally_stable(Some(&payload)));
}

#[test]
fn cpu_morphed_geometry_forces_temporal_hzb_visibility() {
    let primitive = Arc::new(ModelPrimitiveAsset {
        vertices: Vec::new(),
        indices: Vec::new(),
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    });

    assert!(!hzb_geometry_bounds_are_temporally_stable(
        &PendingMeshGeometry::CpuMorphed(primitive.clone())
    ));
    assert!(hzb_geometry_bounds_are_temporally_stable(
        &PendingMeshGeometry::Dynamic(primitive)
    ));
}

#[test]
fn cloned_deformation_records_share_payload_until_last_user_exits() {
    let source = Arc::new(geometry_test_primitive(64));
    let weak = Arc::downgrade(&source);
    let vertices = source.vertices.as_ptr();
    let indices = source.indices.as_ptr();
    let draw = PendingMeshGeometry::Dynamic(source.clone());
    let cloned = draw.clone();
    let morphed = PendingMeshGeometry::CpuMorphed(source.clone());
    let shader_source = super::PendingSkinnedGpuSource::CpuMorphed {
        primitive: source.clone(),
        morph_shape_signature: 7,
    };
    let cloned_shader_source = shader_source.clone();
    for geometry in [&draw, &cloned, &morphed] {
        let (PendingMeshGeometry::Dynamic(primitive) | PendingMeshGeometry::CpuMorphed(primitive)) =
            geometry
        else {
            panic!("expected CPU geometry");
        };
        assert!(Arc::ptr_eq(primitive, &source));
        assert_eq!(primitive.vertices.as_ptr(), vertices);
        assert_eq!(primitive.indices.as_ptr(), indices);
    }
    let super::PendingSkinnedGpuSource::CpuMorphed { primitive, .. } = &cloned_shader_source else {
        panic!("expected CPU morph shader source");
    };
    assert!(Arc::ptr_eq(primitive, &source));
    drop((source, draw, cloned, morphed, shader_source));
    assert!(weak.upgrade().is_some());
    drop(cloned_shader_source);
    assert!(weak.upgrade().is_none());
}

fn geometry_test_primitive(vertex_count: usize) -> ModelPrimitiveAsset {
    ModelPrimitiveAsset {
        vertices: vec![
            crate::asset::MeshVertex::new(
                crate::core::math::Vec3::X,
                crate::core::math::Vec3::Z,
                crate::core::math::Vec2::ZERO,
            );
            vertex_count
        ],
        indices: vec![0; vertex_count * 2],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    }
}

#[cfg(feature = "memory-validation")]
#[test]
fn memory_profile_dynamic_geometry_draws() {
    use crate::memory_validation::{live_bytes, measure};
    const DRAWS: usize = 32;
    let source = Arc::new(geometry_test_primitive(65_536));
    let baseline = live_bytes();
    // Clone-stress probe; the current raster builder normally emits one record.
    let (legacy, old) = measure(|| {
        (0..DRAWS)
            .map(|_| source.as_ref().clone())
            .collect::<Vec<_>>()
    });
    old.report("32_raster_records_65536_vertices", "owned_payload_baseline");
    assert_eq!(legacy[0].vertices, source.vertices);
    drop(legacy);
    assert_eq!(live_bytes(), baseline);
    let (shared, new) = measure(|| {
        let geometry = PendingMeshGeometry::Dynamic(source.clone());
        vec![geometry; DRAWS]
    });
    new.report("32_raster_records_65536_vertices", "shared_geometry");
    assert!(new.live_bytes < old.live_bytes / 1000);
    let PendingMeshGeometry::Dynamic(first) = &shared[0] else {
        unreachable!()
    };
    let PendingMeshGeometry::Dynamic(last) = &shared[DRAWS - 1] else {
        unreachable!()
    };
    assert!(Arc::ptr_eq(first, last));
    assert_eq!(first.vertices, source.vertices);
    drop(shared);
    assert_eq!(live_bytes(), baseline);
}
