mod executors;
mod fragment_store_pipeline;
mod resolve_pipeline;

pub const OIT_FRAGMENT_STORE_EXECUTOR_ID: &str = "oit.fragment_store";
pub const OIT_RESOLVE_EXECUTOR_ID: &str = "oit.resolve";
pub const OIT_RESOLVE_SHADER_SOURCE: &str = include_str!("shaders/resolve.wgsl");
pub const OIT_DRAW_SHADER_SOURCE: &str = include_str!("../../../../shader/includes/zr_oit.wgsl");

pub(crate) use executors::registrations;
pub(in crate::graphics::scene::scene_renderer) use fragment_store_pipeline::OitFragmentStorePipeline;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
