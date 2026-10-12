#[path = "../../../../full_scene_post_process_resources.rs"]
mod full_scene_post_process_resources;
#[path = "../../../../profiled_scene_post_process_resources.rs"]
mod profiled_scene_post_process_resources;
#[path = "../../../../post/process_resources/output_transfer_resources.rs"]
mod scene_output_transfer_resources;

pub(in crate::graphics::scene::scene_renderer::post_process) use full_scene_post_process_resources::FullScenePostProcessResources;
pub(crate) use profiled_scene_post_process_resources::ScenePostProcessResources;
pub(in crate::graphics::scene::scene_renderer::post_process) use scene_output_transfer_resources::SceneOutputTransferResources;
