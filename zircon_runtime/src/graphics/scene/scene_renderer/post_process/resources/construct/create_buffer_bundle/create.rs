use super::super::buffer_bundle::BufferBundle;
use super::bloom_params_buffer::bloom_params_buffer;
use super::cluster_params_buffer::cluster_params_buffer;
use super::color_lut_bake_params_buffer::color_lut_bake_params_buffer;
use super::default_exposure_buffer::default_exposure_buffer;
use super::default_exposure_histogram_buffer::default_exposure_histogram_buffer;
use super::depth_of_field_prepare_params_buffer::depth_of_field_prepare_params_buffer;
use super::exposure_params_buffer::exposure_params_buffer;
use super::hybrid_gi_probe_buffer::hybrid_gi_probe_buffer;
use super::hybrid_gi_trace_region_buffer::hybrid_gi_trace_region_buffer;
use super::hzb_params_buffer::hzb_params_buffer;
use super::light_buffer::light_buffer;
use super::reflection_probe_buffer::reflection_probe_buffer;
use super::ssao_params_buffer::ssao_params_buffer;
use super::taa_resolve_params_buffer::taa_resolve_params_buffer;
use super::velocity_camera_params_buffer::velocity_camera_params_buffer;

pub(in super::super) fn create_buffer_bundle(device: &wgpu::Device) -> BufferBundle {
    BufferBundle {
        bloom_params_buffer: bloom_params_buffer(device),
        ssao_params_buffer: ssao_params_buffer(device),
        cluster_params_buffer: cluster_params_buffer(device),
        depth_of_field_prepare_params_buffer: depth_of_field_prepare_params_buffer(device),
        hzb_params_buffer: hzb_params_buffer(device),
        exposure_params_buffer: exposure_params_buffer(device),
        color_lut_bake_params_buffer: color_lut_bake_params_buffer(device),
        default_exposure_buffer: default_exposure_buffer(device),
        default_exposure_histogram_buffer: default_exposure_histogram_buffer(device),
        taa_resolve_params_buffer: taa_resolve_params_buffer(device),
        velocity_camera_params_buffer: velocity_camera_params_buffer(device),
        light_buffer: light_buffer(device),
        hybrid_gi_probe_buffer: hybrid_gi_probe_buffer(device),
        hybrid_gi_trace_region_buffer: hybrid_gi_trace_region_buffer(device),
        reflection_probe_buffer: reflection_probe_buffer(device),
    }
}
