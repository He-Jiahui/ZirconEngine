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
mod optimization_batch_gz_runtime581_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_gz_runtime581_texture_probe_selection_preserves_presence() {
        assert!(has_valid_texture_planar_probe([true, false, false]));
        assert!(has_valid_texture_planar_probe([false, true, false]));
        assert!(!has_valid_texture_planar_probe([false, false, false]));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gz_runtime581_texture_probe_selection_short_circuit_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 8_192;
        const CANDIDATES: usize = 2_048;
        let mut validities = vec![false; CANDIDATES];
        validities[0] = true;
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &validities, ITERATIONS));
                optimized.push(measure(true, &validities, ITERATIONS));
            } else {
                optimized.push(measure(true, &validities, ITERATIONS));
                legacy.push(measure(false, &validities, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME581_TEXTURE_PROBE_SELECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} candidates={CANDIDATES} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "texture probe presence short-circuit must improve P95 by at least 50%"
        );
    }

    fn measure(optimized: bool, validities: &[bool], iterations: usize) -> u128 {
        let started = Instant::now();
        let mut selected = 0_u64;
        for _ in 0..iterations {
            let result = if optimized {
                has_valid_texture_planar_probe(black_box(validities).iter().copied())
            } else {
                black_box(validities)
                    .iter()
                    .copied()
                    .enumerate()
                    .filter(|(_, is_valid)| *is_valid)
                    .min_by_key(|(probe_id, _)| *probe_id)
                    .is_some()
            };
            selected += u64::from(result);
        }
        black_box(selected);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
