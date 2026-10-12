use super::RuntimePluginFeatureRegistrationReport;
use crate::plugin::runtime_plugin::feature_validation::validate_runtime_plugin_feature_provider_package_id;
use crate::plugin::PluginPackageRole;

impl RuntimePluginFeatureRegistrationReport {
    pub fn provider_package_id_or_owner(&self) -> &str {
        self.provider_package_id
            .as_deref()
            .unwrap_or(self.manifest.owner_plugin_id.as_str())
    }

    /// 校验覆盖值并同步到报告与项目选择；格式问题留在报告诊断中。
    pub fn with_provider_package_id(mut self, package_id: impl Into<String>) -> Self {
        let package_id = package_id.into();
        validate_runtime_plugin_feature_provider_package_id(&package_id, &mut self.diagnostics);
        self.project_selection.provider_package_id = Some(package_id.clone());
        self.provider_package_id = Some(package_id);
        self
    }

    /// Attaches the manifest role that authorizes this feature provider for product selection.
    pub fn with_provider_package_role(mut self, package_role: PluginPackageRole) -> Self {
        self.provider_package_role = package_role;
        self
    }

    pub(crate) fn is_product_catalog_eligible(&self) -> bool {
        self.provider_package_role.is_product_catalog_eligible()
    }
}
