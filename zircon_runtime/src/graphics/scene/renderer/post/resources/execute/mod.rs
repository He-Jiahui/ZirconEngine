#[path = "../../../../post/execute/build_post_process_params/mod.rs"]
mod build_post_process_params;
#[path = "../../execute/create_bind_group/mod.rs"]
mod create_bind_group;
#[path = "../../execute/pass_params_buffer.rs"]
mod pass_params_buffer;
#[path = "../../../../post/execute/prepare_scene_data_uploads.rs"]
mod prepare_scene_data_uploads;
#[path = "run/mod.rs"]
mod run;

pub(in crate::graphics::scene::scene_renderer::post_process::resources) use build_post_process_params::build_post_process_params;
pub(in crate::graphics::scene::scene_renderer::post_process::resources) use create_bind_group::create_bind_group;
pub(in crate::graphics::scene::scene_renderer::post_process::resources) use pass_params_buffer::post_process_params_upload;
pub(in crate::graphics::scene::scene_renderer::post_process::resources) use prepare_scene_data_uploads::prepare_scene_data_uploads;
