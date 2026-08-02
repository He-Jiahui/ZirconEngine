use std::sync::Arc;

use crate::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use crate::engine_module::EngineModule;
use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

mod extension_inputs;
mod feature_reports;
mod profile_modules;
pub(super) mod profile_selection;
mod registration_inputs;
mod registration_reports;
mod target_modules;

use super::load_report::RuntimeModuleLoadReport;
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

pub fn builtin_runtime_modules() -> Vec<Arc<dyn EngineModule>> {
    runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, None).modules
}

pub fn runtime_modules_for_target(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
) -> RuntimeModuleLoadReport {
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
) -> RuntimeModuleLoadReport {
    let inputs = RuntimeModuleRegistrationInputs::from_linked_plugin_ids(linked_plugin_ids);
    runtime_modules_for_target_with_registration_inputs(target, manifest_override, &inputs)
}

pub fn runtime_modules_for_target_with_plugin_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_target_with_plugin_registration_reports_impl(
        target,
        manifest_override,
        registrations,
    )
}

pub fn runtime_modules_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_runtime_profile_impl(profile_id)
}

pub fn runtime_modules_for_runtime_profile_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_runtime_profile_with_plugin_registration_reports_impl(
        profile_id,
        registrations,
    )
}

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports_impl(
        profile_id,
        manifest,
        registrations,
    )
}

pub fn runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports_impl(
        profile_id,
        registrations,
        feature_registrations,
    )
}

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports<
    'a,
>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports_impl(
        profile_id,
        manifest,
        registrations,
        feature_registrations,
    )
}

pub fn runtime_modules_for_target_with_plugin_and_feature_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_target_with_plugin_and_feature_registration_reports_impl(
        target,
        manifest_override,
        registrations,
        feature_registrations,
        None,
    )
}
