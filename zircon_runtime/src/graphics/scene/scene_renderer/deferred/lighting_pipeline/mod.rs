mod create;
mod shader_source;
#[cfg(test)]
mod tests;

pub(in crate::graphics::scene::scene_renderer::deferred) use create::create_lighting_pipeline;
