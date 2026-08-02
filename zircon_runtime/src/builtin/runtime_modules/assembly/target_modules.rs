use crate::builtin::RuntimePluginId;
use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityReport, RuntimeProfileDescriptor,
};

use super::super::availability::target_manifest_availability;
use super::super::core_modules::{
    runtime_core_module_candidates_for_target_with_render_features,
    sort_runtime_modules_by_descriptor_order_with_cache,
};
use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};
use super::super::manifest::manifest_with_mode_baseline;
use super::super::plugin_modules::module_for_plugin;
use super::profile_selection::select_runtime_profile_builtin_module_descriptors;
use super::registration_inputs::RuntimeModuleRegistrationInputs;
use crate::core::framework::platform::RuntimeTargetMode;

pub(super) fn runtime_modules_for_target_with_registration_inputs(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    inputs: &RuntimeModuleRegistrationInputs,
) -> RuntimeModuleLoadReport {
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    runtime_modules_for_target_with_registration_inputs_for_manifest(target, &manifest, inputs)
}

pub(super) fn runtime_modules_for_target_with_registration_inputs_for_manifest(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    inputs: &RuntimeModuleRegistrationInputs,
) -> RuntimeModuleLoadReport {
    let availability = target_manifest_availability(target, manifest, inputs.linked_plugin_ids());
    runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability(
        target,
        manifest,
        inputs,
        availability,
    )
}

pub(super) fn runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    inputs: &RuntimeModuleRegistrationInputs,
    availability: RuntimePluginAvailabilityReport,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_manifest_and_availability(target, manifest, inputs, availability, None)
}

pub(super) fn runtime_modules_for_profile_with_registration_inputs_for_manifest_and_availability(
    profile: &RuntimeProfileDescriptor,
    manifest: &ProjectPluginManifest,
    inputs: &RuntimeModuleRegistrationInputs,
    availability: RuntimePluginAvailabilityReport,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_manifest_and_availability(
        profile.target_mode,
        manifest,
        inputs,
        availability,
        Some(profile),
    )
}

fn runtime_modules_for_manifest_and_availability(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    inputs: &RuntimeModuleRegistrationInputs,
    availability: RuntimePluginAvailabilityReport,
    profile: Option<&RuntimeProfileDescriptor>,
) -> RuntimeModuleLoadReport {
    #[cfg(feature = "graphics")]
    let core_modules = runtime_core_module_candidates_for_target_with_render_features(
        target,
        inputs.asset_importers(),
        inputs.render_features(),
        inputs.geometry_sources(),
        inputs.shading_models(),
        inputs.plugin_shader_module_sources(),
        inputs.render_pass_executors(),
        inputs.runtime_prepare_collectors(),
        inputs.hybrid_gi_runtime_providers(),
        inputs.solari_runtime_providers(),
        inputs.virtual_geometry_runtime_providers(),
    );
    #[cfg(not(feature = "graphics"))]
    let core_modules = runtime_core_module_candidates_for_target_with_render_features(
        target,
        inputs.asset_importers(),
    );
    let (core_modules, core_descriptors) = if let Some(profile) = profile {
        match select_runtime_profile_builtin_module_descriptors(profile, core_modules) {
            Ok(selection) => (selection.modules, selection.descriptors_by_name),
            Err(error) => {
                return RuntimeModuleLoadReport::from_core_error(error)
                    .with_runtime_plugin_availability(availability);
            }
        }
    } else {
        (core_modules, Default::default())
    };
    let mut report =
        RuntimeModuleLoadReport::new(core_modules).with_runtime_plugin_availability(availability);

    for selection in manifest.enabled_for_target(target) {
        let Some(runtime_id) = RuntimePluginId::parse_key(&selection.id) else {
            report.push_diagnostic(RuntimeModuleLoadDiagnostic::UnknownPlugin {
                id: selection.id.clone(),
                required: selection.required,
            });
            continue;
        };
        if report.runtime_plugin_availability.contains(
            RuntimePluginAvailabilityCategory::Linked,
            runtime_id.clone(),
        ) || report.runtime_plugin_availability.contains(
            RuntimePluginAvailabilityCategory::NativeDynamic,
            runtime_id.clone(),
        ) {
            continue;
        }
        if let Some(module) = module_for_plugin(runtime_id) {
            report.modules.push(module);
        }
    }

    match sort_runtime_modules_by_descriptor_order_with_cache(
        std::mem::take(&mut report.modules),
        core_descriptors,
    ) {
        Ok(modules) => report.modules = modules,
        Err(error) => report.push_diagnostic(RuntimeModuleLoadDiagnostic::Core(error)),
    }
    report
}
