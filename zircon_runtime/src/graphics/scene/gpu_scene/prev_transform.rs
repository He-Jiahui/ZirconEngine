use super::gpu_scene::GpuScene;
use super::layout::GPU_INSTANCE_DATA_STRIDE;
use super::update_queue::GpuSceneUpdateQueue;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuScenePrevTransformRollReport {
    pub(crate) live_instance_count: u32,
    pub(crate) visited_entry_count: u32,
    pub(crate) rolled_instance_count: u32,
    pub(crate) dirty_instance_range_count: usize,
}

impl GpuScene {
    pub(crate) fn previous_world_from_local(
        &self,
        entry: super::gpu_scene::GpuSceneEntry,
    ) -> Option<[[f32; 4]; 4]> {
        if !entry.has_rolled_previous_transform {
            return None;
        }
        self.instance_shadow
            .get(entry.first_instance_index as usize)
            .map(|instance| instance.prev_world_from_local)
    }

    pub(crate) fn roll_prev_transforms_after_success(&mut self) -> GpuScenePrevTransformRollReport {
        let mut pending = std::mem::take(&mut self.pending_prev_transform_rolls);
        roll_prev_transforms_for_keys(
            &mut self.entries,
            &mut self.instance_shadow,
            &mut pending,
            &mut self.updates,
            self.stats().instance_count,
        )
    }

    #[cfg(test)]
    pub(crate) fn debug_previous_world_from_local_at(&self, instance_index: u32) -> [[f32; 4]; 4] {
        self.instance_shadow[instance_index as usize].prev_world_from_local
    }

    #[cfg(test)]
    pub(crate) fn debug_dirty_instance_upload_byte_count(&mut self) -> u64 {
        self.updates
            .drain_instance_upload_ranges(GPU_INSTANCE_DATA_STRIDE as u64)
            .into_iter()
            .map(|range| range.byte_len)
            .sum()
    }
}

fn roll_prev_transforms_for_keys(
    entries: &mut HashMap<u64, super::gpu_scene::GpuSceneEntry>,
    instance_shadow: &mut [super::layout::GpuInstanceData],
    pending: &mut HashSet<u64>,
    updates: &mut GpuSceneUpdateQueue,
    live_instance_count: u32,
) -> GpuScenePrevTransformRollReport {
    let mut report = GpuScenePrevTransformRollReport {
        live_instance_count,
        ..GpuScenePrevTransformRollReport::default()
    };

    for stable_instance_key in pending.drain() {
        let Some(entry) = entries.get_mut(&stable_instance_key) else {
            continue;
        };
        report.visited_entry_count += 1;
        {
            let start = entry.first_instance_index;
            let end = start
                .checked_add(entry.instance_count)
                .expect("gpu scene instance span overflowed during prev transform roll");
            let mut span_changed = false;
            for instance_index in start..end {
                let instance = &mut instance_shadow[instance_index as usize];
                if instance.prev_world_from_local != instance.world_from_local {
                    instance.prev_world_from_local = instance.world_from_local;
                    span_changed = true;
                    report.rolled_instance_count += 1;
                }
            }
            entry.has_rolled_previous_transform = true;
            if span_changed {
                updates.mark_instances(start, entry.instance_count);
                report.dirty_instance_range_count += 1;
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::graphics::scene::gpu_scene::update_queue::GpuSceneUpdateQueue;
    use crate::graphics::scene::gpu_scene::{
        GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT, GpuInstanceData,
        GpuPrimitiveData, GpuScene, GpuSceneEntry,
    };

    const TEST_STABLE_INSTANCE_KEY: u64 = 0x6000_0001;
    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn render_gpu_scene_rolls_current_transform_into_previous_after_success() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let entry = scene.register(&backend.device, TEST_STABLE_INSTANCE_KEY, 1);
        scene.write_primitive(entry, test_primitive_data());
        scene.write_instances(entry, &[test_instance_data(3.0, 1.0)]);
        scene
            .flush_updates(&backend)
            .expect("GPU scene upload should be accepted");

        assert_eq!(scene.previous_world_from_local(entry), None);
        let report = scene.roll_prev_transforms_after_success();

        assert_eq!(report.live_instance_count, 1);
        assert_eq!(report.visited_entry_count, 1);
        assert_eq!(report.rolled_instance_count, 1);
        assert_eq!(report.dirty_instance_range_count, 1);
        let rolled = test_matrix(3.0);
        let rolled_entry = scene
            .entry(TEST_STABLE_INSTANCE_KEY)
            .expect("rolled gpu scene entry");
        assert_eq!(scene.previous_world_from_local(rolled_entry), Some(rolled));
        assert_eq!(
            scene.debug_previous_world_from_local_at(rolled_entry.first_instance_index),
            rolled
        );
        assert_eq!(
            scene.debug_dirty_instance_upload_byte_count(),
            crate::graphics::scene::gpu_scene::GPU_INSTANCE_DATA_STRIDE as u64
        );
    }

    #[test]
    fn render_gpu_scene_roll_marks_previous_valid_without_dirty_upload_when_unchanged() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let entry = scene.register(&backend.device, TEST_STABLE_INSTANCE_KEY, 1);
        scene.write_primitive(entry, test_primitive_data());
        scene.write_instances(entry, &[test_instance_data(4.0, 4.0)]);
        scene
            .flush_updates(&backend)
            .expect("GPU scene upload should be accepted");

        let report = scene.roll_prev_transforms_after_success();

        assert_eq!(report.live_instance_count, 1);
        assert_eq!(report.visited_entry_count, 1);
        assert_eq!(report.rolled_instance_count, 0);
        assert_eq!(report.dirty_instance_range_count, 0);
        let rolled_entry = scene
            .entry(TEST_STABLE_INSTANCE_KEY)
            .expect("rolled gpu scene entry");
        assert_eq!(
            scene.previous_world_from_local(rolled_entry),
            Some(test_matrix(4.0))
        );

        let steady_report = scene.roll_prev_transforms_after_success();
        assert_eq!(steady_report.live_instance_count, 1);
        assert_eq!(steady_report.visited_entry_count, 0);
        assert_eq!(steady_report.rolled_instance_count, 0);
        assert_eq!(steady_report.dirty_instance_range_count, 0);
    }

    #[test]
    fn optimization_wave_20260824p_runtime94_rolls_only_pending_transform_entries() {
        let (mut entries, mut instances) = synthetic_roll_state(4, &[3]);
        let mut pending = HashSet::from([3]);
        let mut updates = GpuSceneUpdateQueue::new();

        let report = roll_prev_transforms_for_keys(
            &mut entries,
            &mut instances,
            &mut pending,
            &mut updates,
            4,
        );

        assert_eq!(report.live_instance_count, 4);
        assert_eq!(report.visited_entry_count, 1);
        assert_eq!(report.rolled_instance_count, 1);
        assert_eq!(report.dirty_instance_range_count, 1);
        assert!(pending.is_empty());
        assert!(
            entries
                .get(&3)
                .is_some_and(|entry| entry.has_rolled_previous_transform)
        );
        assert_eq!(
            instances[3].prev_world_from_local,
            instances[3].world_from_local
        );
        assert_eq!(
            instances[2].prev_world_from_local,
            instances[2].world_from_local
        );
    }

    #[test]
    fn optimization_wave_20260824p_runtime94_previous_transform_roll_uses_a_dirty_key_set() {
        let source = include_str!("prev_transform.rs");
        let (implementation, _) = source
            .rsplit_once("#[cfg(test)]")
            .expect("previous transform test module");
        let gpu_scene_source = include_str!("gpu_scene.rs");
        let gpu_scene_implementation = gpu_scene_source
            .split("#[cfg(test)]")
            .next()
            .expect("gpu scene implementation");
        let compact_gpu_scene = gpu_scene_implementation
            .split_whitespace()
            .collect::<String>();

        assert!(gpu_scene_implementation.contains("pending_prev_transform_rolls: HashSet<u64>"));
        assert!(gpu_scene_implementation.contains("stable_instance_key"));
        assert!(
            compact_gpu_scene
                .contains("self.pending_prev_transform_rolls.insert(stable_instance_key);")
        );
        assert!(
            compact_gpu_scene
                .contains("self.pending_prev_transform_rolls.remove(&stable_instance_key);")
        );
        assert!(implementation.contains("std::mem::take(&mut self.pending_prev_transform_rolls)"));
        assert!(implementation.contains("self.stats.instance_count"));
        assert!(!implementation.contains("self.entries.values_mut()"));
    }

    #[test]
    #[ignore = "managed release evidence"]
    fn optimization_wave_20260824p_runtime94_previous_transform_dirty_queue_evidence() {
        const LIVE_ENTRY_COUNT: usize = 100_000;
        const DIRTY_ENTRY_COUNT: usize = 32;
        const SAMPLE_PAIRS: usize = 11;
        const TARGET: Duration = Duration::from_millis(5);

        let dirty_keys = ((LIVE_ENTRY_COUNT - DIRTY_ENTRY_COUNT)..LIVE_ENTRY_COUNT)
            .map(|key| key as u64)
            .collect::<Vec<_>>();
        let (mut entries, mut instances) = synthetic_roll_state(LIVE_ENTRY_COUNT, &dirty_keys);
        let mut updates = GpuSceneUpdateQueue::new();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy_roll(
                    &mut entries,
                    &mut instances,
                    &dirty_keys,
                    &mut updates,
                ));
                optimized_samples.push(measure_indexed_roll(
                    &mut entries,
                    &mut instances,
                    &dirty_keys,
                    &mut updates,
                ));
            } else {
                optimized_samples.push(measure_indexed_roll(
                    &mut entries,
                    &mut instances,
                    &dirty_keys,
                    &mut updates,
                ));
                legacy_samples.push(measure_legacy_roll(
                    &mut entries,
                    &mut instances,
                    &dirty_keys,
                    &mut updates,
                ));
            }
        }

        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let entry_visits_before = LIVE_ENTRY_COUNT;
        let dirty_key_probes_after = DIRTY_ENTRY_COUNT;
        let lookup_work_reduction_percent =
            (1.0 - dirty_key_probes_after as f64 / entry_visits_before as f64) * 100.0;

        assert!(
            optimized_p95 <= TARGET.as_nanos(),
            "optimized_p95_ns={optimized_p95} target_ns={}",
            TARGET.as_nanos()
        );
        assert!(
            optimized_p95.saturating_mul(2) <= legacy_p95,
            "optimized_p95_ns={optimized_p95} legacy_p95_ns={legacy_p95}"
        );
        println!(
            "RUNTIME94_PREV_TRANSFORM_BENCH_V1 live_entries={} dirty_entries={} entry_visits_before={} dirty_key_probes_after={} lookup_work_reduction_percent={:.4} legacy_p95_ns={} optimized_p95_ns={} target_ns={}",
            LIVE_ENTRY_COUNT,
            DIRTY_ENTRY_COUNT,
            entry_visits_before,
            dirty_key_probes_after,
            lookup_work_reduction_percent,
            legacy_p95,
            optimized_p95,
            TARGET.as_nanos()
        );
    }

    fn synthetic_roll_state(
        entry_count: usize,
        dirty_keys: &[u64],
    ) -> (HashMap<u64, GpuSceneEntry>, Vec<GpuInstanceData>) {
        let dirty_keys = dirty_keys.iter().copied().collect::<HashSet<_>>();
        let mut entries = HashMap::with_capacity(entry_count);
        let mut instances = Vec::with_capacity(entry_count);
        for key in 0..entry_count as u64 {
            let current = key as f32 + 1.0;
            let previous = if dirty_keys.contains(&key) {
                current - 1.0
            } else {
                current
            };
            entries.insert(
                key,
                GpuSceneEntry {
                    stable_instance_key: key,
                    primitive_index: key as u32,
                    first_instance_index: key as u32,
                    instance_count: 1,
                    last_transform_revision: 1,
                    has_rolled_previous_transform: false,
                },
            );
            instances.push(test_instance_data(current, previous));
        }
        (entries, instances)
    }

    fn reset_dirty_previous_transforms(instances: &mut [GpuInstanceData], dirty_keys: &[u64]) {
        for key in dirty_keys {
            let instance = &mut instances[*key as usize];
            let current_x = instance.world_from_local[3][0];
            instance.prev_world_from_local = test_matrix(current_x - 1.0);
        }
    }

    fn measure_legacy_roll(
        entries: &mut HashMap<u64, GpuSceneEntry>,
        instances: &mut [GpuInstanceData],
        dirty_keys: &[u64],
        updates: &mut GpuSceneUpdateQueue,
    ) -> u128 {
        reset_dirty_previous_transforms(instances, dirty_keys);
        let elapsed = measure_ns(|| legacy_roll_prev_transforms(entries, instances, updates));
        updates.discard_instance_updates();
        elapsed
    }

    fn measure_indexed_roll(
        entries: &mut HashMap<u64, GpuSceneEntry>,
        instances: &mut [GpuInstanceData],
        dirty_keys: &[u64],
        updates: &mut GpuSceneUpdateQueue,
    ) -> u128 {
        reset_dirty_previous_transforms(instances, dirty_keys);
        let mut pending = dirty_keys.iter().copied().collect::<HashSet<_>>();
        let live_instance_count = entries.len() as u32;
        let elapsed = measure_ns(|| {
            roll_prev_transforms_for_keys(
                entries,
                instances,
                &mut pending,
                updates,
                live_instance_count,
            )
        });
        updates.discard_instance_updates();
        elapsed
    }

    fn legacy_roll_prev_transforms(
        entries: &mut HashMap<u64, GpuSceneEntry>,
        instances: &mut [GpuInstanceData],
        updates: &mut GpuSceneUpdateQueue,
    ) -> u64 {
        let mut checksum = 0_u64;
        for entry in entries.values_mut() {
            checksum = checksum.wrapping_add(u64::from(entry.primitive_index));
            let start = entry.first_instance_index;
            let end = start + entry.instance_count;
            let mut span_changed = false;
            for instance_index in start..end {
                let instance = &mut instances[instance_index as usize];
                if instance.prev_world_from_local != instance.world_from_local {
                    instance.prev_world_from_local = instance.world_from_local;
                    span_changed = true;
                }
            }
            entry.has_rolled_previous_transform = true;
            if span_changed {
                updates.mark_instances(start, entry.instance_count);
            }
        }
        black_box(checksum)
    }

    fn measure_ns<T>(measure: impl FnOnce() -> T) -> u128 {
        let started = Instant::now();
        black_box(measure());
        started.elapsed().as_nanos()
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
        crate::graphics::backend::RenderBackend::new_offscreen()
            .inspect_err(|error| eprintln!("skipping gpu scene prev transform test: {error:?}"))
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
            label: Some("zircon-test-prev-transform-skinned-palette-buffer"),
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

    fn test_primitive_data() -> GpuPrimitiveData {
        GpuPrimitiveData {
            local_bounds_center: [0.0, 0.0, 0.0],
            local_bounds_radius: 1.0,
            tint: [1.0, 1.0, 1.0, 1.0],
            flags: GPU_PRIMITIVE_FLAG_VISIBLE,
            first_instance_index: u32::MAX,
            instance_count: u32::MAX,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            ..GpuPrimitiveData::default()
        }
    }

    fn test_instance_data(current_x: f32, previous_x: f32) -> GpuInstanceData {
        GpuInstanceData {
            world_from_local: test_matrix(current_x),
            prev_world_from_local: test_matrix(previous_x),
            primitive_index: u32::MAX,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            morph_payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            ..GpuInstanceData::default()
        }
    }

    fn test_matrix(translate_x: f32) -> [[f32; 4]; 4] {
        let mut matrix = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        matrix[3][0] = translate_x;
        matrix
    }
}
