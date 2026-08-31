use std::{collections::HashMap, sync::Arc};

use super::gpu_scene::GpuScene;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuScenePrevMorphWeightsRollReport {
    pub(crate) current_weight_state_count: usize,
    pub(crate) previous_weight_state_count: usize,
    pub(crate) removed_previous_weight_state_count: usize,
}

impl GpuScene {
    pub(crate) fn previous_morph_weights(&self, stable_instance_key: u64) -> Option<&[f32]> {
        self.previous_morph_weights
            .get(&stable_instance_key)
            .map(Arc::as_ref)
    }

    pub(crate) fn stage_current_morph_weights(
        &mut self,
        stable_instance_key: u64,
        weights: Option<&[f32]>,
    ) {
        if let Some(weights) = weights.filter(|weights| !weights.is_empty()) {
            if self
                .current_morph_weights
                .get(&stable_instance_key)
                .is_some_and(|current| current.as_ref() == weights)
            {
                return;
            }
            self.current_morph_weights
                .insert(stable_instance_key, Arc::from(weights));
        } else {
            self.current_morph_weights.remove(&stable_instance_key);
        }
    }

    pub(crate) fn roll_prev_morph_weights_after_success(
        &mut self,
    ) -> GpuScenePrevMorphWeightsRollReport {
        let removed_previous_weight_state_count = roll_previous_morph_weight_snapshots(
            &mut self.previous_morph_weights,
            &self.current_morph_weights,
        );

        let previous_weight_state_count = self.previous_morph_weights.len();
        GpuScenePrevMorphWeightsRollReport {
            current_weight_state_count: self.current_morph_weights.len(),
            previous_weight_state_count,
            removed_previous_weight_state_count,
        }
    }
}

fn roll_previous_morph_weight_snapshots(
    previous: &mut HashMap<u64, Arc<[f32]>>,
    current: &HashMap<u64, Arc<[f32]>>,
) -> usize {
    let mut removed = 0;
    previous.retain(|key, previous_weights| {
        let Some(current_weights) = current.get(key) else {
            removed += 1;
            return false;
        };
        if !Arc::ptr_eq(previous_weights, current_weights) {
            *previous_weights = Arc::clone(current_weights);
        }
        true
    });

    if previous.len() < current.len() {
        previous.reserve(current.len() - previous.len());
        for (key, weights) in current {
            if let std::collections::hash_map::Entry::Vacant(entry) = previous.entry(*key) {
                entry.insert(Arc::clone(weights));
            }
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::graphics::scene::gpu_scene::GpuScene;

    const TEST_STABLE_INSTANCE_KEY: u64 = 0x7200_0001;
    const TEST_OTHER_STABLE_INSTANCE_KEY: u64 = 0x7200_0002;
    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn render_gpu_scene_rolls_current_morph_weights_after_success() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);

        scene.stage_current_morph_weights(TEST_STABLE_INSTANCE_KEY, Some(&[0.25, 0.0]));
        let first_report = scene.roll_prev_morph_weights_after_success();

        assert_eq!(first_report.current_weight_state_count, 1);
        assert_eq!(first_report.previous_weight_state_count, 1);
        assert_eq!(first_report.removed_previous_weight_state_count, 0);
        assert_eq!(
            scene.previous_morph_weights(TEST_STABLE_INSTANCE_KEY),
            Some([0.25, 0.0].as_slice())
        );

        scene.stage_current_morph_weights(TEST_STABLE_INSTANCE_KEY, Some(&[0.0, 0.5]));

        assert_eq!(
            scene.previous_morph_weights(TEST_STABLE_INSTANCE_KEY),
            Some([0.25, 0.0].as_slice())
        );

        let second_report = scene.roll_prev_morph_weights_after_success();

        assert_eq!(second_report.current_weight_state_count, 1);
        assert_eq!(second_report.previous_weight_state_count, 1);
        assert_eq!(second_report.removed_previous_weight_state_count, 0);
        assert_eq!(
            scene.previous_morph_weights(TEST_STABLE_INSTANCE_KEY),
            Some([0.0, 0.5].as_slice())
        );
    }

    #[test]
    fn render_gpu_scene_drops_previous_morph_weights_when_current_is_missing() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);

        scene.stage_current_morph_weights(TEST_STABLE_INSTANCE_KEY, Some(&[0.25]));
        scene.stage_current_morph_weights(TEST_OTHER_STABLE_INSTANCE_KEY, Some(&[0.5]));
        let _ = scene.roll_prev_morph_weights_after_success();

        scene.stage_current_morph_weights(TEST_STABLE_INSTANCE_KEY, None);
        let report = scene.roll_prev_morph_weights_after_success();

        assert_eq!(report.current_weight_state_count, 1);
        assert_eq!(report.previous_weight_state_count, 1);
        assert_eq!(report.removed_previous_weight_state_count, 1);
        assert_eq!(scene.previous_morph_weights(TEST_STABLE_INSTANCE_KEY), None);
        assert_eq!(
            scene.previous_morph_weights(TEST_OTHER_STABLE_INSTANCE_KEY),
            Some([0.5].as_slice())
        );
    }

    #[test]
    fn render_gpu_scene_rolls_explicit_zero_morph_weights_for_starting_velocity() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);

        scene.stage_current_morph_weights(TEST_STABLE_INSTANCE_KEY, Some(&[0.0]));
        let first_report = scene.roll_prev_morph_weights_after_success();

        assert_eq!(first_report.current_weight_state_count, 1);
        assert_eq!(first_report.previous_weight_state_count, 1);
        assert_eq!(
            scene.previous_morph_weights(TEST_STABLE_INSTANCE_KEY),
            Some([0.0].as_slice())
        );

        scene.stage_current_morph_weights(TEST_STABLE_INSTANCE_KEY, Some(&[1.0]));

        assert_eq!(
            scene.previous_morph_weights(TEST_STABLE_INSTANCE_KEY),
            Some([0.0].as_slice()),
            "0 -> nonzero morph changes need an explicit zero previous state for velocity"
        );
    }

    #[test]
    fn stable_morph_weights_reuse_current_and_previous_snapshots() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);

        scene.stage_current_morph_weights(TEST_STABLE_INSTANCE_KEY, Some(&[0.25, 0.5]));
        let first_current = Arc::clone(
            scene
                .current_morph_weights
                .get(&TEST_STABLE_INSTANCE_KEY)
                .expect("current morph snapshot"),
        );
        scene.stage_current_morph_weights(TEST_STABLE_INSTANCE_KEY, Some(&[0.25, 0.5]));
        let stable_current = Arc::clone(
            scene
                .current_morph_weights
                .get(&TEST_STABLE_INSTANCE_KEY)
                .expect("stable current morph snapshot"),
        );
        assert!(Arc::ptr_eq(&first_current, &stable_current));

        let _ = scene.roll_prev_morph_weights_after_success();
        let previous = scene
            .previous_morph_weights
            .get(&TEST_STABLE_INSTANCE_KEY)
            .expect("previous morph snapshot");
        assert!(Arc::ptr_eq(&stable_current, previous));
    }

    #[test]
    fn optimization_batch_dg_morph_roll_reuses_updates_and_removes_exact_keys() {
        let stable = Arc::<[f32]>::from([0.25, 0.5]);
        let removed = Arc::<[f32]>::from([1.0]);
        let replaced = Arc::<[f32]>::from([0.0]);
        let replacement = Arc::<[f32]>::from([0.75]);
        let added = Arc::<[f32]>::from([0.125]);
        let mut previous = HashMap::from([(1, Arc::clone(&stable)), (2, removed), (4, replaced)]);
        let current = HashMap::from([
            (1, Arc::clone(&stable)),
            (3, Arc::clone(&added)),
            (4, Arc::clone(&replacement)),
        ]);

        let removed_count = roll_previous_morph_weight_snapshots(&mut previous, &current);

        assert_eq!(removed_count, 1);
        assert_eq!(previous.len(), current.len());
        assert!(Arc::ptr_eq(previous.get(&1).unwrap(), &stable));
        assert!(Arc::ptr_eq(previous.get(&3).unwrap(), &added));
        assert!(Arc::ptr_eq(previous.get(&4).unwrap(), &replacement));
        assert!(!previous.contains_key(&2));
    }

    #[test]
    fn optimization_batch_dg_morph_roll_source_keeps_stable_snapshots_in_place() {
        let source = include_str!("prev_morph_weights.rs");

        assert!(source.contains("previous.retain(|key, previous_weights|"));
        assert!(source.contains("Arc::ptr_eq(previous_weights, current_weights)"));
        assert!(source.contains("if previous.len() < current.len()"));
        assert!(!source.contains("self.previous_morph_weights.clear()"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dg_stable_morph_roll_in_place_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const ROLLS_PER_SAMPLE: usize = 64;
        const WEIGHT_STATE_COUNT: usize = 2_048;

        let current = optimization_batch_dg_morph_fixture(WEIGHT_STATE_COUNT);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(optimization_batch_dg_measure_morph_roll(
                    &current,
                    ROLLS_PER_SAMPLE,
                    optimization_batch_dg_legacy_morph_roll,
                ));
                optimized_samples.push(optimization_batch_dg_measure_morph_roll(
                    &current,
                    ROLLS_PER_SAMPLE,
                    roll_previous_morph_weight_snapshots,
                ));
            } else {
                optimized_samples.push(optimization_batch_dg_measure_morph_roll(
                    &current,
                    ROLLS_PER_SAMPLE,
                    roll_previous_morph_weight_snapshots,
                ));
                legacy_samples.push(optimization_batch_dg_measure_morph_roll(
                    &current,
                    ROLLS_PER_SAMPLE,
                    optimization_batch_dg_legacy_morph_roll,
                ));
            }
        }

        let legacy_p95 = optimization_batch_dg_morph_p95(&mut legacy_samples);
        let optimized_p95 = optimization_batch_dg_morph_p95(&mut optimized_samples);
        println!(
            "RUNTIME415_STABLE_MORPH_ROLL_IN_PLACE_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "in-place stable morph roll p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn optimization_batch_dg_morph_fixture(count: usize) -> HashMap<u64, Arc<[f32]>> {
        (0..count)
            .map(|index| {
                (
                    index as u64,
                    Arc::<[f32]>::from([
                        index as f32,
                        index as f32 * 0.5,
                        index as f32 * 0.25,
                        1.0,
                    ]),
                )
            })
            .collect()
    }

    fn optimization_batch_dg_legacy_morph_roll(
        previous: &mut HashMap<u64, Arc<[f32]>>,
        current: &HashMap<u64, Arc<[f32]>>,
    ) -> usize {
        let removed = previous
            .keys()
            .filter(|key| !current.contains_key(*key))
            .count();
        previous.clear();
        previous.extend(
            current
                .iter()
                .map(|(key, weights)| (*key, Arc::clone(weights))),
        );
        removed
    }

    fn optimization_batch_dg_measure_morph_roll(
        current: &HashMap<u64, Arc<[f32]>>,
        rolls: usize,
        roll: fn(&mut HashMap<u64, Arc<[f32]>>, &HashMap<u64, Arc<[f32]>>) -> usize,
    ) -> u128 {
        let mut previous = current.clone();
        let started_at = std::time::Instant::now();
        let mut removed = 0;
        for _ in 0..rolls {
            removed += roll(
                std::hint::black_box(&mut previous),
                std::hint::black_box(current),
            );
        }
        std::hint::black_box((removed, previous.len()));
        started_at.elapsed().as_nanos()
    }

    fn optimization_batch_dg_morph_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[index]
    }

    fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
        crate::graphics::backend::RenderBackend::new_offscreen()
            .inspect_err(|error| eprintln!("skipping gpu scene prev morph test: {error:?}"))
            .ok()
    }

    fn test_gpu_scene(device: &wgpu::Device) -> GpuScene {
        GpuScene::new(
            device,
            test_skinned_joint_palette_buffer(device),
            test_skinned_joint_palette_min_binding_size(),
        )
    }

    fn test_skinned_joint_palette_buffer(device: &wgpu::Device) -> Arc<wgpu::Buffer> {
        Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-prev-morph-palette-buffer"),
            size: test_skinned_joint_palette_min_binding_size().get(),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette storage size is non-zero")
    }
}
