use std::collections::HashSet;

use crate::builtin::RuntimePluginId;
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
use super::target_modules::{
    runtime_modules_for_profile_with_registration_inputs_for_manifest_and_availability,
    runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability,
};
use crate::core::framework::platform::RuntimeTargetMode;

pub(super) fn runtime_modules_for_target_with_plugin_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let registrations = active_plugin_registration_refs(target, &manifest, registrations);
    let inputs = registration_inputs_for_plugin_reports(&registrations);
    let availability = target_manifest_availability_for_registration_reports(
        target,
        &manifest,
        registrations.iter().copied(),
    );
    let mut report =
        runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability(
            target,
            &manifest,
            &inputs,
            availability,
        );
    extend_asset_importer_errors(&mut report, &inputs);
    report
}

pub(super) fn runtime_modules_for_profile_manifest_with_plugin_registration_reports<'a>(
    profile: &RuntimeProfileDescriptor,
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = active_plugin_registration_refs(target, manifest, registrations);
    let inputs = registration_inputs_for_plugin_reports(&registrations);
    let availability =
        runtime_profile_manifest_availability(profile, manifest, registrations.iter().copied());
    let mut report =
        runtime_modules_for_profile_with_registration_inputs_for_manifest_and_availability(
            profile,
            manifest,
            &inputs,
            availability,
        );
    extend_asset_importer_errors(&mut report, &inputs);
    report
}

pub(super) fn runtime_modules_for_target_with_plugin_and_feature_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
    availability_profile: Option<&RuntimeProfileDescriptor>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().cloned().collect::<Vec<_>>();
    let feature_registrations = feature_registrations
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let active_registrations =
        active_plugin_registration_refs(target, &manifest, registrations.iter());
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
    let availability = if let Some(profile) = availability_profile {
        runtime_profile_manifest_availability(
            profile,
            manifest_override.unwrap_or(&manifest),
            registrations.iter(),
        )
    } else {
        target_manifest_availability_for_registration_reports(
            target,
            &manifest,
            registrations.iter(),
        )
    };
    let mut report = if let Some(profile) = availability_profile {
        runtime_modules_for_profile_with_registration_inputs_for_manifest_and_availability(
            profile,
            &manifest,
            &inputs,
            availability,
        )
    } else {
        runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability(
            target,
            &manifest,
            &inputs,
            availability,
        )
    };
    feature_reports.extend_load_report_diagnostics(&mut report);
    extend_asset_importer_errors(&mut report, &inputs);
    report
}

fn active_plugin_registration_refs<'a>(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> Vec<&'a RuntimePluginRegistrationReport> {
    let enabled_plugin_ids = manifest
        .enabled_for_target(target)
        .filter_map(|selection| RuntimePluginId::parse_key(&selection.id))
        .collect::<HashSet<_>>();
    registrations
        .into_iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration.project_selection.supports_target(target)
                && RuntimePluginId::parse_key(&registration.project_selection.id)
                    .is_some_and(|plugin_id| enabled_plugin_ids.contains(&plugin_id))
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
