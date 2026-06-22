pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) struct BufferBundle {
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) bloom_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) ssao_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) cluster_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) depth_of_field_prepare_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) hzb_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) exposure_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) color_lut_bake_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) default_exposure_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) default_exposure_histogram_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) taa_resolve_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) velocity_camera_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) light_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) hybrid_gi_probe_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) hybrid_gi_trace_region_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) reflection_probe_buffer:
        wgpu::Buffer,
}
