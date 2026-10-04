use crate::plugin::PluginModuleManifest;

// owner 已由清单身份规则另行校验；此谓词仅确认命名空间边界，不承担非空检查。
// 直接借用既有 ID，避免每个模块都为同一 owner 分配临时前缀。
fn has_module_owner_prefix(module_name: &str, owner_id: &str) -> bool {
    module_name
        .strip_prefix(owner_id)
        .is_some_and(|suffix| suffix.starts_with('.'))
}

pub(super) fn validate_runtime_plugin_module_name_owner_prefix(
    manifest_label: &str,
    owner_label: &str,
    owner_id: &str,
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    if !has_module_owner_prefix(&module.name, owner_id) {
        diagnostics.push(format!(
            "{manifest_label} module name `{}` must be prefixed by {owner_label} `{owner_id}`",
            module.name
        ));
    }
}

#[cfg(test)]
#[path = "tests/owner_prefix.rs"]
mod tests;
