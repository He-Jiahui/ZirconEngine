/// 要求清单中的 UI 描述符由当前包承载，避免资源绑定与声明所有者分离。
/// 这里只检查包身份；资源路径及组件字段仍需经过注册表校验。
pub(super) fn validate_runtime_plugin_package_ui_component_owner(
    ui_component_id: &str,
    ui_component_plugin_id: &str,
    package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    if ui_component_plugin_id != package_id {
        diagnostics.push(format!(
            "runtime plugin package manifest ui component `{ui_component_id}` plugin_id `{ui_component_plugin_id}` must match package id `{package_id}`"
        ));
    }
}
