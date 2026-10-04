mod kind_suffix;
mod owner_prefix;
mod shape;
mod uniqueness;

use crate::plugin::PluginModuleManifest;

use self::{
    kind_suffix::validate_runtime_plugin_module_name_kind_suffix,
    owner_prefix::validate_runtime_plugin_module_name_owner_prefix,
    shape::validate_runtime_plugin_module_name_shape,
    uniqueness::validate_runtime_plugin_module_name_uniqueness,
};

/// 模块名既是注册标识，也需表明所属清单和运行角色；包传包 ID，feature 传 feature ID。
/// 去重结果由该清单的模块投影提供；此处不会验证 owner 本身有效，也不会改写名称。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_module_name(
    manifest_label: &str,
    owner_label: &str,
    owner_id: &str,
    module: &PluginModuleManifest,
    is_duplicate: bool,
    validate_field: fn(&str, &str, &mut Vec<String>),
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_name_shape(
        module,
        validate_field,
        validate_namespace,
        diagnostics,
    );
    validate_runtime_plugin_module_name_owner_prefix(
        manifest_label,
        owner_label,
        owner_id,
        module,
        diagnostics,
    );
    validate_runtime_plugin_module_name_kind_suffix(manifest_label, module, diagnostics);
    validate_runtime_plugin_module_name_uniqueness(
        manifest_label,
        module,
        is_duplicate,
        diagnostics,
    );
}
