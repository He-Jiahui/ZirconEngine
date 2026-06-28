use super::super::super::super::depth_of_field_prepare_params::DepthOfFieldPrepareParams;

pub(super) fn depth_of_field_prepare_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-depth-of-field-prepare-params"),
        size: std::mem::size_of::<DepthOfFieldPrepareParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
