use crate::core::framework::render::{
    source_cubemap_mip_size, EnvironmentCubemapUploadReport, SourceCubemapEnvironment,
    SourceCubemapIrradianceCube, SourceCubemapUploadKey, SourceCubemapUploadMip,
    RGBA16F_TEXEL_SIZE_BYTES, SOURCE_CUBEMAP_FACE_COUNT,
};
use crate::graphics::backend::SystemTextureGenerationLease;
use crate::graphics::types::GraphicsError;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::SceneEnvironmentBrdfLut;
use resources::CubemapResources;
use upload_batch::CubemapUploadStagingArena;

mod resources;
mod upload_batch;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneEnvironmentCubemap {
    resources: CubemapResources,
    pending_resources: Option<CubemapResources>,
    sampler: wgpu::Sampler,
    upload_state: CubemapUploadState,
    upload_staging: CubemapUploadStagingArena,
}

#[derive(Clone, Copy, Debug, Default)]
struct CubemapUploadState {
    committed: SourceCubemapUploadKey,
    pending: Option<SourceCubemapUploadKey>,
}

impl CubemapUploadState {
    fn new(committed: SourceCubemapUploadKey) -> Self {
        Self {
            committed,
            pending: None,
        }
    }

    fn committed(self) -> SourceCubemapUploadKey {
        self.committed
    }

    fn record(&mut self, upload_key: SourceCubemapUploadKey) {
        self.pending = Some(upload_key);
    }

    fn discard(&mut self) {
        self.pending = None;
    }

    fn commit(&mut self) {
        if let Some(upload_key) = self.pending.take() {
            self.committed = upload_key;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CubemapUploadChanges {
    source: bool,
    specular: bool,
    irradiance: bool,
}

fn cubemap_upload_changes(
    previous: SourceCubemapUploadKey,
    next: SourceCubemapUploadKey,
    requires_rebind: bool,
) -> CubemapUploadChanges {
    // A rebind replaces all views. Otherwise, upload only the payload affected by its key fields.
    let source = requires_rebind
        || previous.source_revision != next.source_revision
        || previous.source_hash != next.source_hash;
    CubemapUploadChanges {
        source,
        specular: requires_rebind || source || previous.pmrem_hash != next.pmrem_hash,
        irradiance: requires_rebind || previous.irradiance_cube_hash != next.irradiance_cube_hash,
    }
}

impl SceneEnvironmentCubemap {
    pub(in crate::graphics::scene::scene_renderer::core) fn fallback(
        system_textures: &SystemTextureGenerationLease,
    ) -> Self {
        let fallback_texture = system_textures.black_cube_texture().clone();
        let fallback_view = system_textures.black_cube_view().clone();
        let sampler = system_textures.linear_clamp_sampler().clone();
        Self {
            resources: CubemapResources {
                source_texture: fallback_texture.clone(),
                source_view: fallback_view.clone(),
                specular_texture: fallback_texture.clone(),
                specular_view: fallback_view.clone(),
                irradiance_texture: fallback_texture,
                irradiance_view: fallback_view,
                source_face_size: 1,
                source_mip_count: 1,
                pmrem_face_size: 1,
                pmrem_mip_count: 1,
                irradiance_face_size: 1,
                // The fallback views are generation-owned and shared; only textures allocated by
                // this scene environment are included in the logical destination budget.
                resident_source_texture_bytes: 0,
                resident_specular_texture_bytes: 0,
                resident_irradiance_texture_bytes: 0,
            },
            pending_resources: None,
            sampler,
            upload_state: CubemapUploadState::default(),
            upload_staging: CubemapUploadStagingArena::default(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn texture_layout_entry(
        binding: u32,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::Cube,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn sampler_layout_entry(
        binding: u32,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn bind_group_entries<'a>(
        &'a self,
        uniform_buffer: &'a wgpu::Buffer,
        brdf_lut: &'a SceneEnvironmentBrdfLut,
        environment_sh9: &'a wgpu::Buffer,
    ) -> [wgpu::BindGroupEntry<'a>; 7] {
        [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&self.frame_resources().source_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: brdf_lut.binding_resource(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&self.frame_resources().specular_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(
                    &self.frame_resources().irradiance_view,
                ),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: environment_sh9.as_entire_binding(),
            },
        ]
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn bind_group_entries_with_environment_views<
        'a,
    >(
        &'a self,
        uniform_buffer: &'a wgpu::Buffer,
        brdf_lut: &'a SceneEnvironmentBrdfLut,
        source_view: &'a wgpu::TextureView,
        specular_view: &'a wgpu::TextureView,
        environment_sh9: &'a wgpu::Buffer,
    ) -> [wgpu::BindGroupEntry<'a>; 7] {
        [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(source_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: brdf_lut.binding_resource(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(specular_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(
                    &self.frame_resources().irradiance_view,
                ),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: environment_sh9.as_entire_binding(),
            },
        ]
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn source_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.frame_resources().source_view
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn cold_fallback_texture(
        &self,
    ) -> &wgpu::Texture {
        &self.resources.source_texture
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn cold_fallback_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.resources.source_view
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn ensure_uploaded(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        environment: &SourceCubemapEnvironment,
        frame_uploads: &mut WgpuBufferUploadBatch,
    ) -> Result<bool, GraphicsError> {
        self.discard_pending_upload();
        let source_face_size = environment.mip_chain.source_face_size();
        let source_mip_count = environment.mip_chain.source_mip_count();
        let pmrem_face_size = environment.mip_chain.pmrem_face_size();
        let pmrem_mip_count = environment.mip_chain.pmrem_mip_count();
        let irradiance_face_size = environment
            .irradiance_cube()
            .map(SourceCubemapIrradianceCube::face_size)
            .unwrap_or(1);
        let requires_rebind = self.resources.resident_source_texture_bytes == 0
            || self.resources.source_face_size != source_face_size
            || self.resources.source_mip_count != source_mip_count
            || self.resources.pmrem_face_size != pmrem_face_size
            || self.resources.pmrem_mip_count != pmrem_mip_count
            || self.resources.irradiance_face_size != irradiance_face_size;

        let upload_key = environment.texture_upload_key();
        let changes =
            cubemap_upload_changes(self.upload_state.committed(), upload_key, requires_rebind);
        if !changes.source && !changes.specular && !changes.irradiance {
            return Ok(false);
        }

        let prepared_upload = environment.prepared_upload_artifact().ok_or_else(|| {
            GraphicsError::Asset(
                "source cubemap reached render submission without a current upload artifact"
                    .to_owned(),
            )
        })?;
        let source_mips = changes
            .source
            .then(|| prepared_upload.source_mips())
            .filter(|mips| cubemap_upload_mips_match(mips, source_face_size, source_mip_count));
        let pmrem_mips = changes
            .specular
            .then(|| prepared_upload.pmrem_mips())
            .filter(|mips| cubemap_upload_mips_match(mips, pmrem_face_size, pmrem_mip_count));
        let irradiance_mip = changes
            .irradiance
            .then(|| prepared_upload.irradiance_mip())
            .filter(|mip| mip.face_size() == irradiance_face_size);

        if changes.source && source_mips.is_none() {
            return Err(GraphicsError::Asset(
                "source cubemap upload artifact has an invalid source mip layout".to_owned(),
            ));
        }
        if changes.specular && pmrem_mips.is_none() {
            return Err(GraphicsError::Asset(
                "source cubemap upload artifact has an invalid PMREM mip layout".to_owned(),
            ));
        }
        if changes.irradiance && irradiance_mip.is_none() {
            return Err(GraphicsError::Asset(
                "source cubemap upload artifact has an invalid irradiance mip layout".to_owned(),
            ));
        }

        if requires_rebind {
            let source_texture = create_texture(
                device,
                source_face_size,
                source_mip_count,
                "zircon-scene-environment-source-cube",
            );
            let source_view = create_view(
                &source_texture,
                source_mip_count,
                "zircon-scene-environment-source-cube-view",
            );
            let specular_texture = create_texture(
                device,
                pmrem_face_size,
                pmrem_mip_count,
                "zircon-scene-environment-specular-pmrem-cube",
            );
            let specular_view = create_view(
                &specular_texture,
                pmrem_mip_count,
                "zircon-scene-environment-specular-pmrem-cube-view",
            );
            let irradiance_texture = create_texture(
                device,
                irradiance_face_size,
                1,
                "zircon-scene-environment-irradiance-cube",
            );
            let irradiance_view = create_view(
                &irradiance_texture,
                1,
                "zircon-scene-environment-irradiance-cube-view",
            );
            let prepared_uploads = [
                source_mips.map(|mips| (&source_texture, mips)),
                pmrem_mips.map(|mips| (&specular_texture, mips)),
                irradiance_mip.map(|mip| (&irradiance_texture, std::slice::from_ref(mip))),
            ];
            self.upload_staging
                .encode(device, encoder, &prepared_uploads, frame_uploads)
                .map_err(|error| GraphicsError::Asset(error.to_string()))?;

            self.pending_resources = Some(CubemapResources {
                source_texture,
                source_view,
                specular_texture,
                specular_view,
                irradiance_texture,
                irradiance_view,
                source_face_size,
                source_mip_count,
                pmrem_face_size,
                pmrem_mip_count,
                irradiance_face_size,
                resident_source_texture_bytes: cubemap_texture_texel_bytes(
                    source_face_size,
                    source_mip_count,
                ),
                resident_specular_texture_bytes: cubemap_texture_texel_bytes(
                    pmrem_face_size,
                    pmrem_mip_count,
                ),
                resident_irradiance_texture_bytes: cubemap_texture_texel_bytes(
                    irradiance_face_size,
                    1,
                ),
            });
        } else {
            let prepared_uploads = [
                source_mips.map(|mips| (&self.resources.source_texture, mips)),
                pmrem_mips.map(|mips| (&self.resources.specular_texture, mips)),
                irradiance_mip.map(|mip| {
                    (
                        &self.resources.irradiance_texture,
                        std::slice::from_ref(mip),
                    )
                }),
            ];
            self.upload_staging
                .encode(device, encoder, &prepared_uploads, frame_uploads)
                .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        }
        self.upload_state.record(upload_key);
        Ok(requires_rebind)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn begin_frame(&mut self) {
        self.discard_pending_upload();
        self.upload_staging.begin_observation();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn discard_pending_upload(&mut self) {
        self.pending_resources = None;
        self.upload_state.discard();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn commit_pending_upload(&mut self) {
        if let Some(resources) = self.pending_resources.take() {
            self.resources = resources;
        }
        self.upload_state.commit();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn has_pending_rebind(&self) -> bool {
        self.pending_resources.is_some()
    }

    fn frame_resources(&self) -> &CubemapResources {
        self.pending_resources.as_ref().unwrap_or(&self.resources)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn upload_report(
        &self,
    ) -> EnvironmentCubemapUploadReport {
        let resident_texture_bytes = self
            .resources
            .resident_source_texture_bytes
            .saturating_add(self.resources.resident_specular_texture_bytes)
            .saturating_add(self.resources.resident_irradiance_texture_bytes);
        self.upload_staging.report(
            self.upload_state.committed,
            self.upload_state.pending,
            self.resources.resident_source_texture_bytes,
            self.resources.resident_specular_texture_bytes,
            self.resources.resident_irradiance_texture_bytes,
            resident_texture_bytes,
        )
    }
}

fn cubemap_texture_texel_bytes(face_size: u32, mip_count: u32) -> u64 {
    let mut total_texels = 0u64;
    for mip_level in 0..mip_count.max(1) {
        let mip_size = u64::from(source_cubemap_mip_size(face_size.max(1), mip_level));
        total_texels = total_texels.saturating_add(mip_size.saturating_mul(mip_size));
    }
    total_texels
        .saturating_mul(u64::from(SOURCE_CUBEMAP_FACE_COUNT as u32))
        .saturating_mul(RGBA16F_TEXEL_SIZE_BYTES as u64)
}

fn create_texture(
    device: &wgpu::Device,
    face_size: u32,
    mip_count: u32,
    label: &'static str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: face_size.max(1),
            height: face_size.max(1),
            depth_or_array_layers: 6,
        },
        mip_level_count: mip_count.max(1),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn create_view(texture: &wgpu::Texture, mip_count: u32, label: &'static str) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(label),
        format: Some(wgpu::TextureFormat::Rgba16Float),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(mip_count.max(1)),
        base_array_layer: 0,
        array_layer_count: Some(6),
    })
}

fn cubemap_upload_mips_match(
    mips: &[SourceCubemapUploadMip],
    face_size: u32,
    mip_count: u32,
) -> bool {
    mips.len() == mip_count as usize
        && mips.iter().zip(0..mip_count).all(|(mip, mip_level)| {
            mip.mip_level() == mip_level
                && mip.face_size() == source_cubemap_mip_size(face_size, mip_level)
        })
}

#[cfg(test)]
#[path = "tests/environment_cubemap.rs"]
mod tests;
