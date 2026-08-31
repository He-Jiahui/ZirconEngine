#[path = "fullscreen_pass/abi.rs"]
mod abi;
#[path = "fullscreen_pass/builder.rs"]
mod builder;
#[path = "fullscreen_pass/parameter_encoding.rs"]
mod parameter_encoding;
#[path = "fullscreen_pass/pipeline_cache_key.rs"]
mod pipeline_cache_key;
#[path = "fullscreen_pass/plan.rs"]
mod plan;
#[path = "fullscreen_pass/shader_ref.rs"]
mod shader_ref;

pub use abi::{
    FULLSCREEN_FIRST_PASS_INPUT_BINDING, FULLSCREEN_FRAME_GROUP, FULLSCREEN_PARAMS_BINDING,
    FULLSCREEN_PASS_INPUT_GROUP, FULLSCREEN_TRIANGLE_VERTEX_ENTRY,
};
pub use builder::FullscreenPassBuilder;
pub use pipeline_cache_key::FullscreenPipelineCacheKey;
pub use plan::FullscreenPassPlan;
pub use shader_ref::FullscreenShaderRef;

#[cfg(test)]
#[path = "fullscreen_pass/tests.rs"]
mod tests;
