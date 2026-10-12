#[path = "../../oit_buffers/executors.rs"]
mod executors;
#[path = "../../../oit_buffers/fragment_store_pipeline.rs"]
mod fragment_store_pipeline;
#[path = "../../oit_buffers/resolve_pipeline.rs"]
mod resolve_pipeline;

pub const OIT_FRAGMENT_STORE_EXECUTOR_ID: &str = "oit.fragment_store";
pub const OIT_RESOLVE_EXECUTOR_ID: &str = "oit.resolve";
pub const OIT_RESOLVE_SHADER_SOURCE: &str = include_str!("../../oit_buffers/shaders/resolve.wgsl");
pub const OIT_DRAW_SHADER_SOURCE: &str = include_str!("../../../../shader/includes/zr_oit.wgsl");

pub(crate) use executors::registrations;
pub(in crate::graphics::scene::scene_renderer) use fragment_store_pipeline::OitFragmentStorePipeline;

#[cfg(test)]
#[path = "../../oit_buffers/tests/cases.rs"]
mod tests;
