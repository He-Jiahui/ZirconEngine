use crate::plugin::PluginPackageManifest;

use super::super::{
    projection::RuntimePluginPackageValidationProjection,
    validate_runtime_plugin_package_namespace, validate_runtime_plugin_package_token,
};

pub(super) fn validate_runtime_plugin_package_provided_interfaces(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (interface_index, interface) in package_manifest.provides_interfaces.iter().enumerate() {
        validate_runtime_plugin_package_namespace(
            "provided interface id",
            interface.id.as_str(),
            diagnostics,
        );
        if projection.provided_interface_is_duplicate(interface_index) {
            diagnostics.push(format!(
                "runtime plugin package manifest provided interface `{}` must be unique",
                interface.id
            ));
        }
        validate_interface_methods(
            interface_index,
            &interface.id,
            interface,
            projection,
            diagnostics,
        );
    }
}

// 原生桥接按接口方法槽与方法名查找绑定；包声明先保证二者在各自接口中没有冲突。
fn validate_interface_methods(
    interface_index: usize,
    interface_id: &str,
    interface: &crate::plugin::PluginInterfaceManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (method_index, method) in interface.methods.iter().enumerate() {
        validate_runtime_plugin_package_token(
            "provided interface method name",
            method.name.as_str(),
            diagnostics,
        );
        if projection.provided_method_name_is_duplicate(interface_index, method_index) {
            diagnostics.push(format!(
                "runtime plugin package manifest provided interface `{interface_id}` method `{}` must be unique",
                method.name
            ));
        }
        if projection.provided_method_slot_is_duplicate(interface_index, method_index) {
            diagnostics.push(format!(
                "runtime plugin package manifest provided interface `{interface_id}` method slot {} must be unique",
                method.method_slot
            ));
        }
        // BUG: [CR-PLUGIN-VALIDATION-0365] 直接构造的桥接方法可声明重名参数并通过包报告校验；证据：这里只检查令牌，静态清单和脚本 Host 注册均拒绝重名。
        for parameter in &method.parameters {
            validate_runtime_plugin_package_token(
                "provided interface method parameter name",
                parameter.name.as_str(),
                diagnostics,
            );
        }
        for (capability_index, capability) in method.required_capabilities.iter().enumerate() {
            validate_runtime_plugin_package_namespace(
                "provided interface method required capability",
                capability,
                diagnostics,
            );
            if projection.provided_method_capability_is_duplicate(
                interface_index,
                method_index,
                capability_index,
            ) {
                diagnostics.push(format!(
                    "runtime plugin package manifest provided interface `{interface_id}` method `{}` required capability `{capability}` must be unique",
                    method.name
                ));
            }
        }
    }
}
