use std::{collections::HashSet, sync::Arc};

use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    RuntimePluginCatalog, RuntimePluginFeatureDependencyReport,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};
use crate::core::framework::platform::RuntimeTargetMode;

pub(super) struct RuntimeModuleFeatureReports<'a> {
    dependency_report: Arc<RuntimePluginFeatureDependencyReport>,
    active_feature_registrations: Vec<&'a RuntimePluginFeatureRegistrationReport>,
}

impl<'a> RuntimeModuleFeatureReports<'a> {
    pub(super) fn active_feature_registrations(
        &self,
    ) -> &[&'a RuntimePluginFeatureRegistrationReport] {
        &self.active_feature_registrations
    }

    pub(super) fn extend_load_report_diagnostics(self, report: &mut RuntimeModuleLoadReport) {
        report.extend_diagnostics(
            self.dependency_report
                .blocked_features
                .iter()
                .cloned()
                .map(RuntimeModuleLoadDiagnostic::FeatureBlocked)
                .chain(
                    self.dependency_report
                        .diagnostics
                        .iter()
                        .cloned()
                        .map(RuntimeModuleLoadDiagnostic::FeatureDefinition),
                ),
        );
    }
}

pub(super) fn feature_reports_for_plugin_and_feature_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &'a [RuntimePluginFeatureRegistrationReport],
) -> RuntimeModuleFeatureReports<'a> {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.iter().cloned(),
        feature_registrations.iter().cloned(),
    );
    let dependency_report = catalog.feature_dependency_report(manifest, target);
    let active_feature_registrations =
        active_feature_registration_refs(feature_registrations, &dependency_report);
    RuntimeModuleFeatureReports {
        dependency_report,
        active_feature_registrations,
    }
}

fn active_feature_registration_refs<'a>(
    feature_registrations: &'a [RuntimePluginFeatureRegistrationReport],
    feature_report: &RuntimePluginFeatureDependencyReport,
) -> Vec<&'a RuntimePluginFeatureRegistrationReport> {
    let mut available_features = HashSet::with_capacity(feature_report.available_features.len());
    available_features.extend(feature_report.available_features.iter().map(String::as_str));
    let mut active_feature_registrations = Vec::with_capacity(feature_registrations.len());
    active_feature_registrations.extend(
        feature_registrations
            .iter()
            .filter(|registration| available_features.contains(registration.manifest.id.as_str())),
    );
    active_feature_registrations
}

#[cfg(test)]
#[path = "tests/feature_reports_optimization_tests.rs"]
mod optimization_tests;
