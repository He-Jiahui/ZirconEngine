use crate::{plugin::ProjectPluginManifest, RuntimeTargetMode};

use super::features::feature_dependency_report as build_feature_dependency_report;
use super::{RuntimePluginCatalog, RuntimePluginFeatureDependencyReport};

impl RuntimePluginCatalog {
    pub fn feature_dependency_report(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> RuntimePluginFeatureDependencyReport {
        let completed = self.complete_project_manifest(manifest);
        build_feature_dependency_report(
            &self.registrations,
            &self.feature_registrations,
            &completed,
            target,
        )
    }
}
