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
    let registrations = registrations.into_iter().collect::<Vec<_>>();
    runtime_modules_for_profile_manifest_with_plugin_registration_reports(
        profile,
        profile.target_mode,
        manifest,
        registrations.iter().copied(),
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
    let registrations = registrations.into_iter().collect::<Vec<_>>();
    let feature_registrations = feature_registrations.into_iter().collect::<Vec<_>>();
    super::registration_reports::runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        profile.target_mode,
        Some(manifest),
        registrations.iter().copied(),
        feature_registrations.iter().copied(),
        Some(profile),
    )
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn profile_feature_assembly_collects_borrowed_registration_refs() {
        let source = include_str!("profile_modules.rs");
        let start = source
            .find("fn runtime_modules_for_profile_descriptor_manifest_with_plugin_and_feature_registration_reports")
            .expect("profile feature assembly owner");
        let compact = source[start..].split_whitespace().collect::<String>();

        assert!(
            !compact.contains(".into_iter().cloned().collect::<Vec<_>>()"),
            "profile assembly must not deep-clone reports before the target assembly owner"
        );
    }
}
