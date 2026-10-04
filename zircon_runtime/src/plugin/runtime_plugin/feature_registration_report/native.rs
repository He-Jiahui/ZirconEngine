use crate::plugin::{
    PluginFeatureBundleManifest, PluginModuleKind, PluginPackageRole, RuntimeExtensionRegistry,
};

use super::{project_selection_from_feature_manifest, RuntimePluginFeatureRegistrationReport};
use crate::plugin::runtime_plugin::feature_validation::{
    validate_runtime_plugin_feature_manifest, validate_runtime_plugin_feature_provider_package_id,
};

impl RuntimePluginFeatureRegistrationReport {
    /// 原生清单路径保留校验诊断，并仅把 Runtime 模块描述符收集进报告扩展表。
    pub fn from_native_feature_manifest(
        manifest: PluginFeatureBundleManifest,
        provider_package_id: Option<String>,
    ) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::with_capacity(manifest.modules.len());
        validate_runtime_plugin_feature_manifest(&manifest, &mut diagnostics);
        if let Some(provider_package_id) = provider_package_id.as_deref() {
            validate_runtime_plugin_feature_provider_package_id(
                provider_package_id,
                &mut diagnostics,
            );
        }
        for module in manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
        {
            if let Err(error) = extensions.register_module(module.module_descriptor()) {
                diagnostics.push(error.to_string());
            }
        }
        let mut project_selection = project_selection_from_feature_manifest(&manifest);
        project_selection.provider_package_id = provider_package_id.clone();
        Self {
            project_selection,
            provider_package_id,
            provider_package_role: PluginPackageRole::Production,
            manifest,
            extensions,
            diagnostics,
        }
    }
}

#[cfg(test)]
#[path = "tests/native_optimization_batch_20260830bu_runtime_tests.rs"]
mod optimization_batch_20260830bu_runtime_tests;
