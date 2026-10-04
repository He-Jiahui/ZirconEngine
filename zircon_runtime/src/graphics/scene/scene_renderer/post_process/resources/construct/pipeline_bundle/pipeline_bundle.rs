/// 完整场景后处理在设备初始化时构造的管线集合，供每帧图执行按节点复用。
/// 构造时确定输出格式、深度采样模式和 WGSL 入口；重建设备或目标格式后须重建。
pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) struct PipelineBundle
{
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) bloom_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) cluster_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) hzb_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) hzb_msaa_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) half_res_transparency_depth_downsample_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) half_res_transparency_composite_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) exposure_histogram_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) exposure_resolve_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) color_lut_bake_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) depth_of_field_prepare_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) depth_of_field_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) taa_resolve_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) velocity_camera_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) motion_vector_tile_max_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) motion_vector_neighbor_max_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) motion_blur_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) blur_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) scene_composite_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) screen_space_reflection_reflection_pyramid_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) screen_space_reflection_reflection_pyramid_coarse_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) screen_space_reflection_resolve_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) screen_space_reflection_specular_occlusion_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) post_process_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) upscale_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) output_transfer_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) fxaa_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) smaa_edge_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) smaa_blend_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) smaa_resolve_pipeline:
        wgpu::RenderPipeline,
}
