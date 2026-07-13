use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
    RuntimeProfileDescriptor,
};

use super::super::availability::{
    runtime_profile_manifest_availability, target_manifest_availability_for_registration_reports,
};
use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};
use super::super::manifest::manifest_with_mode_baseline;
use super::feature_reports::feature_reports_for_plugin_and_feature_registration_reports;
use super::registration_inputs::{
    registration_inputs_for_plugin_and_feature_reports, registration_inputs_for_plugin_reports,
};
use super::target_modules::runtime_modules_for_target_with_registration_inputs_for_manifest;
use crate::core::framework::platform::RuntimeTargetMode;

pub(super) fn runtime_modules_for_target_with_plugin_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = active_plugin_registration_refs(target, registrations);
    let inputs = registration_inputs_for_plugin_reports(&registrations);
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let mut report = runtime_modules_for_target_with_registration_inputs_for_manifest(
        target, &manifest, &inputs,
    );
    extend_asset_importer_errors(&mut report, &inputs);
    report.runtime_plugin_availability = target_manifest_availability_for_registration_reports(
        target,
        &manifest,
        registrations.iter().copied(),
    );
    report
}

pub(super) fn runtime_modules_for_profile_manifest_with_plugin_registration_reports<'a>(
    profile: &RuntimeProfileDescriptor,
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = active_plugin_registration_refs(target, registrations);
    let inputs = registration_inputs_for_plugin_reports(&registrations);
    let mut report =
        runtime_modules_for_target_with_registration_inputs_for_manifest(target, manifest, &inputs);
    extend_asset_importer_errors(&mut report, &inputs);
    report.runtime_plugin_availability =
        runtime_profile_manifest_availability(profile, manifest, registrations.iter().copied());
    report
}

pub(super) fn runtime_modules_for_target_with_plugin_and_feature_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().cloned().collect::<Vec<_>>();
    let feature_registrations = feature_registrations
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let active_registrations = active_plugin_registration_refs(target, registrations.iter());
    let feature_reports = feature_reports_for_plugin_and_feature_registration_reports(
        target,
        &manifest,
        &registrations,
        &feature_registrations,
    );
    let inputs = registration_inputs_for_plugin_and_feature_reports(
        &active_registrations,
        feature_reports.active_feature_registrations(),
    );
    let mut report = runtime_modules_for_target_with_registration_inputs_for_manifest(
        target, &manifest, &inputs,
    );
    feature_reports.extend_load_report_diagnostics(&mut report);
    extend_asset_importer_errors(&mut report, &inputs);
    report.runtime_plugin_availability = target_manifest_availability_for_registration_reports(
        target,
        &manifest,
        registrations.iter(),
    );
    report
}

fn active_plugin_registration_refs<'a>(
    target: RuntimeTargetMode,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> Vec<&'a RuntimePluginRegistrationReport> {
    registrations
        .into_iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration.project_selection.supports_target(target)
        })
        .collect()
}

fn extend_asset_importer_errors(
    report: &mut RuntimeModuleLoadReport,
    inputs: &super::registration_inputs::RuntimeModuleRegistrationInputs,
) {
    report.extend_diagnostics(
        inputs
            .asset_importer_errors()
            .iter()
            .cloned()
            .map(RuntimeModuleLoadDiagnostic::AssetImporter),
    );
}
