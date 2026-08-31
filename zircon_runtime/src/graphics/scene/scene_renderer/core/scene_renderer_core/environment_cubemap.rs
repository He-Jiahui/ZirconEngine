use crate::core::framework::render::{
    SourceCubemapEnvironment, SourceCubemapIrradianceCube, SourceCubemapUploadKey,
    SourceCubemapUploadMip, source_cubemap_mip_size,
};
use crate::graphics::backend::SystemTextureGenerationLease;
use crate::graphics::types::GraphicsError;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::SceneEnvironmentBrdfLut;
use upload_batch::CubemapUploadStagingArena;

mod upload_batch;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneEnvironmentCubemap {
    source_texture: wgpu::Texture,
    source_view: wgpu::TextureView,
    specular_texture: wgpu::Texture,
    specular_view: wgpu::TextureView,
    irradiance_texture: wgpu::Texture,
    irradiance_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    source_face_size: u32,
    source_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    irradiance_face_size: u32,
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
            source_texture: fallback_texture.clone(),
            source_view: fallback_view.clone(),
            specular_texture: fallback_texture.clone(),
            specular_view: fallback_view.clone(),
            irradiance_texture: fallback_texture,
            irradiance_view: fallback_view,
            sampler,
            source_face_size: 1,
            source_mip_count: 1,
            pmrem_face_size: 1,
            pmrem_mip_count: 1,
            irradiance_face_size: 1,
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
                resource: wgpu::BindingResource::TextureView(&self.source_view),
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
                resource: wgpu::BindingResource::TextureView(&self.specular_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&self.irradiance_view),
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
                resource: wgpu::BindingResource::TextureView(&self.irradiance_view),
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
        &self.source_view
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn cold_fallback_texture(
        &self,
    ) -> &wgpu::Texture {
        &self.source_texture
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn cold_fallback_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.source_view
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
        self.upload_state.discard();
        let source_face_size = environment.mip_chain.source_face_size();
        let source_mip_count = environment.mip_chain.source_mip_count();
        let pmrem_face_size = environment.mip_chain.pmrem_face_size();
        let pmrem_mip_count = environment.mip_chain.pmrem_mip_count();
        let irradiance_face_size = environment
            .irradiance_cube()
            .map(SourceCubemapIrradianceCube::face_size)
            .unwrap_or(1);
        let requires_rebind = self.source_face_size != source_face_size
            || self.source_mip_count != source_mip_count
            || self.pmrem_face_size != pmrem_face_size
            || self.pmrem_mip_count != pmrem_mip_count
            || self.irradiance_face_size != irradiance_face_size;

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

            self.source_texture = source_texture;
            self.source_view = source_view;
            self.specular_texture = specular_texture;
            self.specular_view = specular_view;
            self.irradiance_texture = irradiance_texture;
            self.irradiance_view = irradiance_view;
            self.source_face_size = source_face_size;
            self.source_mip_count = source_mip_count;
            self.pmrem_face_size = pmrem_face_size;
            self.pmrem_mip_count = pmrem_mip_count;
            self.irradiance_face_size = irradiance_face_size;
            self.upload_state = CubemapUploadState::default();
        } else {
            let prepared_uploads = [
                source_mips.map(|mips| (&self.source_texture, mips)),
                pmrem_mips.map(|mips| (&self.specular_texture, mips)),
                irradiance_mip.map(|mip| (&self.irradiance_texture, std::slice::from_ref(mip))),
            ];
            self.upload_staging
                .encode(device, encoder, &prepared_uploads, frame_uploads)
                .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        }
        self.upload_state.record(upload_key);
        Ok(requires_rebind)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn discard_pending_upload(&mut self) {
        self.upload_state.discard();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn commit_pending_upload(&mut self) {
        self.upload_state.commit();
    }
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
mod tests {
    use super::*;

    #[test]
    fn dynamic_cubemap_upload_has_no_queue_or_render_thread_texel_fallback() {
        let source = include_str!("environment_cubemap.rs");
        let product = source
            .split("#[cfg(test)]")
            .next()
            .expect("product source precedes tests");
        let dynamic_upload = product
            .split("fn ensure_uploaded(")
            .nth(1)
            .and_then(|source| source.split("fn discard_pending_upload").next())
            .expect("dynamic upload owner must remain bounded");

        assert!(!dynamic_upload.contains("queue:"));
        assert!(!dynamic_upload.contains("queue.write_buffer("));
        assert!(!dynamic_upload.contains("queue.write_texture("));
        assert!(!dynamic_upload.contains("source_texels()"));
        assert!(!dynamic_upload.contains("pmrem_texels()"));
        assert!(!dynamic_upload.contains("upload_cubemap_texels("));
        assert!(!dynamic_upload.contains("create_sampler(device)"));
        assert!(!dynamic_upload.contains("self.sampler ="));
    }

    #[test]
    fn fallback_slots_share_one_generation_owned_black_cube_and_sampler() {
        let source = include_str!("environment_cubemap.rs");
        let fallback = source
            .split("fn fallback(")
            .nth(1)
            .and_then(|source| source.split("fn texture_layout_entry").next())
            .expect("fallback construction must remain bounded");

        assert_eq!(fallback.matches("create_texture(").count(), 0);
        assert_eq!(fallback.matches("create_view(").count(), 0);
        assert_eq!(fallback.matches("create_sampler(").count(), 0);
        assert_eq!(fallback.matches("write_texture(").count(), 0);
        assert!(fallback.contains("system_textures.black_cube_texture().clone()"));
        assert!(fallback.contains("system_textures.black_cube_view().clone()"));
        assert!(fallback.contains("system_textures.linear_clamp_sampler().clone()"));
        assert_eq!(fallback.matches("fallback_texture.clone()").count(), 2);
        assert_eq!(fallback.matches("fallback_view.clone()").count(), 2);
    }

    #[test]
    fn cubemap_upload_changes_skip_unaffected_texture_groups() {
        let current = upload_key(1, [1; 4], [2; 4], [3; 4]);

        assert_eq!(
            cubemap_upload_changes(current, current, false),
            CubemapUploadChanges {
                source: false,
                specular: false,
                irradiance: false,
            }
        );
        assert_eq!(
            cubemap_upload_changes(current, upload_key(1, [1; 4], [2; 4], [4; 4]), false),
            CubemapUploadChanges {
                source: false,
                specular: false,
                irradiance: true,
            }
        );
        assert_eq!(
            cubemap_upload_changes(current, upload_key(1, [1; 4], [4; 4], [3; 4]), false),
            CubemapUploadChanges {
                source: false,
                specular: true,
                irradiance: false,
            }
        );
        assert_eq!(
            cubemap_upload_changes(current, upload_key(2, [1; 4], [2; 4], [3; 4]), false),
            CubemapUploadChanges {
                source: true,
                specular: true,
                irradiance: false,
            }
        );
        assert_eq!(
            cubemap_upload_changes(current, upload_key(1, [4; 4], [2; 4], [3; 4]), false),
            CubemapUploadChanges {
                source: true,
                specular: true,
                irradiance: false,
            }
        );
        assert_eq!(
            cubemap_upload_changes(current, current, true),
            CubemapUploadChanges {
                source: true,
                specular: true,
                irradiance: true,
            }
        );
    }

    #[test]
    fn cubemap_upload_key_advances_only_after_frame_submission() {
        let previous = upload_key(1, [1; 4], [2; 4], [3; 4]);
        let next = upload_key(2, [4; 4], [5; 4], [6; 4]);
        let mut state = CubemapUploadState::new(previous);

        state.record(next);
        assert_eq!(state.committed(), previous);
        state.discard();
        assert_eq!(state.committed(), previous);

        state.record(next);
        state.commit();
        assert_eq!(state.committed(), next);
    }

    #[test]
    fn prepared_cubemap_upload_batches_all_faces_for_each_mip() {
        let source = include_str!("environment_cubemap.rs");
        let product = source
            .split("#[cfg(test)]")
            .next()
            .expect("product source precedes tests");

        assert!(product.contains("CubemapUploadStagingArena"));
        assert!(product.contains(".encode(device, encoder, &prepared_uploads, frame_uploads)"));
        assert!(product.contains("self.upload_state.record(upload_key);"));
        assert!(!product.contains("self.upload_key = upload_key;"));
    }

    #[test]
    fn cubemap_upload_validates_before_rebind_and_records_after_staging() {
        let source = include_str!("environment_cubemap.rs");
        let dynamic_upload = source
            .split("fn ensure_uploaded(")
            .nth(1)
            .and_then(|source| source.split("fn discard_pending_upload").next())
            .expect("dynamic upload owner must remain bounded");
        let validate = dynamic_upload
            .find("environment.prepared_upload_artifact()")
            .expect("artifact must be validated");
        let rebind = dynamic_upload
            .find("if requires_rebind")
            .expect("resource replacement must remain explicit");
        let stage = dynamic_upload
            .find(".encode(device, encoder, &prepared_uploads, frame_uploads)")
            .expect("staging must enter the caller frame upload batch");
        let publish = dynamic_upload
            .find("self.source_texture = source_texture")
            .expect("new cubemap resources must publish explicitly");
        let record = dynamic_upload
            .find("self.upload_state.record(upload_key)")
            .expect("pending upload identity must be recorded");

        assert!(validate < rebind);
        assert!(rebind < stage);
        assert!(stage < publish);
        assert!(publish < record);
    }

    fn upload_key(
        source_revision: u64,
        source_hash: [u32; 4],
        pmrem_hash: [u32; 4],
        irradiance_cube_hash: [u32; 4],
    ) -> SourceCubemapUploadKey {
        SourceCubemapUploadKey {
            source_revision,
            source_hash,
            pmrem_hash,
            irradiance_cube_hash,
        }
    }
}
