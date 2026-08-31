use super::super::runtime::ShaderRuntime;
use crate::plugin::ShaderModuleSourceBinding;

pub(in crate::graphics::scene::resources) struct PreparedShader {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) dependency_revision: u64,
    pub(in crate::graphics::scene::resources) runtime: ShaderRuntime,
    pub(in crate::graphics::scene::resources) module_source_binding:
        Option<ShaderModuleSourceBinding>,
}
