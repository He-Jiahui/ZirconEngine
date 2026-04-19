use super::super::super::super::post_process_params::PostProcessParams;

pub(super) fn post_process_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-post-process-params"),
        size: std::mem::size_of::<PostProcessParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
