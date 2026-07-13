use bytemuck::{Pod, Zeroable};

use crate::graphics::types::ViewportRenderFrame;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SceneEnvironmentSh9 {
    coefficients: [[f32; 4]; 9],
}

impl SceneEnvironmentSh9 {
    pub(crate) fn from_frame(frame: &ViewportRenderFrame) -> Self {
        Self {
            coefficients: frame
                .environment()
                .skybox
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
mod tests {
    use super::SceneEnvironmentSh9;

    #[test]
    fn scene_environment_sh9_matches_gpu_artifact_layout() {
        assert_eq!(SceneEnvironmentSh9::byte_len(), 9 * 4 * 4);
        assert_eq!(
            SceneEnvironmentSh9::default().coefficients(),
            &[[0.0; 4]; 9]
        );
    }
}
