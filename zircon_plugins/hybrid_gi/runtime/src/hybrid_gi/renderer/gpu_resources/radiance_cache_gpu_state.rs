use super::{
    create_radiance_cache_atlas_buffer, create_radiance_cache_mark_buffer,
    create_radiance_cache_params_buffers, create_radiance_cache_storage_buffer,
};

pub(in crate::hybrid_gi::renderer) struct RadianceCacheGpuState {
    pub(super) params_buffers: [wgpu::Buffer; 6],
    pub(super) storage_buffer: wgpu::Buffer,
    pub(super) mark_buffer: wgpu::Buffer,
    pub(super) trace_buffer: wgpu::Buffer,
    pub(super) filtered_buffer: wgpu::Buffer,
    pub(super) final_atlas_buffer: wgpu::Buffer,
}

impl RadianceCacheGpuState {
    pub(in crate::hybrid_gi::renderer) fn new(device: &wgpu::Device) -> Self {
        Self {
            params_buffers: create_radiance_cache_params_buffers(device),
            storage_buffer: create_radiance_cache_storage_buffer(
                device,
                "zircon-hybrid-gi-radiance-cache-storage",
            ),
            mark_buffer: create_radiance_cache_mark_buffer(device),
            trace_buffer: create_radiance_cache_atlas_buffer(
                device,
                "zircon-hybrid-gi-radiance-cache-trace-atlas",
            ),
            filtered_buffer: create_radiance_cache_atlas_buffer(
                device,
                "zircon-hybrid-gi-radiance-cache-filtered-atlas",
            ),
            final_atlas_buffer: create_radiance_cache_atlas_buffer(
                device,
                "zircon-hybrid-gi-radiance-cache-final-atlas",
            ),
        }
    }
}
