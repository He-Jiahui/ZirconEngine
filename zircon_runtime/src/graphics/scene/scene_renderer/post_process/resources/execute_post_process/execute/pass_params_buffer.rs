use super::super::super::super::post_process_params::PostProcessParams;

pub(in crate::graphics::scene::scene_renderer::post_process::resources) fn create_post_process_params_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    params: &PostProcessParams,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: std::mem::size_of::<PostProcessParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buffer, 0, bytemuck::bytes_of(params));
    buffer
}
