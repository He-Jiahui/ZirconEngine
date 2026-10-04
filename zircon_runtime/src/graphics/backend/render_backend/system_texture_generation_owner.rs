//! 每个设备代际只发布一组系统纹理；首次提交上传后，材质与场景路径共享原生资源租约。
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::graphics::types::GraphicsError;
use zr_rhi::{DeviceGeneration, DeviceId, RenderDevice};
use zr_rhi_wgpu::WgpuRenderDevice;

mod payloads;
mod resources;

use resources::SystemTextureResources;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SystemTexturePayloadCacheState {
    Materialized,
    Reused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SystemTextureGenerationStartupReport {
    builtin_payload_cache_state: SystemTexturePayloadCacheState,
    builtin_payload_cache_wait: Duration,
    builtin_payload_materialization: Duration,
    texture_upload_submission: Duration,
    texture_upload_ticket: Option<zr_rhi::SubmissionTicket>,
    native_submission_count: usize,
    texture_upload_count: usize,
    texture_upload_bytes: u64,
}

impl SystemTextureGenerationStartupReport {
    pub(crate) const fn builtin_payload_cache_state(self) -> SystemTexturePayloadCacheState {
        self.builtin_payload_cache_state
    }

    pub(crate) const fn builtin_payload_cache_wait(self) -> Duration {
        self.builtin_payload_cache_wait
    }

    pub(crate) const fn builtin_payload_materialization(self) -> Duration {
        self.builtin_payload_materialization
    }

    pub(crate) const fn texture_upload_submission(self) -> Duration {
        self.texture_upload_submission
    }

    pub(crate) const fn texture_upload_ticket(self) -> Option<zr_rhi::SubmissionTicket> {
        self.texture_upload_ticket
    }

    pub(crate) const fn texture_upload_count(self) -> usize {
        self.texture_upload_count
    }

    pub(crate) const fn native_submission_count(self) -> usize {
        self.native_submission_count
    }

    pub(crate) const fn texture_upload_bytes(self) -> u64 {
        self.texture_upload_bytes
    }
}

/// Immutable native resources for exactly one WGPU device generation.
pub(crate) struct SystemTextureGenerationOwner {
    device_id: DeviceId,
    generation: DeviceGeneration,
    published: Mutex<Option<SystemTextureGenerationLease>>,
}

#[derive(Clone)]
pub(crate) struct SystemTextureGenerationLease {
    device_id: DeviceId,
    generation: DeviceGeneration,
    resources: SystemTextureResources,
}

impl SystemTextureGenerationOwner {
    pub(crate) fn new(render_device: &WgpuRenderDevice) -> Self {
        let profile = render_device.profile();
        Self {
            device_id: profile.device_id(),
            generation: profile.generation(),
            published: Mutex::new(None),
        }
    }

    pub(crate) fn acquire(
        &self,
        render_device: &WgpuRenderDevice,
        device: &wgpu::Device,
    ) -> Result<
        (
            SystemTextureGenerationLease,
            SystemTextureGenerationStartupReport,
        ),
        GraphicsError,
    > {
        let profile = render_device.profile();
        if profile.device_id() != self.device_id || profile.generation() != self.generation {
            return Err(GraphicsError::WgpuValidation(format!(
                "system texture owner belongs to device {} generation {}, received device {} generation {}",
                self.device_id.raw(),
                self.generation.raw(),
                profile.device_id().raw(),
                profile.generation().raw(),
            )));
        }
        let mut published = self
            .published
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(lease) = published.as_ref() {
            return Ok((
                lease.clone(),
                SystemTextureGenerationStartupReport {
                    builtin_payload_cache_state: SystemTexturePayloadCacheState::Reused,
                    builtin_payload_cache_wait: Duration::ZERO,
                    builtin_payload_materialization: Duration::ZERO,
                    texture_upload_submission: Duration::ZERO,
                    texture_upload_ticket: None,
                    native_submission_count: 0,
                    texture_upload_count: 0,
                    texture_upload_bytes: 0,
                },
            ));
        }

        let builtin_payload_cache_started = Instant::now();
        let (brdf_lut_payload, builtin_payload_cache_state, builtin_payload_materialization) =
            cached_builtin_environment_brdf_lut_rg16float_bytes_with_state();
        let builtin_payload_cache_wait = builtin_payload_cache_started.elapsed();
        let resources::PreparedSystemTextureResources {
            resources,
            uploads,
            upload_count: texture_upload_count,
            upload_bytes: texture_upload_bytes,
        } = SystemTextureResources::prepare(device, brdf_lut_payload)?;

        let texture_upload_submission_started = Instant::now();
        let texture_upload_ticket = render_device.enqueue_native_texture_upload_batch(uploads)?;
        let native_submission_count = render_device.flush_submissions()?;
        let texture_upload_submission = texture_upload_submission_started.elapsed();
        let lease = SystemTextureGenerationLease {
            device_id: self.device_id,
            generation: self.generation,
            resources,
        };
        let report = SystemTextureGenerationStartupReport {
            builtin_payload_cache_state,
            builtin_payload_cache_wait,
            builtin_payload_materialization,
            texture_upload_submission,
            texture_upload_ticket: Some(texture_upload_ticket),
            native_submission_count,
            texture_upload_count,
            texture_upload_bytes,
        };
        *published = Some(lease.clone());
        Ok((lease, report))
    }
}

impl SystemTextureGenerationLease {
    pub(crate) const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub(crate) const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub(crate) fn black_cube_texture(&self) -> &wgpu::Texture {
        self.resources.black_cube_texture()
    }

    pub(crate) fn black_cube_view(&self) -> &wgpu::TextureView {
        self.resources.black_cube_view()
    }

    pub(crate) fn brdf_lut_texture(&self) -> &wgpu::Texture {
        self.resources.brdf_lut_texture()
    }

    pub(crate) fn brdf_lut_view(&self) -> &wgpu::TextureView {
        self.resources.brdf_lut_view()
    }

    pub(crate) fn black_rgba8_texture(&self) -> &wgpu::Texture {
        self.resources.black_rgba8_texture()
    }

    pub(crate) fn black_rgba8_view(&self) -> &wgpu::TextureView {
        self.resources.black_rgba8_view()
    }

    pub(crate) fn black_alpha_one_rgba8_texture(&self) -> &wgpu::Texture {
        self.resources.black_alpha_one_rgba8_texture()
    }

    pub(crate) fn black_alpha_one_rgba8_view(&self) -> &wgpu::TextureView {
        self.resources.black_alpha_one_rgba8_view()
    }

    pub(crate) fn white_rgba8_texture(&self) -> &wgpu::Texture {
        self.resources.white_rgba8_texture()
    }

    pub(crate) fn white_rgba8_view(&self) -> &wgpu::TextureView {
        self.resources.white_rgba8_view()
    }

    pub(crate) fn white_rgba8_srgb_view(&self) -> &wgpu::TextureView {
        self.resources.white_rgba8_srgb_view()
    }

    pub(crate) fn normal_rgba8_texture(&self) -> &wgpu::Texture {
        self.resources.normal_rgba8_texture()
    }

    pub(crate) fn normal_rgba8_view(&self) -> &wgpu::TextureView {
        self.resources.normal_rgba8_view()
    }

    pub(crate) fn black_rgba16float_texture(&self) -> &wgpu::Texture {
        self.resources.black_rgba16float_texture()
    }

    pub(crate) fn black_rgba16float_view(&self) -> &wgpu::TextureView {
        self.resources.black_rgba16float_view()
    }

    pub(crate) fn black_rgba16float_array_view(&self) -> &wgpu::TextureView {
        self.resources.black_rgba16float_array_view()
    }

    pub(crate) fn irradiance_volume_black_texture(&self) -> &wgpu::Texture {
        self.resources.irradiance_volume_black_texture()
    }

    pub(crate) fn irradiance_volume_black_view(&self) -> &wgpu::TextureView {
        self.resources.irradiance_volume_black_view()
    }

    pub(crate) fn effect_lut_texture(&self) -> &wgpu::Texture {
        self.resources.effect_lut_texture()
    }

    pub(crate) fn effect_lut_view(&self) -> &wgpu::TextureView {
        self.resources.effect_lut_view()
    }

    pub(crate) fn effect_lut_3d_texture(&self) -> &wgpu::Texture {
        self.resources.effect_lut_3d_texture()
    }

    pub(crate) fn effect_lut_3d_view(&self) -> &wgpu::TextureView {
        self.resources.effect_lut_3d_view()
    }

    pub(crate) fn linear_clamp_sampler(&self) -> &wgpu::Sampler {
        self.resources.linear_clamp_sampler()
    }
}

fn cached_builtin_environment_brdf_lut_rg16float_bytes_with_state(
) -> (Arc<[u8]>, SystemTexturePayloadCacheState, Duration) {
    static ENCODED_BYTES: OnceLock<Arc<[u8]>> = OnceLock::new();
    let mut payload_materialization = Duration::ZERO;
    let mut materialized = false;
    let payload = ENCODED_BYTES.get_or_init(|| {
        materialized = true;
        let materialization_started = Instant::now();
        let bytes = payloads::builtin_environment_brdf_lut_rg16float_bytes();
        payload_materialization = materialization_started.elapsed();
        bytes
    });
    let state = if materialized {
        SystemTexturePayloadCacheState::Materialized
    } else {
        SystemTexturePayloadCacheState::Reused
    };
    (Arc::clone(payload), state, payload_materialization)
}

#[cfg(test)]
#[path = "tests/system_texture_generation_owner.rs"]
mod tests;
