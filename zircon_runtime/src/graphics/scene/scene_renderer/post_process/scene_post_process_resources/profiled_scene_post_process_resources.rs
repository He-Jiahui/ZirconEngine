use std::ops::Deref;

use super::{FullScenePostProcessResources, SceneOutputTransferResources};

pub(crate) enum ScenePostProcessResources {
    Full(FullScenePostProcessResources),
    OutputTransferOnly(SceneOutputTransferResources),
}

impl ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer) const fn has_full_resources(
        &self,
    ) -> bool {
        matches!(self, Self::Full(_))
    }

    pub(in crate::graphics::scene::scene_renderer) fn black_texture_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.full_resources().black_texture_view
    }

    pub(in crate::graphics::scene::scene_renderer) fn white_texture_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.full_resources().white_texture_view
    }

    pub(in crate::graphics::scene::scene_renderer) fn default_exposure_buffer(
        &self,
    ) -> &wgpu::Buffer {
        &self.full_resources().default_exposure_buffer
    }

    pub(in crate::graphics::scene::scene_renderer) fn default_exposure_histogram_buffer(
        &self,
    ) -> &wgpu::Buffer {
        &self.full_resources().default_exposure_histogram_buffer
    }

    fn full_resources(&self) -> &FullScenePostProcessResources {
        match self {
            Self::Full(resources) => resources,
            Self::OutputTransferOnly(_) => {
                panic!(
                    "output-transfer-only resources do not support a compiled post-process graph"
                )
            }
        }
    }
}

impl Deref for ScenePostProcessResources {
    type Target = FullScenePostProcessResources;

    fn deref(&self) -> &Self::Target {
        self.full_resources()
    }
}
