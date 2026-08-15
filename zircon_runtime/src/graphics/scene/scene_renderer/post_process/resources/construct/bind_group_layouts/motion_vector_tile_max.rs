use crate::graphics::shader::{
    create_fullscreen_pass_input_bind_group_layout,
    create_fullscreen_pass_parameter_bind_group_layout, motion_vector_tile_max_pass_plan,
    ShaderWgpuResourceDescriptor, MOTION_VECTOR_SOURCE_RESOURCE,
};

pub(crate) fn motion_vector_tile_max(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    create_fullscreen_pass_input_bind_group_layout(
        device,
        motion_vector_tile_max_pass_plan(),
        &[ShaderWgpuResourceDescriptor::texture(
            MOTION_VECTOR_SOURCE_RESOURCE,
            wgpu::TextureSampleType::Float { filterable: false },
            wgpu::TextureViewDimension::D2,
            false,
        )],
    )
    .expect("builtin motion-vector fullscreen layout contract must be valid")
}

pub(crate) fn motion_vector_tile_max_parameters(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    create_fullscreen_pass_parameter_bind_group_layout(device, motion_vector_tile_max_pass_plan())
        .expect("motion-vector tile-max fullscreen plan must declare parameters")
}
