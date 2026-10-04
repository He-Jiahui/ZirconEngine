use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use crate::core::framework::render::{IrradianceVolumeData, RenderImageDimension};
use crate::graphics::backend::SystemTextureGenerationLease;
use crate::graphics::scene::resources::IrradianceVolumeTextureBinding;

pub(crate) const IRRADIANCE_VOLUME_TEXTURE_BINDING: u32 = 35;
pub(crate) const IRRADIANCE_VOLUME_SAMPLER_BINDING: u32 = 36;
pub(crate) const IRRADIANCE_VOLUME_PARAMS_BINDING: u32 = 37;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuIrradianceVolumeParams {
    world_to_volume: [[f32; 4]; 4],
    intensity_enabled: [f32; 4],
    flags: [u32; 4],
    normal_to_volume: [[f32; 4]; 3],
}

impl GpuIrradianceVolumeParams {
    fn disabled() -> Self {
        Self {
            world_to_volume: crate::core::math::Mat4::IDENTITY.to_cols_array_2d(),
            intensity_enabled: [0.0; 4],
            flags: [0; 4],
            normal_to_volume: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
        }
    }

    fn from_volume(volume: &IrradianceVolumeData) -> Self {
        let normal_matrix = volume.transform.inverse().transpose().to_cols_array_2d();
        Self {
            world_to_volume: volume.transform.to_cols_array_2d(),
            intensity_enabled: [volume.intensity.max(0.0), 1.0, 0.0, 0.0],
            flags: [u32::from(volume.affects_lightmapped_meshes), 0, 0, 0],
            normal_to_volume: [
                [
                    normal_matrix[0][0],
                    normal_matrix[0][1],
                    normal_matrix[0][2],
                    0.0,
                ],
                [
                    normal_matrix[1][0],
                    normal_matrix[1][1],
                    normal_matrix[1][2],
                    0.0,
                ],
                [
                    normal_matrix[2][0],
                    normal_matrix[2][1],
                    normal_matrix[2][2],
                    0.0,
                ],
            ],
        }
    }
}

/// 持有当前视图的 3D 辐照度纹理及禁用时的合法回退绑定。
/// 视图选择由帧基础阶段完成，图通道只读取已经准备好的绑定。
pub(crate) struct IrradianceVolumeResources {
    fallback_texture: wgpu::Texture,
    fallback_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    selected_texture: Option<IrradianceVolumeTextureBinding>,
}

impl IrradianceVolumeResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        system_textures: &SystemTextureGenerationLease,
    ) -> Self {
        let fallback_texture = system_textures.irradiance_volume_black_texture().clone();
        let fallback_view = system_textures.irradiance_volume_black_view().clone();
        let sampler = system_textures.linear_clamp_sampler().clone();
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-irradiance-volume-params"),
            contents: bytemuck::bytes_of(&GpuIrradianceVolumeParams::disabled()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            fallback_texture,
            fallback_view,
            sampler,
            params_buffer,
            selected_texture: None,
        }
    }

    /// 在渲染图提交前验证纹理的打包维度并将参数加入同一帧上传批次；
    /// 无有效体积时上传禁用参数并绑定黑色回退纹理。
    pub(crate) fn prepare(
        &mut self,
        selected: Option<(IrradianceVolumeData, IrradianceVolumeTextureBinding)>,
        frame_batch: &mut WgpuBufferUploadBatch,
    ) -> Result<(), String> {
        let selected = selected.filter(|(_, texture)| {
            let descriptor = texture.descriptor();
            descriptor.dimension == RenderImageDimension::D3
                && descriptor.height >= 2
                && descriptor.height % 2 == 0
                && descriptor.depth_or_array_layers >= 3
                && descriptor.depth_or_array_layers % 3 == 0
        });
        let params = selected
            .as_ref()
            .map(|(volume, _)| GpuIrradianceVolumeParams::from_volume(volume))
            .unwrap_or_else(GpuIrradianceVolumeParams::disabled);
        let payload: Arc<[u8]> = Arc::from(bytemuck::bytes_of(&params));
        let payload_byte_len = payload.len();
        let Some(upload) =
            WgpuBufferUpload::new(self.params_buffer.clone(), 0, payload, 0..payload_byte_len)
        else {
            return Err(
                "irradiance volume params upload does not match its packed payload range"
                    .to_string(),
            );
        };
        frame_batch.push(upload);
        self.selected_texture = selected.map(|(_, texture)| texture);
        Ok(())
    }

    pub(crate) fn bind_group_entries(&self) -> [wgpu::BindGroupEntry<'_>; 3] {
        let _retain_fallback = &self.fallback_texture;
        let view = self
            .selected_texture
            .as_ref()
            .map(IrradianceVolumeTextureBinding::view)
            .unwrap_or(&self.fallback_view);
        [
            wgpu::BindGroupEntry {
                binding: IRRADIANCE_VOLUME_TEXTURE_BINDING,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: IRRADIANCE_VOLUME_SAMPLER_BINDING,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
            wgpu::BindGroupEntry {
                binding: IRRADIANCE_VOLUME_PARAMS_BINDING,
                resource: self.params_buffer.as_entire_binding(),
            },
        ]
    }
}

pub(crate) fn irradiance_volume_bind_group_layout_entries() -> [wgpu::BindGroupLayoutEntry; 3] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: IRRADIANCE_VOLUME_TEXTURE_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D3,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: IRRADIANCE_VOLUME_SAMPLER_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: IRRADIANCE_VOLUME_PARAMS_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

#[cfg(test)]
#[path = "tests/resources.rs"]
mod tests;
