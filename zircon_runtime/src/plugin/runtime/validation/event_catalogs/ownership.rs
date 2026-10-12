use super::prefix::runtime_plugin_package_event_catalog_has_owner;

/// 对包声明的事件目录检查归属，防止仅共享字符串前缀的包冒用命名空间。
/// 该诊断与目录重复检查独立累计，完整命名空间的合法性仍由注册入口检查。
pub(super) fn validate_runtime_plugin_package_event_catalog_owner(
    event_catalog_namespace: &str,
    package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    if !runtime_plugin_package_event_catalog_has_owner(package_id, event_catalog_namespace) {
        diagnostics.push(format!(
            "runtime plugin package manifest event catalog namespace `{event_catalog_namespace}` must be prefixed by package id `{package_id}`"
        ));
    }
}
