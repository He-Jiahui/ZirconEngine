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
            .map(Vec::as_slice)
    }

    pub(crate) fn stage_current_morph_weights(
        &mut self,
        stable_instance_key: u64,
        weights: Option<&[f32]>,
    ) {
        if let Some(weights) = weights.filter(|weights| !weights.is_empty()) {
            self.current_morph_weights
                .insert(stable_instance_key, weights.to_vec());
        } else {
            self.current_morph_weights.remove(&stable_instance_key);
        }
    }

    pub(crate) fn roll_prev_morph_weights_after_success(
        &mut self,
    ) -> GpuScenePrevMorphWeightsRollReport {
        let removed_previous_weight_state_count = self
            .previous_morph_weights
            .keys()
            .filter(|key| !self.current_morph_weights.contains_key(*key))
            .count();
        self.previous_morph_weights.clear();
        self.previous_morph_weights.extend(
            self.current_morph_weights
                .iter()
                .map(|(key, weights)| (*key, weights.clone())),
        );

        let previous_weight_state_count = self.previous_morph_weights.len();
        GpuScenePrevMorphWeightsRollReport {
            current_weight_state_count: self.current_morph_weights.len(),
            previous_weight_state_count,
            removed_previous_weight_state_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette uniform size is non-zero")
    }
}
