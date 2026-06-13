use crate::plugin::RuntimeExtensionRegistry;

use super::{project_selection_from_feature_manifest, RuntimePluginFeatureRegistrationReport};
use crate::plugin::runtime_plugin::{
    feature_validation::validate_runtime_plugin_feature_manifest, RuntimePluginFeature,
};

impl RuntimePluginFeatureRegistrationReport {
    pub fn from_feature(feature: &dyn RuntimePluginFeature) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        let manifest = feature.manifest();
        validate_runtime_plugin_feature_manifest(&manifest, &mut diagnostics);
        if let Err(error) = feature.register(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        Self {
            project_selection: project_selection_from_feature_manifest(&manifest),
            provider_package_id: None,
            manifest,
            extensions,
            diagnostics,
        }
    }
}
