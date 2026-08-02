use crate::graphics::shader::{
    create_compute_shader_bind_group_layout, hzb_build_dispatch_plan, hzb_build_msaa_dispatch_plan,
    ShaderWgpuResourceDescriptor, HZB_SCENE_DEPTH_RESOURCE, HZB_SOURCE_RESOURCE,
    HZB_TARGET_RESOURCE,
};

pub(crate) fn hzb(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    hzb_for_depth_sampling(device, false, "zircon-hzb-bind-group-layout")
}

pub(crate) fn hzb_msaa(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    hzb_for_depth_sampling(device, true, "zircon-hzb-msaa-bind-group-layout")
}

fn hzb_for_depth_sampling(
    device: &wgpu::Device,
    multisampled: bool,
    label: &'static str,
) -> wgpu::BindGroupLayout {
    let plan = if multisampled {
        hzb_build_msaa_dispatch_plan()
    } else {
        hzb_build_dispatch_plan()
    };
    create_compute_shader_bind_group_layout(
        device,
        plan,
        &[
            ShaderWgpuResourceDescriptor::texture(
                HZB_SCENE_DEPTH_RESOURCE,
                wgpu::TextureSampleType::Depth,
                wgpu::TextureViewDimension::D2,
                multisampled,
            ),
            ShaderWgpuResourceDescriptor::texture(
                HZB_SOURCE_RESOURCE,
                wgpu::TextureSampleType::Float { filterable: false },
                wgpu::TextureViewDimension::D2,
                false,
            ),
            ShaderWgpuResourceDescriptor::storage_texture(
                HZB_TARGET_RESOURCE,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureViewDimension::D2,
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("{label}: {error}"))
}
