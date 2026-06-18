pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) struct PipelineBundle {
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) bloom_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) cluster_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) hzb_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) exposure_histogram_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) exposure_resolve_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) color_lut_bake_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) depth_of_field_prepare_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) depth_of_field_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) taa_resolve_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) velocity_camera_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) motion_vector_tile_max_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) motion_vector_neighbor_max_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) motion_blur_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) blur_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) scene_composite_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) screen_space_reflection_reflection_pyramid_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) screen_space_reflection_reflection_pyramid_coarse_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) screen_space_reflection_resolve_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) screen_space_reflection_specular_occlusion_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) post_process_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) upscale_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) output_transfer_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) fxaa_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) smaa_edge_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) smaa_blend_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) smaa_resolve_pipeline:
        wgpu::RenderPipeline,
}
