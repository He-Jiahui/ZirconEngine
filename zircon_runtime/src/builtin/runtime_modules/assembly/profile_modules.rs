use crate::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use crate::plugin::{
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
    RuntimeProfileDescriptor,
};

use super::super::availability::{
    runtime_profile_availability, runtime_profile_manifest_availability,
};
use super::super::core_modules::minimal_profile_runtime_modules;
use super::super::load_report::RuntimeModuleLoadReport;
use super::registration_inputs::RuntimeModuleRegistrationInputs;
use super::registration_reports::runtime_modules_for_profile_manifest_with_plugin_registration_reports;
use super::target_modules::runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability;

pub(super) fn runtime_modules_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> RuntimeModuleLoadReport {
    if profile_id == RuntimeProfileId::Minimal {
        let profile = RuntimeProfileDescriptor::for_id(profile_id);
        return minimal_profile_runtime_modules_report(runtime_profile_availability(&profile));
    }

    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let manifest = profile.project_manifest();
    let availability = runtime_profile_availability(&profile);
    runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability(
        profile.target_mode,
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
    if profile.id == RuntimeProfileId::Minimal {
        return minimal_profile_runtime_modules_report(runtime_profile_manifest_availability(
            profile,
            manifest,
            registrations.iter().copied(),
        ));
    }

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
    if profile.id == RuntimeProfileId::Minimal {
        return minimal_profile_runtime_modules_report(runtime_profile_manifest_availability(
            profile,
            manifest,
            registrations.iter().copied(),
        ));
    }

    super::registration_reports::runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        profile.target_mode,
        Some(manifest),
        registrations.iter().copied(),
        feature_registrations.iter().copied(),
        Some(profile),
    )
}

fn minimal_profile_runtime_modules_report(
    availability: crate::plugin::RuntimePluginAvailabilityReport,
) -> RuntimeModuleLoadReport {
    match minimal_profile_runtime_modules() {
        Ok(modules) => {
            RuntimeModuleLoadReport::new(modules).with_runtime_plugin_availability(availability)
        }
        Err(error) => RuntimeModuleLoadReport::from_core_error(error)
            .with_runtime_plugin_availability(availability),
    }
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn profile_feature_assembly_collects_borrowed_registration_refs() {
        let source = include_str!("profile_modules.rs");
        let start = source
            .find("fn runtime_modules_for_profile_descriptor_manifest_with_plugin_and_feature_registration_reports")
            .expect("profile feature assembly owner");
        let end = source[start..]
            .find("fn minimal_profile_runtime_modules_report")
            .map(|offset| start + offset)
            .expect("profile feature assembly owner end");
        let compact = source[start..end].split_whitespace().collect::<String>();

        assert!(
            !compact.contains(".into_iter().cloned().collect::<Vec<_>>()"),
            "profile assembly must not deep-clone reports before the target assembly owner"
        );
    }
}
