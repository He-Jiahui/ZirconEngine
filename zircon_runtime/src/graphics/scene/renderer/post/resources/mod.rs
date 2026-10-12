//! 后处理 GPU 执行层：初始化资源归构造入口，帧参数经上传事务进入各个图节点。
//! 局部中间目标和终端物理视口使用不同坐标空间，区域选择由各效果入口负责。
#[path = "construct/mod.rs"]
mod construct;
#[path = "depth_sampling_mode.rs"]
pub(in crate::graphics::scene::scene_renderer::post_process) mod depth_sampling_mode;
#[path = "execute_bloom/mod.rs"]
mod execute_bloom;
#[path = "execute_blur/mod.rs"]
mod execute_blur;
#[path = "../../../post/execute_clustered_lighting/mod.rs"]
mod execute_clustered_lighting;
#[path = "../execute_color_lut_bake/mod.rs"]
mod execute_color_lut_bake;
#[path = "../execute_depth_of_field/mod.rs"]
mod execute_depth_of_field;
#[path = "../../../post/execute_depth_of_field_prepare/mod.rs"]
mod execute_depth_of_field_prepare;
#[path = "../execute_exposure/mod.rs"]
mod execute_exposure;
#[path = "execute_fxaa/mod.rs"]
mod execute_fxaa;
#[path = "../../../../post/resources/execute_half_res_transparency.rs"]
mod execute_half_res_transparency;
#[path = "../execute_hzb_build/mod.rs"]
mod execute_hzb_build;
#[path = "../execute_motion_blur/mod.rs"]
mod execute_motion_blur;
#[path = "../../../post/execute_motion_vector_neighbor_max/mod.rs"]
mod execute_motion_vector_neighbor_max;
#[path = "../../../post/execute_motion_vector_tile_max/mod.rs"]
mod execute_motion_vector_tile_max;
#[path = "../execute_output_transfer/mod.rs"]
mod execute_output_transfer;
#[path = "../execute_post_process/mod.rs"]
mod execute_post_process;
#[path = "../execute_scene_composite/mod.rs"]
mod execute_scene_composite;
#[path = "ssr_pyramid/mod.rs"]
mod execute_screen_space_reflection_reflection_pyramid;
#[path = "../ssr_pyramid_coarse/mod.rs"]
mod execute_screen_space_reflection_reflection_pyramid_coarse;
#[path = "ssr_resolve/mod.rs"]
mod execute_screen_space_reflection_resolve;
#[path = "../ssr_specular_occlusion/mod.rs"]
mod execute_screen_space_reflection_specular_occlusion;
#[path = "execute_smaa/mod.rs"]
mod execute_smaa;
#[path = "execute_upscale/mod.rs"]
mod execute_upscale;
#[path = "../../../../post/resources/process_pass_parameter_buffers.rs"]
pub(in crate::graphics::scene::scene_renderer::post_process) mod post_process_pass_parameter_buffers;
#[path = "render_region.rs"]
mod render_region;
#[path = "shader_sources.rs"]
pub(in crate::graphics::scene::scene_renderer::post_process) mod shader_sources;
#[path = "../../../post/resources/terminal_resource_cache.rs"]
pub(in crate::graphics::scene::scene_renderer::post_process) mod terminal_resource_cache;

pub(in crate::graphics::scene::scene_renderer) use execute_color_lut_bake::{
    color_lut_bake_dispatch_groups, color_lut_bake_workgroup_size,
};
