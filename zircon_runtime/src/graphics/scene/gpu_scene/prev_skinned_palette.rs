use crate::graphics::scene::scene_renderer::SkinnedMeshJointPaletteUniform;

use super::gpu_scene::GpuScene;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GpuSceneSkinnedJointPaletteState {
    pub(crate) signature: u64,
    pub(crate) morph_shape_signature: Option<u64>,
    pub(crate) uniform: SkinnedMeshJointPaletteUniform,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuScenePrevSkinnedPaletteRollReport {
    pub(crate) current_palette_count: usize,
    pub(crate) previous_palette_count: usize,
    pub(crate) removed_previous_palette_count: usize,
}

impl GpuScene {
    pub(crate) fn previous_skinned_joint_palette_state(
        &self,
        stable_instance_key: u64,
    ) -> Option<GpuSceneSkinnedJointPaletteState> {
        self.previous_skinned_joint_palettes
            .get(&stable_instance_key)
            .copied()
    }

    pub(crate) fn stage_current_skinned_joint_palette(
        &mut self,
        stable_instance_key: u64,
        palette: Option<GpuSceneSkinnedJointPaletteState>,
    ) {
        if let Some(palette) = palette {
            self.current_skinned_joint_palettes
                .insert(stable_instance_key, palette);
        } else {
            self.current_skinned_joint_palettes
                .remove(&stable_instance_key);
        }
    }

    pub(crate) fn roll_prev_skinned_palettes_after_success(
        &mut self,
    ) -> GpuScenePrevSkinnedPaletteRollReport {
        let removed_previous_palette_count = self
            .previous_skinned_joint_palettes
            .keys()
            .filter(|key| !self.current_skinned_joint_palettes.contains_key(*key))
            .count();
        self.previous_skinned_joint_palettes.clear();
        self.previous_skinned_joint_palettes.extend(
            self.current_skinned_joint_palettes
                .iter()
                .map(|(key, palette)| (*key, *palette)),
        );

        let previous_palette_count = self.previous_skinned_joint_palettes.len();
        GpuScenePrevSkinnedPaletteRollReport {
            current_palette_count: self.current_skinned_joint_palettes.len(),
            previous_palette_count,
            removed_previous_palette_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::math::Mat4;
    use crate::graphics::scene::gpu_scene::GpuScene;

    use super::*;

    const TEST_STABLE_INSTANCE_KEY: u64 = 0x7000_0001;
    const TEST_OTHER_STABLE_INSTANCE_KEY: u64 = 0x7000_0002;
    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn render_gpu_scene_rolls_current_skinned_palette_after_success() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let first = test_palette(1.0);
        let second = test_palette(2.0);

        scene.stage_current_skinned_joint_palette(TEST_STABLE_INSTANCE_KEY, Some(first));
        let first_report = scene.roll_prev_skinned_palettes_after_success();

        assert_eq!(first_report.current_palette_count, 1);
        assert_eq!(first_report.previous_palette_count, 1);
        assert_eq!(first_report.removed_previous_palette_count, 0);
        assert_eq!(
            scene
                .previous_skinned_joint_palette_state(TEST_STABLE_INSTANCE_KEY)
                .map(|state| state.uniform),
            Some(first.uniform)
        );

        scene.stage_current_skinned_joint_palette(TEST_STABLE_INSTANCE_KEY, Some(second));

        assert_eq!(
            scene
                .previous_skinned_joint_palette_state(TEST_STABLE_INSTANCE_KEY)
                .map(|state| state.uniform),
            Some(first.uniform)
        );

        let second_report = scene.roll_prev_skinned_palettes_after_success();

        assert_eq!(second_report.current_palette_count, 1);
        assert_eq!(second_report.previous_palette_count, 1);
        assert_eq!(second_report.removed_previous_palette_count, 0);
        assert_eq!(
            scene
                .previous_skinned_joint_palette_state(TEST_STABLE_INSTANCE_KEY)
                .map(|state| state.uniform),
            Some(second.uniform)
        );
    }

    #[test]
    fn render_gpu_scene_drops_previous_skinned_palette_when_current_is_missing() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);

        scene
            .stage_current_skinned_joint_palette(TEST_STABLE_INSTANCE_KEY, Some(test_palette(1.0)));
        scene.stage_current_skinned_joint_palette(
            TEST_OTHER_STABLE_INSTANCE_KEY,
            Some(test_palette(2.0)),
        );
        let _ = scene.roll_prev_skinned_palettes_after_success();

        scene.stage_current_skinned_joint_palette(TEST_STABLE_INSTANCE_KEY, None);
        let report = scene.roll_prev_skinned_palettes_after_success();

        assert_eq!(report.current_palette_count, 1);
        assert_eq!(report.previous_palette_count, 1);
        assert_eq!(report.removed_previous_palette_count, 1);
        assert_eq!(
            scene.previous_skinned_joint_palette_state(TEST_STABLE_INSTANCE_KEY),
            None
        );
        assert_eq!(
            scene
                .previous_skinned_joint_palette_state(TEST_OTHER_STABLE_INSTANCE_KEY)
                .map(|state| state.uniform),
            Some(test_palette(2.0).uniform)
        );
    }

    fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
        crate::graphics::backend::RenderBackend::new_offscreen()
            .inspect_err(|error| eprintln!("skipping gpu scene prev palette test: {error:?}"))
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
            label: Some("zircon-test-prev-skinned-palette-buffer"),
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

    fn test_palette(translate_x: f32) -> GpuSceneSkinnedJointPaletteState {
        GpuSceneSkinnedJointPaletteState {
            signature: translate_x.to_bits() as u64,
            morph_shape_signature: None,
            uniform: SkinnedMeshJointPaletteUniform::from_matrices(&[Mat4::from_translation(
                crate::core::math::Vec3::new(translate_x, 0.0, 0.0),
            )])
            .expect("test palette fits fixed uniform ABI"),
        }
    }
}
