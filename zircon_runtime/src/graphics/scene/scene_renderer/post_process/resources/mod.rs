//! 后处理 GPU 执行层：初始化资源归构造入口，帧参数经上传事务进入各个图节点。
//! 局部中间目标和终端物理视口使用不同坐标空间，区域选择由各效果入口负责。
mod construct;
pub(in crate::graphics::scene::scene_renderer::post_process) mod depth_sampling_mode;
mod execute_bloom;
mod execute_blur;
mod execute_clustered_lighting;
mod execute_color_lut_bake;
mod execute_depth_of_field;
mod execute_depth_of_field_prepare;
mod execute_exposure;
mod execute_fxaa;
mod execute_half_res_transparency;
mod execute_hzb_build;
mod execute_motion_blur;
mod execute_motion_vector_neighbor_max;
mod execute_motion_vector_tile_max;
mod execute_output_transfer;
mod execute_post_process;
mod execute_scene_composite;
mod execute_screen_space_reflection_reflection_pyramid;
mod execute_screen_space_reflection_reflection_pyramid_coarse;
mod execute_screen_space_reflection_resolve;
mod execute_screen_space_reflection_specular_occlusion;
mod execute_smaa;
mod execute_upscale;
pub(in crate::graphics::scene::scene_renderer::post_process) mod post_process_pass_parameter_buffers;
mod render_region;
pub(in crate::graphics::scene::scene_renderer::post_process) mod shader_sources;
pub(in crate::graphics::scene::scene_renderer::post_process) mod terminal_resource_cache;

pub(in crate::graphics::scene::scene_renderer) use execute_color_lut_bake::{
    color_lut_bake_dispatch_groups, color_lut_bake_workgroup_size,
};
