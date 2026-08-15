use super::super::super::hybrid_gi_gpu_resources::HybridGiGpuResources;
use super::super::super::{
    create_probe_trace_tile_dispatch_bind_group_layout, create_probe_trace_tile_dispatch_pipeline,
    create_probe_trace_tile_generation_bind_group_layout,
    create_probe_trace_tile_generation_pipeline, create_radiance_cache_bind_group_layout,
    create_radiance_cache_consume_pipeline, create_radiance_cache_update_pipeline,
};
use super::super::bind_group_layout::bind_group_layout;
use super::super::params_buffer::params_buffer;
use super::super::pipeline::pipeline;

impl HybridGiGpuResources {
    pub(in crate::hybrid_gi::renderer) fn new(device: &wgpu::Device) -> Self {
        let global_sdf = super::super::super::global_sdf::GlobalSdfGpuResources::new(device);
        let bind_group_layout = bind_group_layout(device);
        let pipeline = pipeline(device, &bind_group_layout);
        let params_buffer = params_buffer(device);
        let probe_trace_tile_bind_group_layout =
            create_probe_trace_tile_dispatch_bind_group_layout(device);
        let probe_trace_tile_pipeline =
            create_probe_trace_tile_dispatch_pipeline(device, &probe_trace_tile_bind_group_layout);
        let probe_trace_tile_generation_bind_group_layout =
            create_probe_trace_tile_generation_bind_group_layout(device);
        let probe_trace_tile_generation_pipeline = create_probe_trace_tile_generation_pipeline(
            device,
            &probe_trace_tile_generation_bind_group_layout,
        );
        let radiance_cache_bind_group_layout = create_radiance_cache_bind_group_layout(device);
        let radiance_cache_update_pipeline =
            create_radiance_cache_update_pipeline(device, &radiance_cache_bind_group_layout);
        let radiance_cache_consume_pipeline =
            create_radiance_cache_consume_pipeline(device, &radiance_cache_bind_group_layout);
        Self {
            global_sdf,
            bind_group_layout,
            pipeline,
            params_buffer,
            probe_trace_tile_bind_group_layout,
            probe_trace_tile_pipeline,
            probe_trace_tile_generation_bind_group_layout,
            probe_trace_tile_generation_pipeline,
            radiance_cache_bind_group_layout,
            radiance_cache_update_pipeline,
            radiance_cache_consume_pipeline,
        }
    }
}
