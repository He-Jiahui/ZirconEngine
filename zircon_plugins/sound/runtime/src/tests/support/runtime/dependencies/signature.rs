// 运行时依赖投影保留插件 ID、能力和主依赖属性，与静态清单的同名元组比较。
use super::super::super::types::OptionalFeatureDependencySignature;

pub(super) fn dependency_signature(
    dependency: &zircon_runtime::plugin::PluginFeatureDependency,
) -> OptionalFeatureDependencySignature {
    (
        dependency.plugin_id.clone(),
        dependency.capability.clone(),
        dependency.primary,
    )
}
