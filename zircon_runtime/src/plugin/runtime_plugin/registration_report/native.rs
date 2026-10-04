mod runtime_modules;

use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

use self::runtime_modules::register_native_package_runtime_modules;
use super::{
    native_package_projection::native_project_selection_from_package,
    package_contributions::register_package_manifest_contributions,
    validation::validate_runtime_plugin_package_manifest, RuntimePluginRegistrationReport,
};

impl RuntimePluginRegistrationReport {
    /// 原生包路径先累积清单诊断，再登记 Runtime 模块与清单扩展贡献，不调用链接插件钩子。
    pub fn from_native_package_manifest(package_manifest: PluginPackageManifest) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::with_capacity(package_manifest.modules.len());
        drop(validate_runtime_plugin_package_manifest(
            None,
            &package_manifest,
            &mut diagnostics,
        ));
        register_native_package_runtime_modules(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        register_package_manifest_contributions(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        Self {
            project_selection: native_project_selection_from_package(&package_manifest),
            package_manifest,
            extensions,
            diagnostics,
        }
    }
}

#[cfg(test)]
#[path = "tests/native_optimization_batch_20260830bw_runtime_tests.rs"]
mod optimization_batch_20260830bw_runtime_tests;
