use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::framework::render::RenderLayerSet;
use crate::core::math::{Quat, Real, Vec3};
use crate::core::resource::ResourceId;

const PROBE_DIRECTION_EPSILON: Real = 0.000001;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProbeInfluenceShape {
    Box {
        half_extents: Vec3,
        blend_distance: Real,
    },
    Sphere {
        radius: Real,
        blend_distance: Real,
    },
}

impl ProbeInfluenceShape {
    pub fn box_shape(
        half_extents: Vec3,
        blend_distance: Real,
    ) -> Result<Self, ReflectionProbeValidationError> {
        if !half_extents.is_finite() || half_extents.min_element() <= 0.0 {
            return Err(ReflectionProbeValidationError::InvalidBoxHalfExtents {
                half_extents: half_extents.to_array(),
            });
        }
        validate_blend_distance(blend_distance, half_extents.min_element())?;
        Ok(Self::Box {
            half_extents,
            blend_distance,
        })
    }

    pub fn sphere(
        radius: Real,
        blend_distance: Real,
    ) -> Result<Self, ReflectionProbeValidationError> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(ReflectionProbeValidationError::InvalidSphereRadius { radius });
        }
        validate_blend_distance(blend_distance, radius)?;
        Ok(Self::Sphere {
            radius,
            blend_distance,
        })
    }

    pub const fn blend_distance(self) -> Real {
        match self {
            Self::Box { blend_distance, .. } | Self::Sphere { blend_distance, .. } => {
                blend_distance
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectionProbeData {
    probe_id: u64,
    position: Vec3,
    rotation: Quat,
    shape: ProbeInfluenceShape,
    box_projection: bool,
    projection_half_extents: Vec3,
    baked_cubemap: Option<ResourceId>,
    intensity: Real,
    priority: i32,
    layer_mask: RenderLayerSet,
}

impl ReflectionProbeData {
    pub fn try_new(
        probe_id: u64,
        position: Vec3,
        rotation: Quat,
        shape: ProbeInfluenceShape,
        projection_half_extents: Vec3,
    ) -> Result<Self, ReflectionProbeValidationError> {
        if !position.is_finite() {
            return Err(ReflectionProbeValidationError::InvalidPosition {
                position: position.to_array(),
            });
        }
        if !rotation.is_finite() || rotation.length_squared() <= PROBE_DIRECTION_EPSILON {
            return Err(ReflectionProbeValidationError::InvalidRotation {
                rotation: rotation.to_array(),
            });
        }
        if !projection_half_extents.is_finite() || projection_half_extents.min_element() <= 0.0 {
            return Err(
                ReflectionProbeValidationError::InvalidProjectionHalfExtents {
                    half_extents: projection_half_extents.to_array(),
                },
            );
        }
        Ok(Self {
            probe_id,
            position,
            rotation: rotation.normalize(),
            shape,
            box_projection: false,
            projection_half_extents,
            baked_cubemap: None,
            intensity: 1.0,
            priority: 0,
            layer_mask: RenderLayerSet::default(),
        })
    }

    pub const fn probe_id(&self) -> u64 {
        self.probe_id
    }

    pub const fn position(&self) -> Vec3 {
        self.position
    }

    pub const fn rotation(&self) -> Quat {
        self.rotation
    }

    pub const fn shape(&self) -> ProbeInfluenceShape {
        self.shape
    }

    pub const fn box_projection(&self) -> bool {
        self.box_projection
    }

    pub const fn projection_half_extents(&self) -> Vec3 {
        self.projection_half_extents
    }

    pub const fn baked_cubemap(&self) -> Option<ResourceId> {
        self.baked_cubemap
    }

    pub const fn intensity(&self) -> Real {
        self.intensity
    }

    pub const fn priority(&self) -> i32 {
        self.priority
    }

    pub const fn layer_mask(&self) -> &RenderLayerSet {
        &self.layer_mask
    }

    pub const fn with_box_projection(mut self, enabled: bool) -> Self {
        self.box_projection = enabled;
        self
    }

    pub const fn with_baked_cubemap(mut self, cubemap: Option<ResourceId>) -> Self {
        self.baked_cubemap = cubemap;
        self
    }

    pub fn try_with_intensity(
        mut self,
        intensity: Real,
    ) -> Result<Self, ReflectionProbeValidationError> {
        if !intensity.is_finite() || intensity < 0.0 {
            return Err(ReflectionProbeValidationError::InvalidIntensity { intensity });
        }
        self.intensity = intensity;
        Ok(self)
    }

    pub const fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_layer_mask(mut self, layer_mask: RenderLayerSet) -> Self {
        self.layer_mask = layer_mask;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionProbeBlendEntry {
    pub probe_index: usize,
    pub weight: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionProbeBlend {
    pub primary: Option<ReflectionProbeBlendEntry>,
    pub secondary: Option<ReflectionProbeBlendEntry>,
    pub skybox_weight: Real,
}

impl ReflectionProbeBlend {
    pub const fn skybox_only() -> Self {
        Self {
            primary: None,
            secondary: None,
            skybox_weight: 1.0,
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ReflectionProbeValidationError {
    #[error("reflection probe position must be finite, got {position:?}")]
    InvalidPosition { position: [Real; 3] },
    #[error("reflection probe rotation must be finite and nonzero, got {rotation:?}")]
    InvalidRotation { rotation: [Real; 4] },
    #[error("reflection probe box half extents must be finite and positive, got {half_extents:?}")]
    InvalidBoxHalfExtents { half_extents: [Real; 3] },
    #[error("reflection probe sphere radius must be finite and positive, got {radius}")]
    InvalidSphereRadius { radius: Real },
    #[error(
        "reflection probe blend distance must be finite and within [0, {maximum}], got {blend_distance}"
    )]
    InvalidBlendDistance { blend_distance: Real, maximum: Real },
    #[error(
        "reflection probe projection half extents must be finite and positive, got {half_extents:?}"
    )]
    InvalidProjectionHalfExtents { half_extents: [Real; 3] },
    #[error("reflection probe intensity must be finite and nonnegative, got {intensity}")]
    InvalidIntensity { intensity: Real },
}

pub fn reflection_probe_influence_weight(
    probe: &ReflectionProbeData,
    world_position: Vec3,
) -> Real {
    if !world_position.is_finite() {
        return 0.0;
    }
    let position_delta = world_position - probe.position;
    match probe.shape {
        ProbeInfluenceShape::Box {
            half_extents,
            blend_distance,
        } => {
            let local_position = probe.rotation.conjugate() * position_delta;
            let edge_distance = (half_extents - local_position.abs()).min_element();
            boundary_weight(edge_distance, blend_distance)
        }
        ProbeInfluenceShape::Sphere {
            radius,
            blend_distance,
        } => {
            let distance_squared = position_delta.length_squared();
            if radius > 0.0 && distance_squared >= radius * radius {
                0.0
            } else {
                boundary_weight(radius - distance_squared.sqrt(), blend_distance)
            }
        }
    }
}

pub fn reflection_probe_box_project_direction(
    probe: &ReflectionProbeData,
    world_position: Vec3,
    reflection_direction: Vec3,
) -> Vec3 {
    let fallback = normalize_or_zero(reflection_direction);
    if !probe.box_projection || !world_position.is_finite() || fallback == Vec3::ZERO {
        return fallback;
    }

    let world_to_probe = probe.rotation.conjugate();
    let local_position = world_to_probe * (world_position - probe.position);
    let local_direction = world_to_probe * fallback;
    let extent = probe.projection_half_extents;
    let mut distance = Real::INFINITY;
    for axis in 0..3 {
        let direction = local_direction[axis];
        if direction.abs() <= PROBE_DIRECTION_EPSILON {
            continue;
        }
        let plane = if direction > 0.0 {
            extent[axis]
        } else {
            -extent[axis]
        };
        let axis_distance = (plane - local_position[axis]) / direction;
        if axis_distance >= 0.0 {
            distance = distance.min(axis_distance);
        }
    }
    if !distance.is_finite() {
        return fallback;
    }
    let local_hit = local_position + local_direction * distance;
    probe.rotation * local_hit
}

pub fn select_reflection_probe_blend(
    probes: &[ReflectionProbeData],
    world_position: Vec3,
    render_layers: &RenderLayerSet,
) -> ReflectionProbeBlend {
    let mut primary = None;
    let mut secondary = None;
    for (probe_index, probe) in probes.iter().enumerate() {
        if probe.baked_cubemap.is_none()
            || probe.intensity <= 0.0
            || !probe.layer_mask.intersects(render_layers)
        {
            continue;
        }
        let weight = reflection_probe_influence_weight(probe, world_position);
        if weight <= 0.0 {
            continue;
        }
        let candidate = (probe_index, weight, probe.priority, probe.probe_id);
        if primary.is_none_or(|current| reflection_probe_candidate_precedes(candidate, current)) {
            secondary = primary;
            primary = Some(candidate);
        } else if secondary
            .is_none_or(|current| reflection_probe_candidate_precedes(candidate, current))
        {
            secondary = Some(candidate);
        }
    }

    let Some((primary_index, primary_weight, _, _)) = primary else {
        return ReflectionProbeBlend::skybox_only();
    };
    let primary_weight = primary_weight.clamp(0.0, 1.0);
    let primary = Some(ReflectionProbeBlendEntry {
        probe_index: primary_index,
        weight: primary_weight,
    });
    let secondary = secondary.map(|candidate| ReflectionProbeBlendEntry {
        probe_index: candidate.0,
        weight: candidate.1.clamp(0.0, 1.0 - primary_weight),
    });
    let secondary_weight = secondary.map_or(0.0, |entry| entry.weight);
    ReflectionProbeBlend {
        primary,
        secondary,
        skybox_weight: (1.0 - primary_weight - secondary_weight).max(0.0),
    }
}

fn reflection_probe_candidate_precedes(
    left: (usize, Real, i32, u64),
    right: (usize, Real, i32, u64),
) -> bool {
    right
        .1
        .total_cmp(&left.1)
        .then_with(|| right.2.cmp(&left.2))
        .then_with(|| left.3.cmp(&right.3))
        .is_lt()
}

fn validate_blend_distance(
    blend_distance: Real,
    maximum: Real,
) -> Result<(), ReflectionProbeValidationError> {
    if blend_distance.is_finite() && blend_distance >= 0.0 && blend_distance <= maximum {
        return Ok(());
    }
    Err(ReflectionProbeValidationError::InvalidBlendDistance {
        blend_distance,
        maximum,
    })
}

fn boundary_weight(edge_distance: Real, blend_distance: Real) -> Real {
    if edge_distance <= 0.0 {
        return 0.0;
    }
    if blend_distance <= PROBE_DIRECTION_EPSILON {
        return 1.0;
    }
    (edge_distance / blend_distance).clamp(0.0, 1.0)
}

fn normalize_or_zero(value: Vec3) -> Vec3 {
    if !value.is_finite() || value.length_squared() <= PROBE_DIRECTION_EPSILON {
        return Vec3::ZERO;
    }
    value.normalize()
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn reflection_probe_blend_selects_top_two_without_candidate_vec_or_sort() {
        let source = include_str!("reflection_probe.rs");
        let selection = source
            .split("pub fn select_reflection_probe_blend")
            .nth(1)
            .and_then(|text| text.split("fn validate_blend_distance").next())
            .expect("read reflection probe selection helper");

        assert!(
            !selection.contains("collect::<Vec") && !selection.contains("sort_by"),
            "per-position reflection probe selection must retain only the best two candidates instead of allocating and sorting every eligible probe"
        );

        let probes = [probe(20, 0), probe(10, 0), probe(30, 2)];
        let blend = select_reflection_probe_blend(&probes, Vec3::ZERO, &RenderLayerSet::default());

        assert_eq!(blend.primary.map(|entry| entry.probe_index), Some(2));
        assert_eq!(blend.secondary.map(|entry| entry.probe_index), Some(1));
    }

    #[test]
    fn optimization_batch_dz_sphere_weight_uses_squared_distance_broad_phase() {
        let source = include_str!("reflection_probe.rs");
        let weight = source
            .split("pub fn reflection_probe_influence_weight")
            .nth(1)
            .and_then(|text| {
                text.split("pub fn reflection_probe_box_project_direction")
                    .next()
            })
            .expect("read reflection probe influence helper");

        let position_delta = weight
            .find("let position_delta = world_position - probe.position;")
            .expect("probe influence must retain the world-space center delta");
        let box_branch = weight
            .find("ProbeInfluenceShape::Box {")
            .expect("probe influence must retain box weighting");
        let box_rotation = weight
            .find("let local_position = probe.rotation.conjugate() * position_delta;")
            .expect("box weighting must retain local-space rotation");
        let sphere_branch = weight
            .find("ProbeInfluenceShape::Sphere {")
            .expect("probe influence must retain sphere weighting");
        let sphere_distance_squared = weight
            .find("let distance_squared = position_delta.length_squared();")
            .expect("sphere weighting must compute squared center distance once");
        let sphere_rejection = weight
            .find("if radius > 0.0 && distance_squared >= radius * radius")
            .expect("far sphere probes must be rejected before square root");
        let sphere_distance = weight
            .find("boundary_weight(radius - distance_squared.sqrt(), blend_distance)")
            .expect("sphere weighting must use its rotation-invariant center distance");

        assert!(
            position_delta < box_branch
                && box_branch < box_rotation
                && box_rotation < sphere_branch
                && sphere_branch < sphere_distance_squared
                && sphere_distance_squared < sphere_rejection
                && sphere_rejection < sphere_distance,
            "only box influences may rotate the world-space center delta"
        );
    }

    #[test]
    fn optimization_batch_dz_far_sphere_probe_keeps_zero_boundary_weight() {
        let probe = probe(40, 0);

        assert_eq!(
            reflection_probe_influence_weight(&probe, Vec3::new(4.0, 0.0, 0.0)),
            0.0
        );
        assert_eq!(
            reflection_probe_influence_weight(&probe, Vec3::new(40.0, 0.0, 0.0)),
            0.0
        );
        assert_eq!(
            reflection_probe_influence_weight(&probe, Vec3::new(3.5, 0.0, 0.0)),
            0.5
        );
    }

    #[test]
    #[ignore = "release-only far sphere reflection probe benchmark"]
    fn optimization_batch_dz_far_sphere_probe_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const PASSES_PER_SAMPLE: usize = 32;
        const PROBES_PER_PASS: usize = 2_048;

        fn legacy_weight(probe: &ReflectionProbeData, world_position: Vec3) -> Real {
            let position_delta = world_position - probe.position;
            match probe.shape {
                ProbeInfluenceShape::Box {
                    half_extents,
                    blend_distance,
                } => {
                    let local_position = probe.rotation.conjugate() * position_delta;
                    let edge_distance = (half_extents - local_position.abs()).min_element();
                    boundary_weight(edge_distance, blend_distance)
                }
                ProbeInfluenceShape::Sphere {
                    radius,
                    blend_distance,
                } => boundary_weight(radius - position_delta.length(), blend_distance),
            }
        }

        fn measure_legacy(probes: &[ReflectionProbeData]) -> u128 {
            let world_position = black_box(Vec3::new(16_384.0, 8_192.0, -4_096.0));
            let started = Instant::now();
            let mut checksum = 0.0;
            for _ in 0..PASSES_PER_SAMPLE {
                for probe in probes {
                    checksum += legacy_weight(black_box(probe), world_position);
                }
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(probes: &[ReflectionProbeData]) -> u128 {
            let world_position = black_box(Vec3::new(16_384.0, 8_192.0, -4_096.0));
            let started = Instant::now();
            let mut checksum = 0.0;
            for _ in 0..PASSES_PER_SAMPLE {
                for probe in probes {
                    checksum += reflection_probe_influence_weight(black_box(probe), world_position);
                }
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let probes = (0..PROBES_PER_PASS)
            .map(|index| {
                ReflectionProbeData::try_new(
                    index as u64,
                    Vec3::new(index as Real * 0.125, 0.0, 0.0),
                    Quat::IDENTITY,
                    ProbeInfluenceShape::sphere(4.0, 1.0).expect("valid benchmark sphere"),
                    Vec3::splat(4.0),
                )
                .expect("valid benchmark probe")
            })
            .collect::<Vec<_>>();

        for _ in 0..4 {
            black_box(measure_legacy(&probes));
            black_box(measure_optimized(&probes));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&probes));
                optimized_samples.push(measure_optimized(&probes));
            } else {
                optimized_samples.push(measure_optimized(&probes));
                legacy_samples.push(measure_legacy(&probes));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "RUNTIME434_FAR_SPHERE_PROBE_BROAD_PHASE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
passes_per_sample={PASSES_PER_SAMPLE} probes_per_pass={PROBES_PER_PASS} \
pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 \
legacy_sqrt_per_sample={} optimized_sqrt_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            PASSES_PER_SAMPLE * PROBES_PER_PASS,
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "far sphere broad phase must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn probe(probe_id: u64, priority: i32) -> ReflectionProbeData {
        ReflectionProbeData::try_new(
            probe_id,
            Vec3::ZERO,
            Quat::IDENTITY,
            ProbeInfluenceShape::sphere(4.0, 1.0).expect("valid sphere"),
            Vec3::splat(4.0),
        )
        .expect("valid probe")
        .with_baked_cubemap(Some(ResourceId::from_stable_label(&format!(
            "res://probe/{probe_id}.zcube"
        ))))
        .with_priority(priority)
    }
}
