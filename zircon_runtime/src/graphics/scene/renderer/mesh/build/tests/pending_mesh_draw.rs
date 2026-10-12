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
    let primitive = ModelPrimitiveAsset {
        vertices: Vec::new(),
        indices: Vec::new(),
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(!hzb_geometry_bounds_are_temporally_stable(
        &PendingMeshGeometry::CpuMorphed(primitive.clone())
    ));
    assert!(hzb_geometry_bounds_are_temporally_stable(
        &PendingMeshGeometry::Dynamic(primitive)
    ));
}
