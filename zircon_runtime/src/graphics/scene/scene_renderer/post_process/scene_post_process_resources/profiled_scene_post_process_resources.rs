use std::ops::Deref;

use super::{FullScenePostProcessResources, SceneOutputTransferResources};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::params::ssao_params::SsaoParams;

pub(crate) enum ScenePostProcessResources {
    Full(FullScenePostProcessResources),
    OutputTransferOnly(SceneOutputTransferResources),
}

impl ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer) const fn has_full_resources(&self) -> bool {
        matches!(self, Self::Full(_))
    }

    pub(in crate::graphics::scene::scene_renderer) fn black_texture_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.full_resources().black_texture_view
    }

    pub(in crate::graphics::scene::scene_renderer) fn black_texture_identity(
        &self,
    ) -> crate::graphics::resource_identity::SampledTextureIdentity {
        self.full_resources().black_texture_identity
    }

    pub(in crate::graphics::scene::scene_renderer) fn invalidate_taa_resolve_bind_group_cache(
        &self,
    ) {
        let Self::Full(resources) = self else {
            return;
        };
        resources
            .taa_resolve_bind_group_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    pub(in crate::graphics::scene::scene_renderer) fn white_texture_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.full_resources().white_texture_view
    }

    pub(in crate::graphics::scene::scene_renderer) fn hzb_fallback_resource_identity(
        &self,
    ) -> crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity {
        self.full_resources().hzb_fallback_resource_identity
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

    pub(in crate::graphics::scene::scene_renderer) fn ssao_params_buffer(&self) -> &wgpu::Buffer {
        &self.full_resources().ssao_params_buffer
    }

    pub(in crate::graphics::scene::scene_renderer) fn write_ssao_compute_params(
        &self,
        queue: &wgpu::Queue,
        viewport_size: UVec2,
        history_available: bool,
        enabled: bool,
    ) {
        let params = SsaoParams {
            viewport_and_flags: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                u32::from(history_available),
                u32::from(enabled),
            ],
            tuning: [4.6, 0.0015, 0.18, 0.88],
        };
        queue.write_buffer(
            &self.full_resources().ssao_params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );
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
