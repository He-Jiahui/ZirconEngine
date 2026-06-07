use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::pipeline_bundle::PipelineBundle;
use super::bloom_pipeline::bloom_pipeline;
use super::cluster_pipeline::cluster_pipeline;
use super::depth_of_field_prepare_pipeline::depth_of_field_prepare_pipeline;
use super::motion_vector_camera_pipeline::motion_vector_camera_pipeline;
use super::motion_vector_neighbor_max_pipeline::motion_vector_neighbor_max_pipeline;
use super::motion_vector_tile_max_pipeline::motion_vector_tile_max_pipeline;
use super::post_process_pipeline::post_process_pipeline;
use super::screen_space_reflection_depth_pyramid_coarse_pipeline::screen_space_reflection_depth_pyramid_coarse_pipeline;
use super::screen_space_reflection_depth_pyramid_pipeline::screen_space_reflection_depth_pyramid_pipeline;
use super::screen_space_reflection_reflection_pyramid_coarse_pipeline::screen_space_reflection_reflection_pyramid_coarse_pipeline;
use super::screen_space_reflection_reflection_pyramid_pipeline::screen_space_reflection_reflection_pyramid_pipeline;
use super::screen_space_reflection_resolve_pipeline::screen_space_reflection_resolve_pipeline;
use super::screen_space_reflection_specular_occlusion_pipeline::screen_space_reflection_specular_occlusion_pipeline;

pub(crate) fn create_pipeline_bundle(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    bloom_bind_group_layout: &wgpu::BindGroupLayout,
    cluster_bind_group_layout: &wgpu::BindGroupLayout,
    depth_of_field_prepare_bind_group_layout: &wgpu::BindGroupLayout,
    motion_vector_camera_bind_group_layout: &wgpu::BindGroupLayout,
    motion_vector_tile_max_bind_group_layout: &wgpu::BindGroupLayout,
    motion_vector_neighbor_max_bind_group_layout: &wgpu::BindGroupLayout,
    post_process_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> PipelineBundle {
    PipelineBundle {
        bloom_pipeline: bloom_pipeline(device, target_format, bloom_bind_group_layout),
        cluster_pipeline: cluster_pipeline(device, cluster_bind_group_layout),
        depth_of_field_prepare_pipeline: depth_of_field_prepare_pipeline(
            device,
            target_format,
            depth_of_field_prepare_bind_group_layout,
            depth_sampling_mode,
        ),
        motion_vector_camera_pipeline: motion_vector_camera_pipeline(
            device,
            motion_vector_camera_bind_group_layout,
            depth_sampling_mode,
        ),
        motion_vector_tile_max_pipeline: motion_vector_tile_max_pipeline(
            device,
            motion_vector_tile_max_bind_group_layout,
        ),
        motion_vector_neighbor_max_pipeline: motion_vector_neighbor_max_pipeline(
            device,
            motion_vector_neighbor_max_bind_group_layout,
        ),
        screen_space_reflection_depth_pyramid_pipeline:
            screen_space_reflection_depth_pyramid_pipeline(
                device,
                post_process_bind_group_layout,
                depth_sampling_mode,
            ),
        screen_space_reflection_depth_pyramid_coarse_pipeline:
            screen_space_reflection_depth_pyramid_coarse_pipeline(
                device,
                post_process_bind_group_layout,
                depth_sampling_mode,
            ),
        screen_space_reflection_reflection_pyramid_pipeline:
            screen_space_reflection_reflection_pyramid_pipeline(
                device,
                post_process_bind_group_layout,
                depth_sampling_mode,
            ),
        screen_space_reflection_reflection_pyramid_coarse_pipeline:
            screen_space_reflection_reflection_pyramid_coarse_pipeline(
                device,
                post_process_bind_group_layout,
                depth_sampling_mode,
            ),
        screen_space_reflection_resolve_pipeline: screen_space_reflection_resolve_pipeline(
            device,
            target_format,
            post_process_bind_group_layout,
            depth_sampling_mode,
        ),
        screen_space_reflection_specular_occlusion_pipeline:
            screen_space_reflection_specular_occlusion_pipeline(
                device,
                post_process_bind_group_layout,
                depth_sampling_mode,
            ),
        post_process_pipeline: post_process_pipeline(
            device,
            target_format,
            post_process_bind_group_layout,
            depth_sampling_mode,
        ),
    }
}
