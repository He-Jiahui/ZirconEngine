use super::super::runtime::ShaderRuntime;

pub(in crate::scene::resources) struct PreparedShader {
    pub(in crate::scene::resources) revision: u64,
    pub(in crate::scene::resources) runtime: ShaderRuntime,
}
