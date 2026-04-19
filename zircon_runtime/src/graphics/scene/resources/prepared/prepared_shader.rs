use super::super::runtime::ShaderRuntime;

pub(in crate::graphics::scene::resources) struct PreparedShader {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) runtime: ShaderRuntime,
}
