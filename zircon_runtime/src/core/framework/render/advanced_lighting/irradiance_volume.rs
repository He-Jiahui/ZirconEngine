use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::core::framework::render::RenderLayerSet;
use crate::core::math::{Mat4, Real, Vec3};
use crate::core::resource::ResourceId as AssetId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IrradianceVolumeData {
    pub volume_id: u64,
    pub transform: Mat4,
    pub voxels: AssetId,
    pub intensity: Real,
    pub affects_lightmapped_meshes: bool,
    pub priority: i32,
    #[serde(default)]
    pub layer_mask: RenderLayerSet,
}

impl IrradianceVolumeData {
    /// Maps a world-space position into the volume's normalized texture domain.
    /// Authored local volume bounds are `[-0.5, 0.5]` on every axis.
    pub fn world_to_uvw(&self, world_position: Vec3) -> Vec3 {
        self.transform.transform_point3(world_position) + Vec3::splat(0.5)
    }

    pub fn contains_world_position(&self, world_position: Vec3) -> bool {
        let uvw = self.world_to_uvw(world_position);
        uvw.is_finite() && uvw.cmpge(Vec3::ZERO).all() && uvw.cmple(Vec3::ONE).all()
    }
}

pub fn select_irradiance_volume<'a>(
    volumes: &'a [IrradianceVolumeData],
    world_position: Vec3,
    render_layers: &RenderLayerSet,
) -> Option<&'a IrradianceVolumeData> {
    volumes
        .iter()
        .filter(|volume| {
            volume.intensity > 0.0
                && volume.layer_mask.intersects(render_layers)
                && volume.contains_world_position(world_position)
        })
        .max_by(|left, right| irradiance_volume_priority_cmp(left, right))
}

pub fn select_irradiance_volume_for_view<'a>(
    volumes: &'a [IrradianceVolumeData],
    render_layers: &RenderLayerSet,
    visible_world_positions: &[Vec3],
) -> Option<&'a IrradianceVolumeData> {
    let mut selected: Option<&IrradianceVolumeData> = None;
    for volume in volumes {
        if !(volume.intensity > 0.0) || !volume.layer_mask.intersects(render_layers) {
            continue;
        }
        if selected.is_some_and(|current| irradiance_volume_priority_cmp(volume, current).is_lt()) {
            continue;
        }
        if !volume.transform.is_finite()
            || !(volume.transform.determinant().abs() > Real::EPSILON)
            || (!visible_world_positions.is_empty()
                && !visible_world_positions
                    .iter()
                    .copied()
                    .any(|position| volume.contains_world_position(position)))
        {
            continue;
        }
        selected = Some(volume);
    }
    selected
}

fn irradiance_volume_priority_cmp(
    left: &IrradianceVolumeData,
    right: &IrradianceVolumeData,
) -> Ordering {
    left.priority
        .cmp(&right.priority)
        .then_with(|| right.volume_id.cmp(&left.volume_id))
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::core::resource::ResourceId;

    const EPSILON: f32 = 1.0e-5;

    #[test]
    fn render_irrvol_world_to_uvw_roundtrip() {
        let world_from_volume = Mat4::from_scale_rotation_translation(
            Vec3::new(4.0, 2.0, 8.0),
            crate::core::math::Quat::from_rotation_y(0.4),
            Vec3::new(10.0, 3.0, -6.0),
        );
        let volume = volume(1, 0, world_from_volume.inverse());
        let local = Vec3::new(0.25, -0.125, 0.4);
        let world = world_from_volume.transform_point3(local);
        let uvw = volume.world_to_uvw(world);

        assert_vec3_near(uvw, local + Vec3::splat(0.5));
        assert!(volume.contains_world_position(world));
        assert!(!volume.contains_world_position(
            world_from_volume.transform_point3(Vec3::new(0.51, 0.0, 0.0))
        ));
    }

    #[test]
    fn render_irrvol_selection_prefers_priority_inside() {
        let layers = RenderLayerSet::layer(2);
        let volumes = vec![
            volume(9, 1, Mat4::IDENTITY),
            volume(7, 5, Mat4::IDENTITY),
            volume(3, 5, Mat4::IDENTITY),
            IrradianceVolumeData {
                layer_mask: RenderLayerSet::layer(4),
                ..volume(1, 99, Mat4::IDENTITY)
            },
        ];

        let selected = select_irradiance_volume(&volumes, Vec3::ZERO, &layers)
            .expect("an intersecting volume should be selected");
        assert_eq!(selected.volume_id, 3);
        assert!(select_irradiance_volume(&volumes, Vec3::splat(0.75), &layers).is_none());
    }

    #[test]
    fn render_irrvol_view_selection_does_not_require_camera_containment() {
        let layers = RenderLayerSet::layer(2);
        let volumes = vec![volume(7, 4, Mat4::IDENTITY), volume(3, 4, Mat4::IDENTITY)];

        let selected = select_irradiance_volume_for_view(&volumes, &layers, &[])
            .expect("a layer-compatible volume should be selected for per-pixel containment");

        assert_eq!(selected.volume_id, 3);
    }

    #[test]
    fn render_irrvol_view_selection_ignores_unrelated_higher_priority_volume() {
        let layers = RenderLayerSet::layer(2);
        let visible = volume(7, 4, Mat4::IDENTITY);
        let unrelated = volume(3, 99, Mat4::from_translation(Vec3::new(-100.0, 0.0, 0.0)));
        let volumes = [visible, unrelated];

        let selected = select_irradiance_volume_for_view(&volumes, &layers, &[Vec3::ZERO])
            .expect("the volume containing visible scene content should be selected");

        assert_eq!(selected.volume_id, 7);
    }

    fn legacy_select_irradiance_volume_for_view<'a>(
        volumes: &'a [IrradianceVolumeData],
        render_layers: &RenderLayerSet,
        visible_world_positions: &[Vec3],
    ) -> Option<&'a IrradianceVolumeData> {
        volumes
            .iter()
            .filter(|volume| {
                volume.intensity > 0.0
                    && volume.layer_mask.intersects(render_layers)
                    && volume.transform.is_finite()
                    && volume.transform.determinant().abs() > Real::EPSILON
                    && (visible_world_positions.is_empty()
                        || visible_world_positions
                            .iter()
                            .copied()
                            .any(|position| volume.contains_world_position(position)))
            })
            .max_by(|left, right| irradiance_volume_priority_cmp(left, right))
    }

    #[test]
    fn optimization_batch_er_view_selection_preserves_priority_and_tie_order() {
        let layers = RenderLayerSet::layer(2);
        let volumes = [
            volume(9, 5, Mat4::IDENTITY),
            volume(3, 5, Mat4::ZERO),
            volume(9, 5, Mat4::from_scale(Vec3::splat(0.5))),
            volume(12, 4, Mat4::IDENTITY),
        ];
        let positions = [Vec3::ZERO, Vec3::splat(100.0)];

        let legacy = legacy_select_irradiance_volume_for_view(&volumes, &layers, &positions);
        let optimized = select_irradiance_volume_for_view(&volumes, &layers, &positions);

        assert_eq!(
            optimized.map(|volume| volume.volume_id),
            legacy.map(|volume| volume.volume_id)
        );
        assert!(std::ptr::eq(
            optimized.expect("optimized selected volume"),
            &volumes[2]
        ));

        let source = include_str!("irradiance_volume.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("irradiance volume production implementation");
        let view_selection = production
            .split("pub fn select_irradiance_volume_for_view")
            .nth(1)
            .expect("view volume selection");
        let priority_rejection = view_selection
            .find("irradiance_volume_priority_cmp(volume, current).is_lt()")
            .expect("lower-priority rejection");
        let determinant = view_selection
            .find("volume.transform.determinant()")
            .expect("candidate transform validation");
        assert!(priority_rejection < determinant);
    }

    #[test]
    #[ignore = "release-only irradiance volume contender benchmark"]
    fn optimization_batch_er_irradiance_volume_contender_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const SELECTIONS_PER_SAMPLE: usize = 128;
        const VOLUME_COUNT: usize = 256;
        const POSITION_COUNT: usize = 16;

        fn measure_legacy(
            volumes: &[IrradianceVolumeData],
            layers: &RenderLayerSet,
            positions: &[Vec3],
        ) -> u128 {
            let started = Instant::now();
            let mut checksum = 0_u64;
            for _ in 0..SELECTIONS_PER_SAMPLE {
                checksum = checksum.wrapping_add(
                    legacy_select_irradiance_volume_for_view(
                        black_box(volumes),
                        black_box(layers),
                        black_box(positions),
                    )
                    .expect("legacy benchmark selection")
                    .volume_id,
                );
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(
            volumes: &[IrradianceVolumeData],
            layers: &RenderLayerSet,
            positions: &[Vec3],
        ) -> u128 {
            let started = Instant::now();
            let mut checksum = 0_u64;
            for _ in 0..SELECTIONS_PER_SAMPLE {
                checksum = checksum.wrapping_add(
                    select_irradiance_volume_for_view(
                        black_box(volumes),
                        black_box(layers),
                        black_box(positions),
                    )
                    .expect("optimized benchmark selection")
                    .volume_id,
                );
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

        let layers = RenderLayerSet::layer(2);
        let mut volumes = Vec::with_capacity(VOLUME_COUNT);
        volumes.push(volume(1, 10_000, Mat4::IDENTITY));
        volumes.extend((1..VOLUME_COUNT).map(|index| volume(index as u64 + 1, 0, Mat4::IDENTITY)));
        let mut positions = vec![Vec3::splat(100.0); POSITION_COUNT];
        positions[POSITION_COUNT - 1] = Vec3::ZERO;

        for _ in 0..4 {
            black_box(measure_legacy(&volumes, &layers, &positions));
            black_box(measure_optimized(&volumes, &layers, &positions));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&volumes, &layers, &positions));
                optimized_samples.push(measure_optimized(&volumes, &layers, &positions));
            } else {
                optimized_samples.push(measure_optimized(&volumes, &layers, &positions));
                legacy_samples.push(measure_legacy(&volumes, &layers, &positions));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME450_IRRADIANCE_CONTENDER_FILTER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             selections_per_sample={SELECTIONS_PER_SAMPLE} volume_count={VOLUME_COUNT} \
             position_count={POSITION_COUNT} pair_order=alternating_legacy_even \
             legacy_expensive_candidates_per_selection={VOLUME_COUNT} \
             optimized_expensive_candidates_per_selection=1 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(20),
            "irradiance contender filtering must reduce P95 by at least 80%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn volume(volume_id: u64, priority: i32, transform: Mat4) -> IrradianceVolumeData {
        IrradianceVolumeData {
            volume_id,
            transform,
            voxels: ResourceId::from_stable_label(&format!(
                "runtime://irradiance-volume/{volume_id}"
            )),
            intensity: 1.0,
            affects_lightmapped_meshes: false,
            priority,
            layer_mask: RenderLayerSet::layer(2),
        }
    }

    fn assert_vec3_near(actual: Vec3, expected: Vec3) {
        assert!((actual.x - expected.x).abs() <= EPSILON);
        assert!((actual.y - expected.y).abs() <= EPSILON);
        assert!((actual.z - expected.z).abs() <= EPSILON);
    }
}
