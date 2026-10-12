#[path = "create.rs"]
mod create;
#[path = "../../../deferred/lighting_pipeline/shader_source.rs"]
mod shader_source;
#[cfg(test)]
#[path = "../tests/cases.rs"]
mod tests;

pub(in crate::graphics::scene::scene_renderer::deferred) use create::DeferredLightingPipelineCache;
