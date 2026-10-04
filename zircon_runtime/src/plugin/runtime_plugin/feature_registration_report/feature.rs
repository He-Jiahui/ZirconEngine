use crate::plugin::{PluginPackageRole, RuntimeExtensionRegistry};

use super::{project_selection_from_feature_manifest, RuntimePluginFeatureRegistrationReport};
use crate::plugin::runtime_plugin::{
    feature_validation::validate_runtime_plugin_feature_manifest, RuntimePluginFeature,
};

impl RuntimePluginFeatureRegistrationReport {
    /// 先读取并校验清单，再调用特性实现登记扩展；校验诊断不会短路这个钩子。
    pub fn from_feature(feature: &dyn RuntimePluginFeature) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let manifest = feature.manifest();
        let mut diagnostics = Vec::with_capacity(manifest.modules.len());
        validate_runtime_plugin_feature_manifest(&manifest, &mut diagnostics);
        if let Err(error) = feature.register(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        Self {
            project_selection: project_selection_from_feature_manifest(&manifest),
            provider_package_id: None,
            provider_package_role: PluginPackageRole::Production,
            manifest,
            extensions,
            diagnostics,
        }
    }
}

#[cfg(test)]
#[path = "tests/feature_optimization_batch_20260830bv_runtime_tests.rs"]
mod optimization_batch_20260830bv_runtime_tests;
