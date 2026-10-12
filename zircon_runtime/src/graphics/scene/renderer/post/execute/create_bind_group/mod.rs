#[path = "../../../../post/create_bind_group/bind_group_entries.rs"]
mod bind_group_entries;
#[path = "../../../../post/execute/create_bind_group/create.rs"]
mod create;

pub(in crate::graphics::scene::scene_renderer::post_process::resources) use create::create_bind_group;
