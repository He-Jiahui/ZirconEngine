use std::num::NonZeroU64;

use super::super::primitives::SceneEnvironmentSh9;

/// 场景 group0 的公共环境布局，供 mesh、天空和 deferred 使用同一资源入口。
/// 源 cube、镜面 PMREM、漫反射 cube 与 SH9 分槽保留，使静态/实时环境可换资源而不换 ABI。
pub(in crate::graphics::scene::scene_renderer) fn scene_bind_group_layout_entries(
) -> [wgpu::BindGroupLayoutEntry; 7] {
    [
        scene_uniform_layout_entry(),
        environment_cube_texture_entry(1),
        environment_sampler_entry(2),
        environment_brdf_lut_entry(3),
        environment_cube_texture_entry(4),
        environment_cube_texture_entry(5),
        environment_sh9_uniform_entry(6),
    ]
}

fn environment_sh9_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: NonZeroU64::new(SceneEnvironmentSh9::byte_len()),
        },
        count: None,
    }
}

fn scene_uniform_layout_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX
            | wgpu::ShaderStages::FRAGMENT
            | wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn environment_cube_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
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

fn environment_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn environment_brdf_lut_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    }
}

#[cfg(test)]
#[path = "tests/scene_bind_group_layout.rs"]
mod tests;
