mod build_post_process_params;
mod create_bind_group;
mod pass_params_buffer;
mod prepare_scene_data_uploads;
mod run;

pub(in crate::graphics::scene::scene_renderer::post_process::resources) use build_post_process_params::build_post_process_params;
pub(in crate::graphics::scene::scene_renderer::post_process::resources) use create_bind_group::create_bind_group;
pub(in crate::graphics::scene::scene_renderer::post_process::resources) use pass_params_buffer::post_process_params_upload;
pub(in crate::graphics::scene::scene_renderer::post_process::resources) use prepare_scene_data_uploads::prepare_scene_data_uploads;
