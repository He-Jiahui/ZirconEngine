use super::super::super::hybrid_gi_completion_params::HybridGiCompletionParams;

pub(in crate::graphics::scene::scene_renderer::hybrid_gi::gpu_resources::new) fn params_buffer(
    device: &wgpu::Device,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-hybrid-gi-completion-params"),
        size: std::mem::size_of::<HybridGiCompletionParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
