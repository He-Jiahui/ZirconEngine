use std::sync::Mutex;

use super::super::resources::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::resources::terminal_resource_cache::TerminalPostProcessResourceCache;
use crate::graphics::resource_identity::SampledTextureIdentity;
use crate::graphics::scene::scene_renderer::temporal::taa::taa_resolve_bind_group_cache::TaaResolveBindGroupCache;
use crate::graphics::shader::FullscreenPassParameterBindings;
use crate::graphics::scene::scene_renderer::post_process::resources::post_process_pass_parameter_buffers::PostProcessPassParameterBuffers;

pub(crate) struct FullScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_pass_parameter_buffers:
        PostProcessPassParameterBuffers,
    pub(in crate::graphics::scene::scene_renderer) hzb_fallback_resource_identity:
        crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity,
    pub(in crate::graphics::scene::scene_renderer) depth_sampling_mode:
        PostProcessDepthSamplingMode,
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_msaa_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) half_res_transparency_depth_downsample_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) half_res_transparency_composite_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) exposure_histogram_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) exposure_resolve_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) color_lut_bake_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_of_field_prepare_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer) taa_resolve_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer) taa_resolve_bind_group_cache:
        Mutex<TaaResolveBindGroupCache>,
    pub(in crate::graphics::scene::scene_renderer) velocity_camera_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_tile_max_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_tile_max_parameter_bindings:
        FullscreenPassParameterBindings,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_neighbor_max_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) upscale_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) output_transfer_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) smaa_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) terminal_resource_cache:
        TerminalPostProcessResourceCache,
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_msaa_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) half_res_transparency_depth_downsample_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) half_res_transparency_composite_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) exposure_histogram_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) exposure_resolve_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) color_lut_bake_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_of_field_prepare_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_of_field_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer) taa_resolve_pipeline: wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer) velocity_camera_pipeline: wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_tile_max_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_neighbor_max_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_blur_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) blur_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) scene_composite_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_reflection_pyramid_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_reflection_pyramid_coarse_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_resolve_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_specular_occlusion_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) upscale_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) output_transfer_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) fxaa_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) smaa_edge_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) smaa_blend_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) smaa_resolve_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) ssao_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) half_res_transparency_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) exposure_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) color_lut_bake_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) default_exposure_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) default_exposure_histogram_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer) taa_resolve_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) primary_upscale_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) secondary_upscale_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_of_field_prepare_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer) velocity_camera_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) light_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_probe_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_trace_region_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) reflection_probe_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer) black_texture_view: wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer) black_texture_identity: SampledTextureIdentity,
    pub(in crate::graphics::scene::scene_renderer::post_process) white_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_source_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_lut_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_lut_texture_3d_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_lut_sampler: wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::post_process) scene_depth_sampler: wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::post_process) upscale_sampler: wgpu::Sampler,
}
