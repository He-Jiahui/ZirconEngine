use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    RuntimePluginCatalog, RuntimePluginFeatureDependencyReport,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};
use crate::core::framework::platform::RuntimeTargetMode;

pub(super) struct RuntimeModuleFeatureReports<'a> {
    dependency_report: RuntimePluginFeatureDependencyReport,
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
                .into_iter()
                .map(RuntimeModuleLoadDiagnostic::FeatureBlocked)
                .chain(
                    self.dependency_report
                        .diagnostics
                        .into_iter()
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
    feature_registrations
        .iter()
        .filter(|registration| {
            feature_report
                .available_features
                .iter()
                .any(|id| id == &registration.manifest.id)
        })
        .collect()
}
