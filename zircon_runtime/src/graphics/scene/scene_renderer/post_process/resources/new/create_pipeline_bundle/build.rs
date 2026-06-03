use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::pipeline_bundle::PipelineBundle;
use super::bloom_pipeline::bloom_pipeline;
use super::cluster_pipeline::cluster_pipeline;
use super::post_process_pipeline::post_process_pipeline;

pub(crate) fn create_pipeline_bundle(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    bloom_bind_group_layout: &wgpu::BindGroupLayout,
    cluster_bind_group_layout: &wgpu::BindGroupLayout,
    post_process_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> PipelineBundle {
    PipelineBundle {
        bloom_pipeline: bloom_pipeline(device, target_format, bloom_bind_group_layout),
        cluster_pipeline: cluster_pipeline(device, cluster_bind_group_layout),
        post_process_pipeline: post_process_pipeline(
            device,
            target_format,
            post_process_bind_group_layout,
            depth_sampling_mode,
        ),
    }
}
