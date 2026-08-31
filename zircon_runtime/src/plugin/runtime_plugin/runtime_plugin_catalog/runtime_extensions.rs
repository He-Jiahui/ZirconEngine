use std::sync::Arc;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

use super::extension_report::{runtime_extension_report, RuntimeExtensionCatalogReport};
use super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn runtime_extensions(&self) -> RuntimeExtensionCatalogReport {
        runtime_extension_report(&self.registrations)
    }

    pub fn runtime_extensions_for_project(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Arc<RuntimeExtensionCatalogReport> {
        Arc::clone(
            &self
                .compiled_project_plan(manifest, target)
                .extension_report,
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
