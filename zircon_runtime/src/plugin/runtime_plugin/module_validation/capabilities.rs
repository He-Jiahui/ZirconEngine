mod presence;
mod row;
mod rows;

use crate::plugin::PluginModuleManifest;

use self::{
    presence::validate_runtime_plugin_module_capability_presence,
    rows::validate_runtime_plugin_module_capability_rows,
};

/// 包与 feature 清单共用的模块能力声明门槛；这里只累积诊断，不注册能力。
/// 调用端负责提供所属清单的字段规则和模块内去重结果；投影与模块列表须来自同一份清单。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_module_capabilities(
    manifest_label: &str,
    module: &PluginModuleManifest,
    validate_field: Option<fn(&str, &str, &mut Vec<String>)>,
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    is_duplicate: impl Fn(usize) -> bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_capability_presence(manifest_label, module, diagnostics);
    validate_runtime_plugin_module_capability_rows(
        manifest_label,
        module,
        validate_field,
        validate_namespace,
        is_duplicate,
        diagnostics,
    );
}
