#[path = "material_runtime.rs"]
mod material_runtime;
#[path = "shader_runtime.rs"]
mod shader_runtime;

pub(crate) use material_runtime::MaterialCaptureSeed;
pub(crate) use material_runtime::{MaterialDisabledPasses, MaterialRuntime};
pub(in crate::graphics::scene::resources) use shader_runtime::ShaderRuntime;
