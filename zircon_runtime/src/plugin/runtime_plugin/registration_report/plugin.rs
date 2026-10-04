use crate::plugin::runtime_plugin::{
    descriptor::validate_runtime_plugin_descriptor, RuntimePlugin,
};
use crate::plugin::RuntimeExtensionRegistry;

use super::{
    package_contributions::register_package_manifest_contributions,
    validation::{
        validate_runtime_plugin_package_manifest, validate_runtime_plugin_registration_interfaces,
        validate_runtime_plugin_registration_system_anchors,
    },
    RuntimePluginRegistrationReport,
};

impl RuntimePluginRegistrationReport {
    /// 调用链接插件登记扩展后，再并入清单贡献并核对接口、系统锚点；问题随报告返回。
    pub fn from_plugin(plugin: &dyn RuntimePlugin) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let package_manifest = plugin.package_manifest();
        let mut diagnostics = Vec::with_capacity(package_manifest.modules.len());
        validate_runtime_plugin_descriptor(plugin, &mut diagnostics);
        if let Err(error) = extensions.register_module(plugin.module_descriptor().clone()) {
            diagnostics.push(error.to_string());
        }
        if let Err(error) = plugin.register(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        for source in plugin.shader_module_sources() {
            if let Err(error) =
                extensions.register_plugin_shader_module_source(&package_manifest.id, source)
            {
                diagnostics.push(error.to_string());
            }
        }
        let projection = validate_runtime_plugin_package_manifest(
            Some(plugin.descriptor()),
            &package_manifest,
            &mut diagnostics,
        );
        register_package_manifest_contributions(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        validate_runtime_plugin_registration_interfaces(
            &package_manifest,
            &projection,
            &extensions,
            &mut diagnostics,
        );
        validate_runtime_plugin_registration_system_anchors(
            &package_manifest,
            &projection,
            &extensions,
            &mut diagnostics,
        );
        Self {
            package_manifest,
            project_selection: plugin.project_selection(),
            extensions,
            diagnostics,
        }
    }
}

#[cfg(test)]
#[path = "tests/plugin_optimization_batch_20260830bx_runtime_tests.rs"]
mod optimization_batch_20260830bx_runtime_tests;

#[cfg(test)]
#[path = "tests/plugin.rs"]
mod tests;
