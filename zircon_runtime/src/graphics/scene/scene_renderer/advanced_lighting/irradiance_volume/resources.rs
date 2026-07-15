use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::framework::render::{IrradianceVolumeData, RenderImageDimension};
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

pub(crate) struct IrradianceVolumeResources {
    fallback_texture: wgpu::Texture,
    fallback_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    selected_texture: Option<IrradianceVolumeTextureBinding>,
}

impl IrradianceVolumeResources {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let fallback_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-irradiance-volume-fallback"),
            size: wgpu::Extent3d {
                width: 1,
                height: 2,
                depth_or_array_layers: 3,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            fallback_texture.as_image_copy(),
            &[0; 24],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(2),
            },
            wgpu::Extent3d {
                width: 1,
                height: 2,
                depth_or_array_layers: 3,
            },
        );
        let fallback_view = fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-irradiance-volume-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });
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

    pub(crate) fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        selected: Option<(IrradianceVolumeData, IrradianceVolumeTextureBinding)>,
    ) {
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
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
        self.selected_texture = selected.map(|(_, texture)| texture);
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
mod tests {
    use super::*;
    use crate::core::math::{Mat4, Quat, Vec3};
    use crate::core::resource::ResourceId;

    #[test]
    fn render_irrvol_gpu_normal_matrix_handles_rotation_and_nonuniform_scale() {
        let world_from_volume = Mat4::from_scale_rotation_translation(
            Vec3::new(4.0, 2.0, 0.5),
            Quat::from_rotation_y(0.7) * Quat::from_rotation_x(-0.35),
            Vec3::new(3.0, -2.0, 5.0),
        );
        let volume = IrradianceVolumeData {
            volume_id: 1,
            transform: world_from_volume.inverse(),
            voxels: ResourceId::from_stable_label("runtime://irradiance-volume/normal-matrix"),
            intensity: 1.0,
            affects_lightmapped_meshes: false,
            priority: 0,
            layer_mask: Default::default(),
        };
        let params = GpuIrradianceVolumeParams::from_volume(&volume);
        let normal_ws = Vec3::new(0.3, 0.8, -0.2).normalize();
        let actual = Vec3::new(
            params.normal_to_volume[0][0] * normal_ws.x
                + params.normal_to_volume[1][0] * normal_ws.y
                + params.normal_to_volume[2][0] * normal_ws.z,
            params.normal_to_volume[0][1] * normal_ws.x
                + params.normal_to_volume[1][1] * normal_ws.y
                + params.normal_to_volume[2][1] * normal_ws.z,
            params.normal_to_volume[0][2] * normal_ws.x
                + params.normal_to_volume[1][2] * normal_ws.y
                + params.normal_to_volume[2][2] * normal_ws.z,
        )
        .normalize();
        let expected = volume
            .transform
            .inverse()
            .transpose()
            .transform_vector3(normal_ws)
            .normalize();

        assert!((actual - expected).length() <= 1.0e-5);
        assert_eq!(std::mem::size_of::<GpuIrradianceVolumeParams>(), 144);
    }
}
