use std::collections::HashSet;

use crate::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use crate::plugin::{
    CompiledProjectPluginPlan, RuntimePluginAvailabilityReport, RuntimePluginDescriptor,
    RuntimePluginRegistrationReport, RuntimeProfileDescriptor,
};

#[cfg(test)]
use std::cell::Cell;

use crate::core::framework::platform::RuntimeTargetMode;

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

pub(super) fn runtime_profile_compiled_plan_availability(
    profile: &RuntimeProfileDescriptor,
    plan: &CompiledProjectPluginPlan,
) -> RuntimePluginAvailabilityReport {
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_for_manifest_with_providers(
        descriptors.iter(),
        plan.completed_manifest(),
        plan.linked_provider_package_ids(),
        plan.native_dynamic_provider_package_ids(),
    )
}

pub(super) fn target_compiled_plan_availability(
    plan: &CompiledProjectPluginPlan,
) -> RuntimePluginAvailabilityReport {
    let profile = RuntimeProfileDescriptor::new(
        runtime_profile_id_for_target_availability(plan.target_mode()),
        "compiled project plugin plan module selection",
        plan.target_mode(),
    );
    runtime_profile_compiled_plan_availability(&profile, plan)
}

pub(super) fn target_manifest_availability(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    linked_plugin_ids: &HashSet<String>,
) -> RuntimePluginAvailabilityReport {
    let profile = RuntimeProfileDescriptor::new(
        runtime_profile_id_for_target_availability(target),
        "target module selection",
        target,
    );
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_for_manifest_with_linked_membership(
        descriptors.iter(),
        manifest,
        linked_plugin_ids,
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
    #[cfg(test)]
    PROJECTION_BUILD_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    RuntimePluginDescriptor::builtin_catalog()
}

#[cfg(test)]
thread_local! {
    static PROJECTION_BUILD_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(super) fn reset_projection_build_count() {
    PROJECTION_BUILD_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(super) fn projection_build_count() -> usize {
    PROJECTION_BUILD_COUNT.with(Cell::get)
}
