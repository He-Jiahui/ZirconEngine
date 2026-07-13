use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};

use super::resolved_volumetric_fog_settings;

pub(crate) const VOLUMETRIC_APPLY_PARAMS_BINDING: u32 = 25;
pub(crate) const VOLUMETRIC_INTEGRATED_BINDING: u32 = 26;
pub(crate) const VOLUMETRIC_SAMPLER_BINDING: u32 = 27;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuVolumetricApplyParams {
    depth: [f32; 4],
    viewport: [f32; 4],
}

impl GpuVolumetricApplyParams {
    fn from_frame(
        frame: &ViewportRenderFrame,
        render_region: ViewportRenderRegion,
        integrated_volume_available: bool,
    ) -> Self {
        let settings = resolved_volumetric_fog_settings(&frame.extract).ok();
        let camera = frame.extract.view.selected_effective_camera();
        let enabled = integrated_volume_available && settings.is_some();
        let depth_distribution_exp = settings
            .map(|settings| settings.depth_distribution_exp)
            .unwrap_or(1.0)
            .max(0.01);
        let origin = render_region.physical_position();
        let size = render_region
            .physical_size()
            .max(crate::core::math::UVec2::ONE);
        Self {
            depth: [
                camera.z_near.max(0.0001),
                camera.z_far.max(camera.z_near + 0.0001),
                depth_distribution_exp,
                if enabled { 1.0 } else { 0.0 },
            ],
            viewport: [
                origin.x as f32,
                origin.y as f32,
                1.0 / size.x as f32,
                1.0 / size.y as f32,
            ],
        }
    }
}

pub(crate) struct VolumetricApplyFallbackResources {
    integrated_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl VolumetricApplyFallbackResources {
    pub(crate) fn new(device: &wgpu::Device, label_prefix: &str) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{label_prefix}-volumetric-fallback-texture")),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let integrated_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{label_prefix}-volumetric-sampler")),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        Self {
            integrated_view,
            sampler,
        }
    }

    pub(crate) fn create_params_buffer(
        &self,
        device: &wgpu::Device,
        frame: &ViewportRenderFrame,
        render_region: ViewportRenderRegion,
        integrated_volume_available: bool,
        label: &str,
    ) -> wgpu::Buffer {
        let params =
            GpuVolumetricApplyParams::from_frame(frame, render_region, integrated_volume_available);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }

    pub(crate) fn create_disabled_params_buffer(
        &self,
        device: &wgpu::Device,
        label: &str,
    ) -> wgpu::Buffer {
        let params = GpuVolumetricApplyParams {
            depth: [0.1, 1.0, 1.0, 0.0],
            viewport: [0.0, 0.0, 1.0, 1.0],
        };
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }

    pub(crate) fn bind_group_entries<'a>(
        &'a self,
        params_buffer: &'a wgpu::Buffer,
        integrated_view: Option<&'a wgpu::TextureView>,
    ) -> [wgpu::BindGroupEntry<'a>; 3] {
        [
            wgpu::BindGroupEntry {
                binding: VOLUMETRIC_APPLY_PARAMS_BINDING,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: VOLUMETRIC_INTEGRATED_BINDING,
                resource: wgpu::BindingResource::TextureView(
                    integrated_view.unwrap_or(&self.integrated_view),
                ),
            },
            wgpu::BindGroupEntry {
                binding: VOLUMETRIC_SAMPLER_BINDING,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
        ]
    }
}

pub(crate) fn volumetric_apply_bind_group_layout_entries(
    visibility: wgpu::ShaderStages,
) -> [wgpu::BindGroupLayoutEntry; 3] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: VOLUMETRIC_APPLY_PARAMS_BINDING,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: VOLUMETRIC_INTEGRATED_BINDING,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D3,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: VOLUMETRIC_SAMPLER_BINDING,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volumetric_apply_layout_reserves_plan18_bindings() {
        let entries = volumetric_apply_bind_group_layout_entries(wgpu::ShaderStages::FRAGMENT);
        assert_eq!(
            entries.map(|entry| entry.binding),
            [
                VOLUMETRIC_APPLY_PARAMS_BINDING,
                VOLUMETRIC_INTEGRATED_BINDING,
                VOLUMETRIC_SAMPLER_BINDING,
            ]
        );
    }
}
