use crate::core::framework::render::{
    derive_planar_reflection_camera, PlanarReflectionProbeData, ProbeInfluenceShape,
    ReflectionProbeData, RenderCameraTarget,
};
use crate::core::math::{view_matrix, Vec3};
use crate::core::resource::ResourceId;
use crate::graphics::types::ViewportRenderFrame;

use super::capacity::PLANAR_REFLECTION_TEXTURE_SIZE;
use super::gpu_layout::GpuPlanarReflection;

pub(super) struct ReflectionProbeCandidate<'a> {
    pub(super) probe: &'a ReflectionProbeData,
    pub(super) cubemap: ResourceId,
    pub(super) revision: Option<u64>,
    pub(super) distance: f32,
    pub(super) extraction_order: usize,
}

pub(super) fn reflection_probe_candidate_order(
    left: &ReflectionProbeCandidate<'_>,
    right: &ReflectionProbeCandidate<'_>,
) -> std::cmp::Ordering {
    left.distance
        .total_cmp(&right.distance)
        .then_with(|| right.probe.priority().cmp(&left.probe.priority()))
        .then_with(|| left.probe.probe_id().cmp(&right.probe.probe_id()))
        .then_with(|| left.cubemap.cmp(&right.cubemap))
        .then_with(|| left.extraction_order.cmp(&right.extraction_order))
}

pub(super) fn selected_planar_reflection_params(
    frame: &ViewportRenderFrame,
) -> Option<GpuPlanarReflection> {
    let camera_layers = frame.extract.view.selected_camera_layers();
    let planar_probes = &frame.extract.lighting.advanced_lighting.planar_probes;
    match frame.extract.view.selected_camera_target() {
        RenderCameraTarget::Texture(target) => has_valid_texture_planar_probe(
            planar_probes
                .iter()
                .filter(|probe| probe.capture_target() == Some(*target))
                .map(|probe| planar_gpu_params(frame, probe).is_some()),
        )
        .then_some(GpuPlanarReflection::default()),
        _ => planar_probes
            .iter()
            .filter(|probe| {
                probe.capture_target().is_some() && probe.layer_mask.intersects(camera_layers)
            })
            .filter_map(|probe| {
                planar_gpu_params(frame, probe).map(|params| (probe.probe_id, params))
            })
            .min_by_key(|(probe_id, _)| *probe_id)
            .map(|(_, params)| params),
    }
}

fn has_valid_texture_planar_probe<I>(validities: I) -> bool
where
    I: IntoIterator<Item = bool>,
{
    validities.into_iter().any(|is_valid| is_valid)
}

fn planar_gpu_params(
    frame: &ViewportRenderFrame,
    probe: &PlanarReflectionProbeData,
) -> Option<GpuPlanarReflection> {
    let target = probe.capture_target()?;
    let main_camera = frame.extract.view.selected_camera_descriptor()?;
    let reflected = derive_planar_reflection_camera(main_camera, probe, target)?;
    let projection = reflected.camera.projection_override?;
    let clip_from_world = projection * view_matrix(reflected.camera.transform);
    let determinant = probe.plane_transform.determinant();
    if !determinant.is_finite() || determinant.abs() <= 1.0e-6 {
        return None;
    }
    let local_from_world = probe.plane_transform.inverse();
    let resolution = probe.resolution.clamp(1, PLANAR_REFLECTION_TEXTURE_SIZE);
    let mip_count = u32::BITS - resolution.leading_zeros();
    let scale = resolution as f32 / PLANAR_REFLECTION_TEXTURE_SIZE as f32;
    Some(GpuPlanarReflection {
        clip_from_world: clip_from_world.to_cols_array_2d(),
        local_from_world: local_from_world.to_cols_array_2d(),
        bounds_min: probe.bounds_min.extend(0.0).to_array(),
        bounds_max: probe.bounds_max.extend(0.0).to_array(),
        sample_params: [scale, scale, mip_count as f32, 1.0],
    })
}

pub(super) fn probe_distance_to_influence(
    probe: &ReflectionProbeData,
    world_position: Vec3,
) -> f32 {
    let position_delta = world_position - probe.position();
    match probe.shape() {
        ProbeInfluenceShape::Box { half_extents, .. } => {
            let local = probe.rotation().conjugate() * position_delta;
            (local.abs() - half_extents).max(Vec3::ZERO).length()
        }
        ProbeInfluenceShape::Sphere { radius, .. } => (position_delta.length() - radius).max(0.0),
    }
}

#[cfg(test)]
#[path = "tests/selection_optimization_batch_gz_runtime581_tests.rs"]
mod optimization_batch_gz_runtime581_tests;
