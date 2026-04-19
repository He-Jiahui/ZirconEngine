use super::super::pipeline_bundle::PipelineBundle;
use super::bloom_pipeline::bloom_pipeline;
use super::cluster_pipeline::cluster_pipeline;
use super::post_process_pipeline::post_process_pipeline;
use super::ssao_pipeline::ssao_pipeline;

pub(crate) fn create_pipeline_bundle(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    bloom_bind_group_layout: &wgpu::BindGroupLayout,
    ssao_bind_group_layout: &wgpu::BindGroupLayout,
    cluster_bind_group_layout: &wgpu::BindGroupLayout,
    post_process_bind_group_layout: &wgpu::BindGroupLayout,
) -> PipelineBundle {
    PipelineBundle {
        bloom_pipeline: bloom_pipeline(device, target_format, bloom_bind_group_layout),
        ssao_pipeline: ssao_pipeline(device, ssao_bind_group_layout),
        cluster_pipeline: cluster_pipeline(device, cluster_bind_group_layout),
        post_process_pipeline: post_process_pipeline(
            device,
            target_format,
            post_process_bind_group_layout,
        ),
    }
}
