use super::super::runtime::ShaderRuntime;
use crate::core::resource::ResourceReadinessRowIdentity;
use crate::plugin::ShaderModuleSourceBinding;

/// 同时保留 shader 源版本和依赖发布身份，源版本未变时也能识别依赖失效；仅带导入路径的 include 资产建立模块源绑定。
pub(in crate::graphics::scene::resources) struct PreparedShader {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) dependency_identity: ResourceReadinessRowIdentity,
    pub(in crate::graphics::scene::resources) runtime: ShaderRuntime,
    pub(in crate::graphics::scene::resources) module_source_binding:
        Option<ShaderModuleSourceBinding>,
}
