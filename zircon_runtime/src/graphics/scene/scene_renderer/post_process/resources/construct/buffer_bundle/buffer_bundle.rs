pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) struct BufferBundle
{
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) bloom_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) ssao_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) cluster_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) depth_of_field_prepare_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) hzb_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) half_res_transparency_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) exposure_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) color_lut_bake_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) default_exposure_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) default_exposure_histogram_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) taa_resolve_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) velocity_camera_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) light_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) hybrid_gi_probe_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) hybrid_gi_trace_region_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) reflection_probe_buffer:
        wgpu::Buffer,
}
