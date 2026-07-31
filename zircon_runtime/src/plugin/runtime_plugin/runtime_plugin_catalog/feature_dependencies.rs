use std::sync::Arc;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

#[cfg(test)]
use super::feature_resolution::FeatureResolutionStats;
use super::features::feature_dependency_report as build_feature_dependency_report;
#[cfg(test)]
use super::features::feature_dependency_report_with_stats as build_feature_dependency_report_with_stats;
#[cfg(test)]
use super::project_manifest::complete_project_manifest as complete_catalog_project_manifest;
use super::{RuntimePluginCatalog, RuntimePluginFeatureDependencyReport};

impl RuntimePluginCatalog {
    pub fn feature_dependency_report(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Arc<RuntimePluginFeatureDependencyReport> {
        Arc::clone(self.project_plan_for(manifest, target).feature_report())
    }

    pub(crate) fn feature_dependency_report_for_completed_manifest(
        &self,
        completed: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> RuntimePluginFeatureDependencyReport {
        build_feature_dependency_report(&self.projection, completed, target)
    }

    #[cfg(test)]
    pub(super) fn feature_dependency_report_with_stats(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> (RuntimePluginFeatureDependencyReport, FeatureResolutionStats) {
        let completed =
            complete_catalog_project_manifest(&self.registrations, &self.projection, manifest);
        build_feature_dependency_report_with_stats(&self.projection, &completed, target)
    }
}
