/// 归属来自包投影的包能力与可选特性能力集合；模块或扩展特性中的同名声明不会自动满足约束。
pub(super) fn validate_runtime_plugin_package_capability_status_ownership(
    capability: &str,
    is_owned: bool,
    diagnostics: &mut Vec<String>,
) {
    if !is_owned {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{}` must reference a package or optional feature capability declared by the same package",
            capability
        ));
    }
}
