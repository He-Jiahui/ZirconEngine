use std::sync::Arc;

use crate::engine_module::EngineModule;
use crate::plugin::{
    ProjectPluginManifest, RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport, RuntimeProfileDescriptor, RuntimeProfileId,
};

mod registration_inputs;
mod target_modules;

use super::availability::{
    runtime_profile_availability, runtime_profile_manifest_availability,
    target_manifest_availability_for_registration_reports,
};
use super::core_modules::minimal_profile_runtime_modules;
use super::manifest::manifest_with_mode_baseline;
use super::{RuntimeModuleLoadReport, RuntimeTargetMode};
use registration_inputs::{
    active_feature_registration_refs, active_plugin_registration_refs,
    registration_inputs_for_plugin_and_feature_reports, registration_inputs_for_plugin_reports,
    RuntimeModuleRegistrationInputs,
};
use target_modules::{
    runtime_modules_for_target_with_registration_inputs,
    runtime_modules_for_target_with_registration_inputs_for_manifest,
};

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
    let registrations = active_plugin_registration_refs(target, registrations);
    let inputs = registration_inputs_for_plugin_reports(&registrations);
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let mut report = runtime_modules_for_target_with_registration_inputs_for_manifest(
        target, &manifest, &inputs,
    );
    report
        .errors
        .extend(inputs.asset_importer_errors().iter().cloned());
    report.runtime_plugin_availability = target_manifest_availability_for_registration_reports(
        target,
        &manifest,
        registrations.iter().copied(),
    );
    report
}

pub fn runtime_modules_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> RuntimeModuleLoadReport {
    if profile_id == RuntimeProfileId::Minimal {
        let profile = RuntimeProfileDescriptor::for_id(profile_id);
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_availability(&profile));
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

pub fn runtime_modules_for_runtime_profile_with_plugin_registration_reports<'a>(
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

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().collect::<Vec<_>>();
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    if profile_id == RuntimeProfileId::Minimal {
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_manifest_availability(
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

pub fn runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports<'a>(
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

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports<
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
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_manifest_availability(
                &profile,
                manifest,
                registrations.iter(),
            ));
    }

    let mut report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        profile.target_mode,
        Some(manifest),
        registrations.iter(),
        feature_registrations.iter(),
    );
    report.runtime_plugin_availability =
        runtime_profile_manifest_availability(&profile, manifest, registrations.iter());
    report
}

fn runtime_modules_for_profile_manifest_with_plugin_registration_reports<'a>(
    profile: &RuntimeProfileDescriptor,
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = active_plugin_registration_refs(target, registrations);
    let inputs = registration_inputs_for_plugin_reports(&registrations);
    let mut report =
        runtime_modules_for_target_with_registration_inputs_for_manifest(target, manifest, &inputs);
    report
        .errors
        .extend(inputs.asset_importer_errors().iter().cloned());
    report.runtime_plugin_availability =
        runtime_profile_manifest_availability(profile, manifest, registrations.iter().copied());
    report
}

pub fn runtime_modules_for_target_with_plugin_and_feature_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().cloned().collect::<Vec<_>>();
    let feature_registrations = feature_registrations
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.clone(),
        feature_registrations.clone(),
    );
    let active_registrations = active_plugin_registration_refs(target, registrations.iter());
    let feature_report = catalog.feature_dependency_report(&manifest, target);
    let active_feature_registrations =
        active_feature_registration_refs(&feature_registrations, &feature_report);
    let inputs = registration_inputs_for_plugin_and_feature_reports(
        &active_registrations,
        &active_feature_registrations,
    );
    let mut report = runtime_modules_for_target_with_registration_inputs_for_manifest(
        target, &manifest, &inputs,
    );
    for blocked in feature_report.blocked_features {
        if blocked.required {
            report.errors.push(blocked.to_diagnostic());
        } else {
            report.warnings.push(blocked.to_diagnostic());
        }
    }
    report.errors.extend(feature_report.diagnostics);
    report
        .errors
        .extend(inputs.asset_importer_errors().iter().cloned());
    report.runtime_plugin_availability = target_manifest_availability_for_registration_reports(
        target,
        &manifest,
        registrations.iter(),
    );
    report
}
