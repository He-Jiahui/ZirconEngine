use crate::plugin::{PluginModuleKind, PluginModuleManifest};

/// 运行与编辑模块采用角色后缀，使清单标识与 kind 一致；项目选择仍按 kind 分派。
/// Native/VM 的产物名称不套用该后缀约束；例如分发模块可以使用所属 feature 的分发后缀。
pub(super) fn validate_runtime_plugin_module_name_kind_suffix(
    manifest_label: &str,
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    match module.kind {
        PluginModuleKind::Runtime if !module.name.ends_with(".runtime") => {
            diagnostics.push(format!(
                "{manifest_label} runtime module name `{}` must end with `.runtime`",
                module.name
            ));
        }
        PluginModuleKind::Editor if !module.name.ends_with(".editor") => {
            diagnostics.push(format!(
                "{manifest_label} editor module name `{}` must end with `.editor`",
                module.name
            ));
        }
        _ => {}
    }
}
