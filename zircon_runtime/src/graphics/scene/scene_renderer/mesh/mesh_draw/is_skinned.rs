use super::MeshDraw;

impl MeshDraw {
    pub(crate) fn is_skinned(&self) -> bool {
        self.skinned
    }

    pub(crate) fn has_skinned_joint_palette_upload(&self) -> bool {
        self.skinned_joint_palette_buffer.is_some()
    }

    pub(crate) fn has_previous_skinned_joint_palette_upload(&self) -> bool {
        self.previous_skinned_joint_palette_buffer.is_some()
    }

    pub(crate) fn has_skinned_gpu_source_candidate(&self) -> bool {
        self.skinned_gpu_source.is_some()
    }

    pub(crate) fn has_skinned_gpu_cpu_morphed_source_candidate(&self) -> bool {
        self.skinned_gpu_source.is_some() && self.skinned_gpu_source_uses_cpu_morphed_source
    }

    pub(crate) fn uses_skinned_gpu_skinning(&self) -> bool {
        self.skinned_gpu_skinning_enabled
    }
}
