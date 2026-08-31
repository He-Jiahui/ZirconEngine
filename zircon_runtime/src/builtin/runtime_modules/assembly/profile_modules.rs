use crate::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use crate::plugin::{
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
    RuntimeProfileDescriptor,
};

use super::super::availability::runtime_profile_availability;
use super::super::load_report::RuntimeModuleLoadReport;
use super::registration_inputs::RuntimeModuleRegistrationInputs;
use super::registration_reports::runtime_modules_for_profile_manifest_with_plugin_registration_reports;
use super::target_modules::runtime_modules_for_profile_with_registration_inputs_for_manifest_and_availability;

pub(super) fn runtime_modules_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let manifest = profile.project_manifest();
    let availability = runtime_profile_availability(&profile);
    runtime_modules_for_profile_with_registration_inputs_for_manifest_and_availability(
        &profile,
        &manifest,
        &RuntimeModuleRegistrationInputs::empty(),
        availability,
    )
}

pub(super) fn runtime_modules_for_runtime_profile_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let manifest = profile.project_manifest();
    runtime_modules_for_profile_descriptor_manifest_with_plugin_registration_reports(
        &profile,
        &manifest,
        registrations,
    )
}

pub(super) fn runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    runtime_modules_for_profile_descriptor_manifest_with_plugin_registration_reports(
        &profile,
        manifest,
        registrations,
    )
}

fn runtime_modules_for_profile_descriptor_manifest_with_plugin_registration_reports<'a>(
    profile: &RuntimeProfileDescriptor,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_profile_manifest_with_plugin_registration_reports(
        profile,
        profile.target_mode,
        manifest,
        registrations,
    )
}

pub(super) fn runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports<
    'a,
>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let manifest = profile.project_manifest();
    runtime_modules_for_profile_descriptor_manifest_with_plugin_and_feature_registration_reports(
        &profile,
        &manifest,
        registrations,
        feature_registrations,
    )
}

pub(super) fn runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports<
    'a,
>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    runtime_modules_for_profile_descriptor_manifest_with_plugin_and_feature_registration_reports(
        &profile,
        manifest,
        registrations,
        feature_registrations,
    )
}

fn runtime_modules_for_profile_descriptor_manifest_with_plugin_and_feature_registration_reports<
    'a,
>(
    profile: &RuntimeProfileDescriptor,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    super::registration_reports::runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        profile.target_mode,
        Some(manifest),
        registrations,
        feature_registrations,
        Some(profile),
    )
}

#[cfg(test)]
#[path = "profile_modules/direct_iterator_forwarding_tests.rs"]
mod direct_iterator_forwarding_tests;
