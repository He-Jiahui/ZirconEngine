use crate::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

#[path = "assembly/compiled_plan.rs"]
pub(super) mod compiled_plan;
#[path = "assembly/extension_inputs.rs"]
mod extension_inputs;
#[path = "assembly/feature_reports.rs"]
mod feature_reports;
#[path = "assembly/profile_modules.rs"]
mod profile_modules;
#[path = "assembly/profile_selection.rs"]
pub(super) mod profile_selection;
#[path = "assembly/registration_inputs.rs"]
mod registration_inputs;
#[path = "assembly/registration_reports.rs"]
mod registration_reports;
#[path = "assembly/target_modules.rs"]
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
/// Requires a plan admitted for this runtime profile and its preset maturity floor.
/// Target-only or differently admitted plans are rejected;
/// this wrapper cannot re-admit an immutable plan. For an explicit maturity override use
/// `RuntimeModuleCompositionCompiler::new(plan)` with its matching applied policy.
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
