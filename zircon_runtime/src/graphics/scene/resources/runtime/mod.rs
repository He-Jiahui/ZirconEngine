mod material_runtime;
mod shader_runtime;

#[cfg(test)]
pub(crate) use material_runtime::MaterialCaptureSeed;
pub(crate) use material_runtime::MaterialRuntime;
pub(in crate::graphics::scene::resources) use shader_runtime::ShaderRuntime;
