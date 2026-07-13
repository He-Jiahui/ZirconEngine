use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

use super::extension_report::{
    runtime_extension_report, runtime_extension_report_for_project, RuntimeExtensionCatalogReport,
};
use super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn runtime_extensions(&self) -> RuntimeExtensionCatalogReport {
        runtime_extension_report(&self.registrations)
    }

    pub fn runtime_extensions_for_project(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> RuntimeExtensionCatalogReport {
        let completed = self.complete_project_manifest(manifest);
        let feature_report = self.feature_dependency_report(&completed, target);
        runtime_extension_report_for_project(
            &self.registrations,
            &self.feature_registrations,
            &completed,
            target,
            &feature_report,
        )
    }
}
