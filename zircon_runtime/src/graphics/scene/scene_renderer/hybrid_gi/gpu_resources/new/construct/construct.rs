use super::super::super::hybrid_gi_gpu_resources::HybridGiGpuResources;
use super::super::bind_group_layout::bind_group_layout;
use super::super::params_buffer::params_buffer;
use super::super::pipeline::pipeline;

impl HybridGiGpuResources {
    pub(in crate::graphics::scene::scene_renderer) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = bind_group_layout(device);
        let pipeline = pipeline(device, &bind_group_layout);
        let params_buffer = params_buffer(device);

        Self {
            bind_group_layout,
            pipeline,
            params_buffer,
        }
    }
}
