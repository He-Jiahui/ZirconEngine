use bytemuck::{Pod, Zeroable};

use crate::graphics::types::ViewportRenderFrame;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
/// 独立于 SceneUniform 的九项辐照度 SH 参数，绑定到场景组 6；无源环境时上传零系数。
// 当前契约是场景 group 0 的 binding 6：scene_bind_group_layout_entries 与 WGSL 镜像使用同一槽位。
pub(crate) struct SceneEnvironmentSh9 {
    coefficients: [[f32; 4]; 9],
}

impl SceneEnvironmentSh9 {
    pub(crate) fn from_frame(frame: &ViewportRenderFrame) -> Self {
        Self {
            coefficients: frame
                .source_cubemap_environment()
                .map(|environment| environment.irradiance_sh9)
                .unwrap_or([[0.0; 4]; 9]),
        }
    }

    pub(crate) const fn byte_len() -> u64 {
        std::mem::size_of::<Self>() as u64
    }

    #[cfg(test)]
    pub(super) const fn coefficients(&self) -> &[[f32; 4]; 9] {
        &self.coefficients
    }
}

impl Default for SceneEnvironmentSh9 {
    fn default() -> Self {
        Self {
            coefficients: [[0.0; 4]; 9],
        }
    }
}

#[cfg(test)]
#[path = "tests/scene_environment_sh9.rs"]
mod tests;
