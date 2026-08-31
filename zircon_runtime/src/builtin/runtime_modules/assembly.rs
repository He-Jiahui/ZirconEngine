use crate::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

pub(super) mod compiled_plan;
mod extension_inputs;
mod feature_reports;
mod profile_modules;
pub(super) mod profile_selection;
mod registration_inputs;
mod registration_reports;
mod target_modules;

use super::load_report::RuntimeModuleLoadReport;
use super::{
    finish_runtime_module_composition, RuntimeModuleCompositionCompiler,
    RuntimeModuleCompositionIdentitySeed, RuntimeModuleCompositionResult,
};
use crate::core::framework::platform::RuntimeTargetMode;
use profile_modules::{
    runtime_modules_for_runtime_profile as runtime_modules_for_runtime_profile_impl,
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports as runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports_impl,
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports as runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports_impl,
    runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports as runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports_impl,
    runtime_modules_for_runtime_profile_with_plugin_registration_reports as runtime_modules_for_runtime_profile_with_plugin_registration_reports_impl,
};
use registration_inputs::RuntimeModuleRegistrationInputs;
use registration_reports::{
    runtime_modules_for_target_with_plugin_and_feature_registration_reports as runtime_modules_for_target_with_plugin_and_feature_registration_reports_impl,
    runtime_modules_for_target_with_plugin_registration_reports as runtime_modules_for_target_with_plugin_registration_reports_impl,
};
use target_modules::runtime_modules_for_target_with_registration_inputs;

/// Materializes runtime modules and availability from one immutable plugin plan generation.
pub fn runtime_modules_for_compiled_project_plugin_plan(
    plan: &crate::plugin::CompiledProjectPluginPlan,
) -> RuntimeModuleCompositionResult {
    RuntimeModuleCompositionCompiler::new(plan).compile()
}

/// Materializes a profile-constrained module report from one immutable plugin plan generation.
/// A plan compiled for another target is rejected before any module is returned.
pub fn runtime_modules_for_runtime_profile_compiled_project_plugin_plan(
    profile_id: RuntimeProfileId,
    plan: &crate::plugin::CompiledProjectPluginPlan,
) -> RuntimeModuleCompositionResult {
    RuntimeModuleCompositionCompiler::new(plan)
        .for_runtime_profile(profile_id)
        .compile()
}

pub fn runtime_modules_for_target(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
) -> RuntimeModuleCompositionResult {
    runtime_modules_for_target_with_linked_plugins(
        target,
        manifest_override,
        std::iter::empty::<String>(),
    )
}

pub fn runtime_modules_for_target_with_linked_plugins(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
) -> RuntimeModuleCompositionResult {
    let inputs = RuntimeModuleRegistrationInputs::from_linked_plugin_ids(linked_plugin_ids);
    finish_legacy_report(
        runtime_modules_for_target_with_registration_inputs(target, manifest_override, &inputs),
        target,
        None,
    )
}

pub fn runtime_modules_for_target_with_plugin_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleCompositionResult {
    finish_legacy_report(
        runtime_modules_for_target_with_plugin_registration_reports_impl(
            target,
            manifest_override,
            registrations,
        ),
        target,
        None,
    )
}

pub fn runtime_modules_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> RuntimeModuleCompositionResult {
    let target = crate::plugin::RuntimeProfileDescriptor::for_id(profile_id).target_mode;
    finish_legacy_report(
        runtime_modules_for_runtime_profile_impl(profile_id),
        target,
        Some(profile_id),
    )
}

pub fn runtime_modules_for_runtime_profile_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleCompositionResult {
    let target = crate::plugin::RuntimeProfileDescriptor::for_id(profile_id).target_mode;
    finish_legacy_report(
        runtime_modules_for_runtime_profile_with_plugin_registration_reports_impl(
            profile_id,
            registrations,
        ),
        target,
        Some(profile_id),
    )
}

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleCompositionResult {
    let target = crate::plugin::RuntimeProfileDescriptor::for_id(profile_id).target_mode;
    finish_legacy_report(
        runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports_impl(
            profile_id,
            manifest,
            registrations,
        ),
        target,
        Some(profile_id),
    )
}

pub fn runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleCompositionResult {
    let target = crate::plugin::RuntimeProfileDescriptor::for_id(profile_id).target_mode;
    finish_legacy_report(
        runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports_impl(
            profile_id,
            registrations,
            feature_registrations,
        ),
        target,
        Some(profile_id),
    )
}

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports<
    'a,
>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleCompositionResult {
    let target = crate::plugin::RuntimeProfileDescriptor::for_id(profile_id).target_mode;
    finish_legacy_report(
        runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports_impl(
            profile_id,
            manifest,
            registrations,
            feature_registrations,
        ),
        target,
        Some(profile_id),
    )
}

pub fn runtime_modules_for_target_with_plugin_and_feature_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleCompositionResult {
    finish_legacy_report(
        runtime_modules_for_target_with_plugin_and_feature_registration_reports_impl(
            target,
            manifest_override,
            registrations,
            feature_registrations,
            None,
        ),
        target,
        None,
    )
}

fn finish_legacy_report(
    report: RuntimeModuleLoadReport,
    target: RuntimeTargetMode,
    runtime_profile: Option<RuntimeProfileId>,
) -> RuntimeModuleCompositionResult {
    finish_runtime_module_composition(
        report,
        RuntimeModuleCompositionIdentitySeed::legacy(target, runtime_profile),
    )
}
