use crate::plugin::PluginPackageManifest;

use super::{
    RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

impl RuntimePluginCatalog {
    pub fn registrations(&self) -> &[RuntimePluginRegistrationReport] {
        &self.registrations
    }

    pub fn feature_registrations(&self) -> &[RuntimePluginFeatureRegistrationReport] {
        &self.feature_registrations
    }

    pub fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        self.registrations
            .iter()
            .map(|registration| registration.package_manifest.clone())
            .collect()
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}
