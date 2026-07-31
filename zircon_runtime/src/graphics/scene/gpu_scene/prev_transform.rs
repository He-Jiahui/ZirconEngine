use super::gpu_scene::GpuScene;
use super::layout::GPU_INSTANCE_DATA_STRIDE;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuScenePrevTransformRollReport {
    pub(crate) live_instance_count: u32,
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
        let mut report = GpuScenePrevTransformRollReport {
            live_instance_count: self
                .entries
                .values()
                .map(|entry| entry.instance_count)
                .sum(),
            ..GpuScenePrevTransformRollReport::default()
        };

        for entry in self.entries.values_mut() {
            let start = entry.first_instance_index;
            let end = start
                .checked_add(entry.instance_count)
                .expect("gpu scene instance span overflowed during prev transform roll");
            let mut span_changed = false;
            for instance_index in start..end {
                let instance = &mut self.instance_shadow[instance_index as usize];
                if instance.prev_world_from_local != instance.world_from_local {
                    instance.prev_world_from_local = instance.world_from_local;
                    span_changed = true;
                    report.rolled_instance_count += 1;
                }
            }
            entry.has_rolled_previous_transform = true;
            if span_changed {
                self.updates.mark_instances(start, entry.instance_count);
                report.dirty_instance_range_count += 1;
            }
        }

        report
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::graphics::scene::gpu_scene::{
        GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT, GpuInstanceData,
        GpuPrimitiveData, GpuScene,
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
        let _ = scene.flush_updates(&backend.queue);

        assert_eq!(scene.previous_world_from_local(entry), None);
        let report = scene.roll_prev_transforms_after_success();

        assert_eq!(report.live_instance_count, 1);
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
        let _ = scene.flush_updates(&backend.queue);

        let report = scene.roll_prev_transforms_after_success();

        assert_eq!(report.live_instance_count, 1);
        assert_eq!(report.rolled_instance_count, 0);
        assert_eq!(report.dirty_instance_range_count, 0);
        let rolled_entry = scene
            .entry(TEST_STABLE_INSTANCE_KEY)
            .expect("rolled gpu scene entry");
        assert_eq!(
            scene.previous_world_from_local(rolled_entry),
            Some(test_matrix(4.0))
        );
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
            bounds_center: [0.0, 0.0, 0.0],
            bounds_radius: 1.0,
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
