use crate::plugin::{
    ProjectPluginManifest, RuntimePluginAvailabilityReport, RuntimePluginDescriptor,
    RuntimePluginRegistrationReport, RuntimeProfileDescriptor, RuntimeProfileId,
};

use super::RuntimeTargetMode;

pub(super) fn runtime_profile_availability(
    profile: &RuntimeProfileDescriptor,
) -> RuntimePluginAvailabilityReport {
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_with_providers(
        descriptors.iter(),
        std::iter::empty::<String>(),
        std::iter::empty::<String>(),
    )
}

pub(super) fn runtime_profile_manifest_availability<'a>(
    profile: &RuntimeProfileDescriptor,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimePluginAvailabilityReport {
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_for_manifest_and_registration_reports(
        descriptors.iter(),
        manifest,
        registrations,
    )
}

pub(super) fn target_manifest_availability<'a>(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    linked_plugin_ids: impl IntoIterator<Item = &'a String>,
) -> RuntimePluginAvailabilityReport {
    let profile = RuntimeProfileDescriptor::new(
        runtime_profile_id_for_target_availability(target),
        "target module selection",
        target,
    );
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_for_manifest_with_providers(
        descriptors.iter(),
        manifest,
        linked_plugin_ids,
        std::iter::empty::<String>(),
    )
}

pub(super) fn target_manifest_availability_for_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimePluginAvailabilityReport {
    let profile = RuntimeProfileDescriptor::new(
        runtime_profile_id_for_target_availability(target),
        "target module selection",
        target,
    );
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_for_manifest_and_registration_reports(
        descriptors.iter(),
        manifest,
        registrations,
    )
}

fn runtime_profile_id_for_target_availability(target: RuntimeTargetMode) -> RuntimeProfileId {
    match target {
        RuntimeTargetMode::ClientRuntime => RuntimeProfileId::Client2d,
        RuntimeTargetMode::ServerRuntime => RuntimeProfileId::Server,
        RuntimeTargetMode::EditorHost => RuntimeProfileId::Editor,
    }
}

fn runtime_plugin_descriptors() -> Vec<RuntimePluginDescriptor> {
    RuntimePluginDescriptor::builtin_catalog()
}
