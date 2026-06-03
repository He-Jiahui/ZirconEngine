use crate::core::ModuleDescriptor;
use crate::plugin::{PluginFeatureBundleManifest, PluginModuleKind, RuntimeExtensionRegistry};

use super::{project_selection_from_feature_manifest, RuntimePluginFeatureRegistrationReport};
use crate::plugin::runtime_plugin::feature_validation::{
    validate_runtime_plugin_feature_manifest, validate_runtime_plugin_feature_provider_package_id,
};

impl RuntimePluginFeatureRegistrationReport {
    pub fn from_native_feature_manifest(
        manifest: PluginFeatureBundleManifest,
        provider_package_id: Option<String>,
    ) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
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
            if let Err(error) = extensions.register_module(ModuleDescriptor::new(
                module.name.clone(),
                format!(
                    "Native dynamic runtime plugin feature module provided by {}",
                    manifest.id
                ),
            )) {
                diagnostics.push(error.to_string());
            }
        }
        let mut project_selection = project_selection_from_feature_manifest(&manifest);
        project_selection.provider_package_id = provider_package_id.clone();
        Self {
            project_selection,
            provider_package_id,
            manifest,
            extensions,
            diagnostics,
        }
    }
}
