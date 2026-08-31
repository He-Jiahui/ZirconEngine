use std::ops::Deref;

use super::{FullScenePostProcessResources, SceneOutputTransferResources};
use crate::core::math::UVec2;
use crate::graphics::CompiledAoProfile;
use crate::graphics::scene::scene_renderer::post_process::params::ssao_params::SsaoParams;
use crate::rhi::{BufferDesc, BufferUsage};
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

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

    pub(in crate::graphics::scene::scene_renderer) fn default_exposure_buffer_binding(
        &self,
    ) -> wgpu::BufferBinding<'_> {
        wgpu::BufferBinding {
            buffer: self.default_exposure_buffer(),
            offset: 0,
            size: None,
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn default_exposure_histogram_buffer(
        &self,
    ) -> &wgpu::Buffer {
        &self.full_resources().default_exposure_histogram_buffer
    }

    pub(in crate::graphics::scene::scene_renderer) fn default_exposure_histogram_buffer_binding(
        &self,
    ) -> wgpu::BufferBinding<'_> {
        wgpu::BufferBinding {
            buffer: self.default_exposure_histogram_buffer(),
            offset: 0,
            size: None,
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn ssao_params_buffer(&self) -> &wgpu::Buffer {
        &self.full_resources().ssao_params_buffer
    }

    pub(in crate::graphics::scene::scene_renderer) fn ssao_params_buffer_desc(
        &self,
        label: &'static str,
    ) -> BufferDesc {
        BufferDesc::new(
            label,
            self.ssao_params_buffer().size(),
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn prepare_ssao_compute_params_upload(
        &self,
        profile: &CompiledAoProfile,
        runtime_extent: UVec2,
        frame_uploads: &mut WgpuBufferUploadBatch,
    ) -> Result<(), String> {
        let params = SsaoParams::from_compiled_profile(profile, runtime_extent)?;
        frame_uploads.push(WgpuBufferUpload::from_bytes(
            self.full_resources().ssao_params_buffer.clone(),
            0,
            bytemuck::bytes_of(&params),
        ));
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::SsaoParams;

    #[test]
    fn ssao_params_are_prepared_into_the_frame_upload_transaction() {
        let source = include_str!("profiled_scene_post_process_resources.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("profiled post-process resources source");

        assert!(!production.contains("queue.write_buffer"));
        assert!(!production.contains("queue: &wgpu::Queue"));
        assert!(production.contains("WgpuBufferUpload::from_bytes("));
        assert!(production.contains("frame_uploads.push("));
        assert!(production.contains("SsaoParams::from_compiled_profile("));
        assert!(!production.contains("tuning: ["));
    }

    #[test]
    fn ssao_params_share_the_feature_owned_abi_layout() {
        assert_eq!(
            std::mem::size_of::<SsaoParams>(),
            std::mem::size_of::<([u32; 4], [u32; 4], [f32; 4], [f32; 4])>()
        );
    }
}
