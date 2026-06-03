use super::RuntimePluginFeatureRegistrationReport;
use crate::plugin::runtime_plugin::feature_validation::validate_runtime_plugin_feature_provider_package_id;

impl RuntimePluginFeatureRegistrationReport {
    pub fn provider_package_id_or_owner(&self) -> &str {
        self.provider_package_id
            .as_deref()
            .unwrap_or(self.manifest.owner_plugin_id.as_str())
    }

    pub fn with_provider_package_id(mut self, package_id: impl Into<String>) -> Self {
        let package_id = package_id.into();
        validate_runtime_plugin_feature_provider_package_id(&package_id, &mut self.diagnostics);
        self.project_selection.provider_package_id = Some(package_id.clone());
        self.provider_package_id = Some(package_id);
        self
    }
}
