mod material_runtime;
mod shader_runtime;

pub(crate) use material_runtime::MaterialRuntime;
pub(in crate::graphics::scene::resources) use shader_runtime::ShaderRuntime;
