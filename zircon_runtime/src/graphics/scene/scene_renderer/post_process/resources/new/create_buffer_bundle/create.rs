use super::super::buffer_bundle::BufferBundle;
use super::bloom_params_buffer::bloom_params_buffer;
use super::cluster_params_buffer::cluster_params_buffer;
use super::hybrid_gi_probe_buffer::hybrid_gi_probe_buffer;
use super::hybrid_gi_trace_region_buffer::hybrid_gi_trace_region_buffer;
use super::light_buffer::light_buffer;
use super::post_process_params_buffer::post_process_params_buffer;
use super::reflection_probe_buffer::reflection_probe_buffer;
use super::ssao_params_buffer::ssao_params_buffer;

pub(in super::super) fn create_buffer_bundle(device: &wgpu::Device) -> BufferBundle {
    BufferBundle {
        bloom_params_buffer: bloom_params_buffer(device),
        ssao_params_buffer: ssao_params_buffer(device),
        cluster_params_buffer: cluster_params_buffer(device),
        post_process_params_buffer: post_process_params_buffer(device),
        light_buffer: light_buffer(device),
        hybrid_gi_probe_buffer: hybrid_gi_probe_buffer(device),
        hybrid_gi_trace_region_buffer: hybrid_gi_trace_region_buffer(device),
        reflection_probe_buffer: reflection_probe_buffer(device),
    }
}
