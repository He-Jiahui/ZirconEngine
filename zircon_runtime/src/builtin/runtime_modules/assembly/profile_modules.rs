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
use super::target_modules::runtime_modules_for_target_with_registration_inputs_for_manifest;

pub(super) fn runtime_modules_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> RuntimeModuleLoadReport {
    if profile_id == RuntimeProfileId::Minimal {
        let profile = RuntimeProfileDescriptor::for_id(profile_id);
        return minimal_profile_runtime_modules_report(runtime_profile_availability(&profile));
    }

    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let manifest = profile.project_manifest();
    runtime_modules_for_target_with_registration_inputs_for_manifest(
        profile.target_mode,
        &manifest,
        &RuntimeModuleRegistrationInputs::empty(),
    )
    .with_runtime_plugin_availability(runtime_profile_availability(&profile))
}

pub(super) fn runtime_modules_for_runtime_profile_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports(
        profile_id,
        &profile.project_manifest(),
        registrations,
    )
}

pub(super) fn runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().collect::<Vec<_>>();
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    if profile_id == RuntimeProfileId::Minimal {
        return minimal_profile_runtime_modules_report(runtime_profile_manifest_availability(
            &profile,
            manifest,
            registrations.iter().copied(),
        ));
    }

    runtime_modules_for_profile_manifest_with_plugin_registration_reports(
        &profile,
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
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports(
        profile_id,
        &profile.project_manifest(),
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
    let registrations = registrations.into_iter().cloned().collect::<Vec<_>>();
    let feature_registrations = feature_registrations
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    if profile_id == RuntimeProfileId::Minimal {
        return minimal_profile_runtime_modules_report(runtime_profile_manifest_availability(
            &profile,
            manifest,
            registrations.iter(),
        ));
    }

    let mut report = super::registration_reports::runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        profile.target_mode,
        Some(manifest),
        registrations.iter(),
        feature_registrations.iter(),
    );
    report.runtime_plugin_availability =
        runtime_profile_manifest_availability(&profile, manifest, registrations.iter());
    report
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
