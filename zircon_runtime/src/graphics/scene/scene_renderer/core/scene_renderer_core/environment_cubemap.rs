use crate::core::framework::render::{
    source_cubemap_face_mip_offset, source_cubemap_mip_size, SourceCubemapEnvironment,
    SourceCubemapIrradianceCube, SourceCubemapUploadKey,
};

use super::half_float::push_f16_le_bytes;
use super::SceneEnvironmentBrdfLut;

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
    upload_key: SourceCubemapUploadKey,
}

impl SceneEnvironmentCubemap {
    pub(in crate::graphics::scene::scene_renderer::core) fn fallback(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let source_texture = create_texture(device, 1, 1, "zircon-scene-environment-source-cube");
        let source_view = create_view(
            &source_texture,
            1,
            "zircon-scene-environment-source-cube-view",
        );
        let specular_texture =
            create_texture(device, 1, 1, "zircon-scene-environment-specular-pmrem-cube");
        let specular_view = create_view(
            &specular_texture,
            1,
            "zircon-scene-environment-specular-pmrem-cube-view",
        );
        let irradiance_texture =
            create_texture(device, 1, 1, "zircon-scene-environment-irradiance-cube");
        let irradiance_view = create_view(
            &irradiance_texture,
            1,
            "zircon-scene-environment-irradiance-cube-view",
        );
        let sampler = create_sampler(device);
        upload_single_rgba16_cubemap(queue, &source_texture, [0.0, 0.0, 0.0, 1.0]);
        upload_single_rgba16_cubemap(queue, &specular_texture, [0.0, 0.0, 0.0, 1.0]);
        upload_single_rgba16_cubemap(queue, &irradiance_texture, [0.0, 0.0, 0.0, 1.0]);
        Self {
            source_texture,
            source_view,
            specular_texture,
            specular_view,
            irradiance_texture,
            irradiance_view,
            sampler,
            source_face_size: 1,
            source_mip_count: 1,
            pmrem_face_size: 1,
            pmrem_mip_count: 1,
            irradiance_face_size: 1,
            upload_key: SourceCubemapUploadKey::default(),
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

    pub(in crate::graphics::scene::scene_renderer::core) fn ensure_uploaded(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        environment: &SourceCubemapEnvironment,
    ) -> bool {
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
        if requires_rebind {
            self.source_texture = create_texture(
                device,
                source_face_size,
                source_mip_count,
                "zircon-scene-environment-source-cube",
            );
            self.source_view = create_view(
                &self.source_texture,
                source_mip_count,
                "zircon-scene-environment-source-cube-view",
            );
            self.specular_texture = create_texture(
                device,
                pmrem_face_size,
                pmrem_mip_count,
                "zircon-scene-environment-specular-pmrem-cube",
            );
            self.specular_view = create_view(
                &self.specular_texture,
                pmrem_mip_count,
                "zircon-scene-environment-specular-pmrem-cube-view",
            );
            self.irradiance_texture = create_texture(
                device,
                irradiance_face_size,
                1,
                "zircon-scene-environment-irradiance-cube",
            );
            self.irradiance_view = create_view(
                &self.irradiance_texture,
                1,
                "zircon-scene-environment-irradiance-cube-view",
            );
            self.sampler = create_sampler(device);
            self.source_face_size = source_face_size;
            self.source_mip_count = source_mip_count;
            self.pmrem_face_size = pmrem_face_size;
            self.pmrem_mip_count = pmrem_mip_count;
            self.irradiance_face_size = irradiance_face_size;
            self.upload_key = SourceCubemapUploadKey::default();
        }

        let upload_key = environment.texture_upload_key();
        if !requires_rebind && self.upload_key == upload_key {
            return false;
        }

        upload_cubemap_texels(
            queue,
            &self.source_texture,
            source_face_size,
            source_mip_count,
            environment.mip_chain.source_texels(),
        );
        upload_cubemap_texels(
            queue,
            &self.specular_texture,
            pmrem_face_size,
            pmrem_mip_count,
            environment.mip_chain.pmrem_texels(),
        );
        if let Some(irradiance_cube) = environment.irradiance_cube() {
            upload_irradiance_cube_texels(queue, &self.irradiance_texture, irradiance_cube);
        } else {
            upload_single_rgba16_cubemap(queue, &self.irradiance_texture, [0.0, 0.0, 0.0, 1.0]);
        }
        self.upload_key = upload_key;
        requires_rebind
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

fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-scene-environment-source-cube-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    })
}

fn upload_single_rgba16_cubemap(queue: &wgpu::Queue, texture: &wgpu::Texture, texel: [f32; 4]) {
    let byte_data = rgba16float_texels(&[texel]);
    for face in 0..6 {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: face,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &byte_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn upload_cubemap_texels(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    face_size: u32,
    mip_count: u32,
    texels: &[[f32; 4]],
) {
    for face_index in 0..6 {
        let face = crate::core::framework::render::CubemapFace::from_index(face_index)
            .expect("source cubemap face index must be in range");
        for mip in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip);
            let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
            let byte_data =
                rgba16float_texels(&texels[offset..offset + mip_size as usize * mip_size as usize]);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: mip,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face_index as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &byte_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(8 * mip_size),
                    rows_per_image: Some(mip_size),
                },
                wgpu::Extent3d {
                    width: mip_size,
                    height: mip_size,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}

fn upload_irradiance_cube_texels(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    irradiance_cube: &SourceCubemapIrradianceCube,
) {
    let face_size = irradiance_cube.face_size();
    for face_index in 0..6 {
        let face = crate::core::framework::render::CubemapFace::from_index(face_index)
            .expect("source irradiance cubemap face index must be in range");
        let mut texels = Vec::with_capacity(face_size as usize * face_size as usize);
        for y in 0..face_size {
            for x in 0..face_size {
                let texel = irradiance_cube.texel(face, x, y);
                texels.push([texel[0], texel[1], texel[2], 1.0]);
            }
        }
        let byte_data = rgba16float_texels(&texels);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: face_index as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &byte_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8 * face_size),
                rows_per_image: Some(face_size),
            },
            wgpu::Extent3d {
                width: face_size,
                height: face_size,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn rgba16float_texels(texels: &[[f32; 4]]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(texels.len() * 8);
    for texel in texels {
        for channel in texel {
            push_f16_le_bytes(&mut bytes, *channel);
        }
    }
    bytes
}
