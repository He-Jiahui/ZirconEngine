use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

use super::extension_report::{
    runtime_extension_report, runtime_extension_report_for_project, RuntimeExtensionCatalogReport,
};
use super::features::feature_dependency_report;
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
        let feature_report = feature_dependency_report(
            &self.registrations,
            &self.feature_registrations,
            &completed,
            target,
        );
        runtime_extension_report_for_project(
            &self.registrations,
            &self.feature_registrations,
            &completed,
            target,
            &feature_report,
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_extension_report_does_not_complete_an_already_completed_manifest() {
        let source = include_str!("runtime_extensions.rs");
        let repeated_completion = ["self", ".feature_dependency_report(&completed"].concat();
        assert!(!source.contains(&repeated_completion));
    }
}
