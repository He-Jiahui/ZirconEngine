use crate::plugin::PluginDependencyManifest;

/// 仅请求接口的依赖允许省略能力；调用端先保证能力或接口至少声明一项，再解释此返回值。
pub(super) fn validate_runtime_plugin_package_dependency_capability_presence<'a>(
    dependency: &'a PluginDependencyManifest,
    _diagnostics: &mut Vec<String>,
) -> Option<&'a str> {
    dependency.capability.as_deref()
}
