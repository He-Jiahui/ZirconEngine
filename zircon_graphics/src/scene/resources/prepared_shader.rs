use super::shader_runtime::ShaderRuntime;

pub(super) struct PreparedShader {
    pub(super) revision: u64,
    pub(super) runtime: ShaderRuntime,
}
