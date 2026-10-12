use crate::plugin::{PluginModuleKind, PluginModuleManifest};

/// Runtime 与 Editor 的能力前缀表达运行角色；Native/VM 只受共用命名空间规则约束。
/// 分发模块可复用 Runtime 能力，不能仅因它是独立二进制产物就要求新的能力前缀。
pub(super) fn validate_runtime_plugin_module_capability_kind_prefix(
    manifest_label: &str,
    module: &PluginModuleManifest,
    capability: &str,
    diagnostics: &mut Vec<String>,
) {
    match module.kind {
        PluginModuleKind::Runtime if !capability.starts_with("runtime.") => {
            diagnostics.push(format!(
                "{manifest_label} runtime module `{}` capability `{capability}` must start with `runtime.`",
                module.name
            ));
        }
        PluginModuleKind::Editor if !capability.starts_with("editor.") => {
            diagnostics.push(format!(
                "{manifest_label} editor module `{}` capability `{capability}` must start with `editor.`",
                module.name
            ));
        }
        _ => {}
    }
}
