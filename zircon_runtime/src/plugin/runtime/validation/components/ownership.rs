/// 防止清单以当前包的注册入口声明另一插件拥有的组件类型。
/// 使用精确包身份比较；诊断不替调用方重写组件的所有者。
pub(super) fn validate_runtime_plugin_package_component_owner(
    component_type_id: &str,
    component_plugin_id: &str,
    package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    if component_plugin_id != package_id {
        diagnostics.push(format!(
            "runtime plugin package manifest component type `{component_type_id}` plugin_id `{component_plugin_id}` must match package id `{package_id}`"
        ));
    }
}
