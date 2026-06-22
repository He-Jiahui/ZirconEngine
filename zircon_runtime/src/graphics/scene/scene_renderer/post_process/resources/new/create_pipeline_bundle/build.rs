use crate::graphics::scene::scene_renderer::post_process::POST_PROCESS_INTERMEDIATE_HDR_FORMAT;

use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::pipeline_bundle::PipelineBundle;
use super::bloom_pipeline::bloom_pipeline;
use super::blur_pipeline::blur_pipeline;
use super::cluster_pipeline::cluster_pipeline;
use super::color_lut_bake_pipeline::color_lut_bake_pipeline;
use super::depth_of_field_pipeline::depth_of_field_pipeline;
use super::depth_of_field_prepare_pipeline::depth_of_field_prepare_pipeline;
use super::exposure_histogram_pipeline::exposure_histogram_pipeline;
use super::exposure_resolve_pipeline::exposure_resolve_pipeline;
use super::fxaa_pipeline::fxaa_pipeline;
use super::hzb_pipeline::hzb_pipeline;
use super::motion_blur_pipeline::motion_blur_pipeline;
use super::motion_vector_neighbor_max_pipeline::motion_vector_neighbor_max_pipeline;
use super::motion_vector_tile_max_pipeline::motion_vector_tile_max_pipeline;
use super::output_transfer_pipeline::output_transfer_pipeline;
use super::post_process_pipeline::post_process_pipeline;
use super::scene_composite_pipeline::scene_composite_pipeline;
use super::screen_space_reflection_reflection_pyramid_coarse_pipeline::screen_space_reflection_reflection_pyramid_coarse_pipeline;
use super::screen_space_reflection_reflection_pyramid_pipeline::screen_space_reflection_reflection_pyramid_pipeline;
use super::screen_space_reflection_resolve_pipeline::screen_space_reflection_resolve_pipeline;
use super::screen_space_reflection_specular_occlusion_pipeline::screen_space_reflection_specular_occlusion_pipeline;
use super::smaa_pipeline::smaa_pipeline_bundle;
use super::taa_resolve_pipeline::taa_resolve_pipeline;
use super::upscale_pipeline::upscale_pipeline;
use super::velocity_camera_pipeline::velocity_camera_pipeline;

pub(crate) fn create_pipeline_bundle(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    bloom_bind_group_layout: &wgpu::BindGroupLayout,
    cluster_bind_group_layout: &wgpu::BindGroupLayout,
    hzb_bind_group_layout: &wgpu::BindGroupLayout,
    exposure_histogram_bind_group_layout: &wgpu::BindGroupLayout,
    exposure_resolve_bind_group_layout: &wgpu::BindGroupLayout,
    color_lut_bake_bind_group_layout: &wgpu::BindGroupLayout,
    depth_of_field_prepare_bind_group_layout: &wgpu::BindGroupLayout,
    taa_resolve_bind_group_layout: &wgpu::BindGroupLayout,
    velocity_camera_bind_group_layout: &wgpu::BindGroupLayout,
    motion_vector_tile_max_bind_group_layout: &wgpu::BindGroupLayout,
    motion_vector_neighbor_max_bind_group_layout: &wgpu::BindGroupLayout,
    post_process_bind_group_layout: &wgpu::BindGroupLayout,
    upscale_bind_group_layout: &wgpu::BindGroupLayout,
    output_transfer_bind_group_layout: &wgpu::BindGroupLayout,
    smaa_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> PipelineBundle {
    let smaa_pipeline_bundle = smaa_pipeline_bundle(device, target_format, smaa_bind_group_layout);
    PipelineBundle {
        bloom_pipeline: bloom_pipeline(device, target_format, bloom_bind_group_layout),
        cluster_pipeline: cluster_pipeline(device, cluster_bind_group_layout),
        hzb_pipeline: hzb_pipeline(device, hzb_bind_group_layout),
        exposure_histogram_pipeline: exposure_histogram_pipeline(
            device,
            exposure_histogram_bind_group_layout,
        ),
        exposure_resolve_pipeline: exposure_resolve_pipeline(
            device,
            exposure_resolve_bind_group_layout,
        ),
        color_lut_bake_pipeline: color_lut_bake_pipeline(device, color_lut_bake_bind_group_layout),
        depth_of_field_prepare_pipeline: depth_of_field_prepare_pipeline(
            device,
            target_format,
            depth_of_field_prepare_bind_group_layout,
            depth_sampling_mode,
        ),
        depth_of_field_pipeline: depth_of_field_pipeline(
            device,
            POST_PROCESS_INTERMEDIATE_HDR_FORMAT,
            post_process_bind_group_layout,
            depth_sampling_mode,
        ),
        taa_resolve_pipeline: taa_resolve_pipeline(
            device,
            taa_resolve_bind_group_layout,
            depth_sampling_mode,
        ),
        velocity_camera_pipeline: velocity_camera_pipeline(
            device,
            velocity_camera_bind_group_layout,
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
        motion_blur_pipeline: motion_blur_pipeline(
            device,
            POST_PROCESS_INTERMEDIATE_HDR_FORMAT,
            post_process_bind_group_layout,
            depth_sampling_mode,
        ),
        blur_pipeline: blur_pipeline(
            device,
            POST_PROCESS_INTERMEDIATE_HDR_FORMAT,
            post_process_bind_group_layout,
            depth_sampling_mode,
        ),
        scene_composite_pipeline: scene_composite_pipeline(
            device,
            POST_PROCESS_INTERMEDIATE_HDR_FORMAT,
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
        upscale_pipeline: upscale_pipeline(device, upscale_bind_group_layout),
        output_transfer_pipeline: output_transfer_pipeline(
            device,
            target_format,
            output_transfer_bind_group_layout,
        ),
        fxaa_pipeline: fxaa_pipeline(device, target_format, output_transfer_bind_group_layout),
        smaa_edge_pipeline: smaa_pipeline_bundle.edge,
        smaa_blend_pipeline: smaa_pipeline_bundle.blend,
        smaa_resolve_pipeline: smaa_pipeline_bundle.resolve,
    }
}
