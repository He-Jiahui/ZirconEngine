use crate::core::framework::render::ViewportCameraSnapshot;

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn motion_vector_camera(
        &self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&ViewportCameraSnapshot> {
        self.motion_vector_cameras.get(key)
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_motion_vector_camera(
        &mut self,
        key: &ViewportCameraHistoryKey,
        camera: ViewportCameraSnapshot,
    ) {
        if let Some(previous) = self.motion_vector_cameras.get_mut(key) {
            *previous = camera;
        } else {
            self.motion_vector_cameras.insert(key.clone(), camera);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::{
        CameraRenderDescriptor, RenderCameraTarget, RenderLayerSet, RenderViewportDescriptor,
        RenderViewportRect, ViewportCameraSnapshot,
    };
    use crate::core::math::{Transform, UVec2, Vec3};

    use super::super::camera_history_key::ViewportCameraHistoryKey;
    use super::ViewportRecord;

    #[test]
    fn viewport_record_keeps_motion_vector_camera_per_camera_key() {
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let left_key = camera_key(1, UVec2::ZERO);
        let right_key = camera_key(1, UVec2::new(32, 0));
        let mut left_camera = ViewportCameraSnapshot::default();
        left_camera.transform = Transform::from_translation(Vec3::new(-1.0, 0.0, 4.0));
        let mut right_camera = ViewportCameraSnapshot::default();
        right_camera.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 4.0));
        let left_transform = left_camera.transform;
        let right_transform = right_camera.transform;

        record.replace_motion_vector_camera(&left_key, left_camera);
        record.replace_motion_vector_camera(&right_key, right_camera);

        assert_eq!(
            record
                .motion_vector_camera(&left_key)
                .map(|camera| camera.transform),
            Some(left_transform)
        );
        assert_eq!(
            record
                .motion_vector_camera(&right_key)
                .map(|camera| camera.transform),
            Some(right_transform)
        );
    }

    #[test]
    fn optimization_batch_fm_runtime469_borrowed_camera_key_preserves_replacement_semantics() {
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let key = shared_layer_camera_key();
        let mut first = ViewportCameraSnapshot::default();
        first.transform = Transform::from_translation(Vec3::new(-1.0, 0.0, 4.0));
        let mut replacement = ViewportCameraSnapshot::default();
        replacement.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 4.0));
        let replacement_transform = replacement.transform;

        record.replace_motion_vector_camera(&key, first);
        record.replace_motion_vector_camera(&key, replacement);

        assert_eq!(record.motion_vector_cameras.len(), 1);
        assert_eq!(
            record
                .motion_vector_camera(&key)
                .map(|camera| camera.transform),
            Some(replacement_transform)
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fm_runtime469_borrowed_camera_key_benchmark() {
        const REPLACEMENTS_PER_SAMPLE: usize = 262_144;
        const SAMPLE_PAIRS: usize = 17;

        let key = shared_layer_camera_key();
        for _ in 0..4 {
            black_box(measure_legacy(&key, REPLACEMENTS_PER_SAMPLE));
            black_box(measure_optimized(&key, REPLACEMENTS_PER_SAMPLE));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&key, REPLACEMENTS_PER_SAMPLE));
                optimized_samples.push(measure_optimized(&key, REPLACEMENTS_PER_SAMPLE));
            } else {
                optimized_samples.push(measure_optimized(&key, REPLACEMENTS_PER_SAMPLE));
                legacy_samples.push(measure_legacy(&key, REPLACEMENTS_PER_SAMPLE));
            }
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME469_BORROWED_MOTION_CAMERA_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} replacements_per_sample={REPLACEMENTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=15",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(85) / 100,
            "borrowed motion camera key replacement must reduce P95 by at least 15%"
        );
    }

    fn measure_legacy(key: &ViewportCameraHistoryKey, replacement_count: usize) -> u128 {
        let mut cameras = HashMap::from([(key.clone(), 0_u64)]);
        let started = Instant::now();
        for revision in 0..replacement_count as u64 {
            cameras.insert(black_box(key).clone(), black_box(revision));
        }
        black_box(cameras);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(key: &ViewportCameraHistoryKey, replacement_count: usize) -> u128 {
        let mut cameras = HashMap::from([(key.clone(), 0_u64)]);
        let started = Instant::now();
        for revision in 0..replacement_count as u64 {
            if let Some(camera) = cameras.get_mut(black_box(key)) {
                *camera = black_box(revision);
            } else {
                cameras.insert(key.clone(), revision);
            }
        }
        black_box(cameras);
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn shared_layer_camera_key() -> ViewportCameraHistoryKey {
        let mut descriptor =
            CameraRenderDescriptor::from_camera_payload(Some(7), ViewportCameraSnapshot::default());
        descriptor.culling_mask = RenderLayerSet::from_layers([0, 40, 80, 120, 160]);
        descriptor.volume_mask = RenderLayerSet::from_layers([1, 41, 81, 121, 161]);
        ViewportCameraHistoryKey::from_camera(&descriptor)
    }

    fn camera_key(entity: u64, position: UVec2) -> ViewportCameraHistoryKey {
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        );
        descriptor.target = RenderCameraTarget::PrimarySurface;
        descriptor.viewport_rect = Some(RenderViewportRect::new(position, UVec2::new(32, 64)));
        ViewportCameraHistoryKey::from_camera(&descriptor)
    }
}
