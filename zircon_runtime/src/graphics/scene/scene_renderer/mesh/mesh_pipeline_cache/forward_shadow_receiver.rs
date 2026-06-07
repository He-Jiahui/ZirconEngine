use bytemuck::{bytes_of, Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
use crate::graphics::scene::scene_renderer::shadow::{
    SHADOW_RECEIVER_DEPTH_BIAS, SHADOW_RECEIVER_MIN_VISIBILITY,
};

use super::MeshPipelineCache;

const FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES: wgpu::ShaderStages =
    wgpu::ShaderStages::FRAGMENT;
const SHADOW_RECEIVER_DISABLED: f32 = 0.0;
const SHADOW_RECEIVER_ENABLED: f32 = 1.0;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::mesh) struct ForwardShadowReceiverUniform {
    light_view_proj: [[f32; 4]; 4],
    params: [f32; 4],
}

impl ForwardShadowReceiverUniform {
    pub(in crate::graphics::scene::scene_renderer::mesh) fn disabled() -> Self {
        Self {
            light_view_proj: crate::core::math::RenderMat4::IDENTITY.to_cols_array_2d(),
            params: [SHADOW_RECEIVER_DISABLED, 0.0, 1.0, 0.0],
        }
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn from_shadow_scene_uniform(
        shadow_scene_uniform: Option<SceneUniform>,
    ) -> Self {
        let Some(shadow_scene_uniform) = shadow_scene_uniform else {
            return Self::disabled();
        };

        Self {
            light_view_proj: shadow_scene_uniform.view_proj,
            params: [
                SHADOW_RECEIVER_ENABLED,
                SHADOW_RECEIVER_DEPTH_BIAS,
                SHADOW_RECEIVER_MIN_VISIBILITY,
                0.0,
            ],
        }
    }
}

impl MeshPipelineCache {
    pub(in crate::graphics::scene::scene_renderer) fn update_forward_shadow_receiver(
        &self,
        queue: &wgpu::Queue,
        shadow_scene_uniform: Option<SceneUniform>,
    ) {
        let uniform = ForwardShadowReceiverUniform::from_shadow_scene_uniform(shadow_scene_uniform);
        queue.write_buffer(
            &self.forward_shadow_receiver_uniform_buffer,
            0,
            bytes_of(&uniform),
        );
    }

    pub(in crate::graphics::scene::scene_renderer) fn create_forward_shadow_receiver_bind_group(
        &self,
        device: &wgpu::Device,
        shadow_map_view: Option<&wgpu::TextureView>,
    ) -> wgpu::BindGroup {
        let receiver_uniform_buffer = if shadow_map_view.is_some() {
            &self.forward_shadow_receiver_uniform_buffer
        } else {
            &self.forward_shadow_receiver_disabled_uniform_buffer
        };
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-forward-shadow-receiver-bind-group"),
            layout: &self.forward_shadow_receiver_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        shadow_map_view.unwrap_or(&self.fallback_shadow_map_view),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: receiver_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.forward_shadow_compare_sampler),
                },
            ],
        })
    }
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_shadow_receiver_layout(
    device: &wgpu::Device,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-forward-shadow-receiver-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: FORWARD_SHADOW_RECEIVER_BINDING_SHADER_STAGES,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_shadow_receiver_uniform_buffer(
    device: &wgpu::Device,
    label: &'static str,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytes_of(&ForwardShadowReceiverUniform::disabled()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_forward_shadow_compare_sampler(
    device: &wgpu::Device,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-forward-shadow-compare-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    })
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_fallback_shadow_map_view(
    device: &wgpu::Device,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-forward-shadow-fallback-texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: super::super::super::core::DEPTH_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

#[cfg(test)]
mod tests {
    use super::ForwardShadowReceiverUniform;
    use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
    use crate::graphics::scene::scene_renderer::shadow::{
        SHADOW_RECEIVER_DEPTH_BIAS, SHADOW_RECEIVER_MIN_VISIBILITY,
    };

    #[test]
    fn disabled_receiver_keeps_forward_shadow_sampling_neutral() {
        let uniform = ForwardShadowReceiverUniform::from_shadow_scene_uniform(None);

        assert_eq!(uniform.params, [0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn enabled_receiver_forwards_light_view_projection_and_bias_policy() {
        let light_view_proj = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 3.0, 0.0],
            [4.0, 5.0, 6.0, 1.0],
        ];
        let uniform = ForwardShadowReceiverUniform::from_shadow_scene_uniform(Some(SceneUniform {
            view_proj: light_view_proj,
            inverse_view_proj: crate::core::math::RenderMat4::IDENTITY.to_cols_array_2d(),
            light_dir: [0.0, -1.0, 0.0, 0.0],
            light_color: [1.0, 1.0, 1.0, 1.0],
            ambient_color: [0.0, 0.0, 0.0, 1.0],
            previous_view_proj: light_view_proj,
            motion_params: [0.0, 0.0, 0.0, 0.0],
        }));

        assert_eq!(uniform.light_view_proj, light_view_proj);
        assert_eq!(
            uniform.params,
            [
                1.0,
                SHADOW_RECEIVER_DEPTH_BIAS,
                SHADOW_RECEIVER_MIN_VISIBILITY,
                0.0,
            ]
        );
        assert!(uniform.params[1] > 0.0 && uniform.params[1] < 0.02);
        assert!(uniform.params[2] > 0.0 && uniform.params[2] <= 1.0);
    }
}
