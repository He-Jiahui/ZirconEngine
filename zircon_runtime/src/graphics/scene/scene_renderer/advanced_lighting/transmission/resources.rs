use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::graphics::backend::SystemTextureGenerationLease;

pub(crate) const TRANSMISSION_SCENE_COLOR_BINDING: u32 = 31;
pub(crate) const TRANSMISSION_SCENE_COLOR_SAMPLER_BINDING: u32 = 32;
pub(crate) const TRANSMISSION_SCENE_COLOR_PARAMS_BINDING: u32 = 38;
#[cfg(test)]
const TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL: [u8; 4] = [0, 0, 0, 0];

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuTransmissionSceneColorParams {
    available: u32,
    _padding: [u32; 3],
}

impl GpuTransmissionSceneColorParams {
    const fn new(available: bool) -> Self {
        Self {
            available: available as u32,
            _padding: [0; 3],
        }
    }
}

pub(crate) struct TransmissionSceneColorFallbackResources {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    available_params: wgpu::Buffer,
    unavailable_params: wgpu::Buffer,
}

impl TransmissionSceneColorFallbackResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        system_textures: &SystemTextureGenerationLease,
    ) -> Self {
        let texture = system_textures.black_rgba8_texture().clone();
        let view = system_textures.black_rgba8_view().clone();
        let sampler = system_textures.linear_clamp_sampler().clone();
        let available_params = create_params_buffer(device, true);
        let unavailable_params = create_params_buffer(device, false);
        Self {
            _texture: texture,
            view,
            sampler,
            available_params,
            unavailable_params,
        }
    }

    pub(crate) fn bind_group_entries<'a>(
        &'a self,
        scene_color_view: Option<&'a wgpu::TextureView>,
    ) -> [wgpu::BindGroupEntry<'a>; 3] {
        let available = scene_color_view.is_some();
        [
            wgpu::BindGroupEntry {
                binding: TRANSMISSION_SCENE_COLOR_BINDING,
                resource: wgpu::BindingResource::TextureView(
                    scene_color_view.unwrap_or(&self.view),
                ),
            },
            wgpu::BindGroupEntry {
                binding: TRANSMISSION_SCENE_COLOR_SAMPLER_BINDING,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
            wgpu::BindGroupEntry {
                binding: TRANSMISSION_SCENE_COLOR_PARAMS_BINDING,
                resource: if available {
                    self.available_params.as_entire_binding()
                } else {
                    self.unavailable_params.as_entire_binding()
                },
            },
        ]
    }
}

fn create_params_buffer(device: &wgpu::Device, available: bool) -> wgpu::Buffer {
    let params = GpuTransmissionSceneColorParams::new(available);
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(if available {
            "zircon-transmission-scene-color-available-params"
        } else {
            "zircon-transmission-scene-color-unavailable-params"
        }),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

pub(crate) fn transmission_scene_color_bind_group_layout_entries() -> [wgpu::BindGroupLayoutEntry; 3]
{
    [
        wgpu::BindGroupLayoutEntry {
            binding: TRANSMISSION_SCENE_COLOR_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: TRANSMISSION_SCENE_COLOR_SAMPLER_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: TRANSMISSION_SCENE_COLOR_PARAMS_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                    GpuTransmissionSceneColorParams,
                >() as u64),
            },
            count: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        GpuTransmissionSceneColorParams, TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL,
        transmission_scene_color_bind_group_layout_entries,
    };

    #[test]
    fn render_transmission_zero_step_fallback_marks_scene_copy_unavailable() {
        assert_eq!(TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL, [0, 0, 0, 0]);
        assert_eq!(GpuTransmissionSceneColorParams::new(false).available, 0);
        assert_eq!(GpuTransmissionSceneColorParams::new(true).available, 1);
        assert_eq!(std::mem::size_of::<GpuTransmissionSceneColorParams>(), 16);
        let layout_entries = transmission_scene_color_bind_group_layout_entries();
        assert_eq!(
            layout_entries.each_ref().map(|entry| entry.binding),
            [31, 32, 38]
        );
        let wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            min_binding_size,
            ..
        } = &layout_entries[2].ty
        else {
            panic!("transmission scene-color params must remain a uniform buffer");
        };
        assert_eq!(min_binding_size.as_ref().map(|size| size.get()), Some(16));
    }
}
