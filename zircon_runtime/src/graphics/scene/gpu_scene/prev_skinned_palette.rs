use zr_rhi_wgpu::WgpuBufferUploadBatch;

use crate::graphics::scene::scene_renderer::SkinnedMeshJointPaletteStorage;

use super::gpu_scene::GpuScene;

/// 保存某稳定实例的骨骼姿态签名和调色板快照，供下一帧运动向量判定历史是否可复用。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GpuSceneSkinnedJointPaletteState {
    pub(crate) signature: u64,
    pub(crate) morph_shape_signature: Option<u64>,
    pub(crate) storage: SkinnedMeshJointPaletteStorage,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuScenePrevSkinnedPaletteRollReport {
    pub(crate) current_palette_count: usize,
    pub(crate) previous_palette_count: usize,
    pub(crate) removed_previous_palette_count: usize,
}

impl GpuScene {
    pub(crate) fn begin_skinned_joint_palette_frame(&mut self) {
        self.skinned_palette_arena.begin_frame();
    }

    pub(crate) fn stage_skinned_joint_palette_arena(
        &mut self,
        stable_instance_key: u64,
        current_storage: Option<&SkinnedMeshJointPaletteStorage>,
        expose_current: bool,
        expose_previous: bool,
    ) -> [u32; 4] {
        self.skinned_palette_arena.stage_palette(
            stable_instance_key,
            current_storage,
            expose_current,
            expose_previous,
        )
    }

    pub(crate) fn prepare_skinned_joint_palette_upload(
        &mut self,
        device: &wgpu::Device,
    ) -> (WgpuBufferUploadBatch, u64) {
        let prepared = self.skinned_palette_arena.prepare_upload(device);
        if prepared.buffer_recreated {
            self.rebuild_scene_bind_group(device);
        }
        (prepared.batch, prepared.uploaded_bytes)
    }

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

    /// 只有场景帧成功提交后才将本帧姿态转为历史；失败帧不得污染下一帧速度重建。
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
        self.skinned_palette_arena.commit_after_scene_success();

        let previous_palette_count = self.previous_skinned_joint_palettes.len();
        GpuScenePrevSkinnedPaletteRollReport {
            current_palette_count: self.current_skinned_joint_palettes.len(),
            previous_palette_count,
            removed_previous_palette_count,
        }
    }
}

#[cfg(test)]
#[path = "tests/prev_skinned_palette.rs"]
mod tests;
