pub(crate) const SHADOW_ATLAS_BINDING: u32 = 8;
pub(crate) const SHADOW_ATLAS_SAMPLER_BINDING: u32 = 9;
pub(crate) const SHADOW_ATLAS_SLOT_BUFFER_BINDING: u32 = 10;
pub(crate) const SHADOW_GLOBALS_BINDING: u32 = 11;

/// 向 deferred 与 forward 共用的场景 group 添加阴影 atlas ABI；调用方必须使用相同的 binding 编号。
pub(crate) fn shadow_atlas_bind_group_layout_entries(
    visibility: wgpu::ShaderStages,
) -> [wgpu::BindGroupLayoutEntry; 4] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: SHADOW_ATLAS_BINDING,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Depth,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: SHADOW_ATLAS_SAMPLER_BINDING,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: SHADOW_ATLAS_SLOT_BUFFER_BINDING,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: SHADOW_GLOBALS_BINDING,
            visibility,
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
#[path = "tests/bindings.rs"]
mod tests;
