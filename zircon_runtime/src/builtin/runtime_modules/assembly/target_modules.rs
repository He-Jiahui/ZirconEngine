use std::collections::HashSet;

use crate::builtin::RuntimePluginId;
use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::RuntimePluginAvailabilityCategory;

use super::super::availability::target_manifest_availability;
use super::super::core_modules::{
    runtime_core_modules_for_target_with_render_features, sort_runtime_modules_by_descriptor_order,
};
use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};
use super::super::manifest::manifest_with_mode_baseline;
use super::super::plugin_modules::module_for_plugin;
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
    let linked_plugin_ids = inputs
        .linked_plugin_ids()
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    #[cfg(feature = "graphics")]
    let core_modules = runtime_core_modules_for_target_with_render_features(
        target,
        inputs.asset_importers(),
        inputs.render_features(),
        inputs.geometry_sources(),
        inputs.shading_models(),
        inputs.render_pass_executors(),
        inputs.runtime_prepare_collectors(),
        inputs.hybrid_gi_runtime_providers(),
        inputs.solari_runtime_providers(),
        inputs.virtual_geometry_runtime_providers(),
    );
    #[cfg(not(feature = "graphics"))]
    let core_modules =
        runtime_core_modules_for_target_with_render_features(target, inputs.asset_importers());
    let core_modules = match core_modules {
        Ok(modules) => modules,
        Err(error) => return RuntimeModuleLoadReport::from_core_error(error),
    };
    let mut report = RuntimeModuleLoadReport::new(core_modules);
    report.runtime_plugin_availability =
        target_manifest_availability(target, manifest, linked_plugin_ids.iter());

    for selection in manifest.enabled_for_target(target) {
        let Some(runtime_id) = RuntimePluginId::parse_key(&selection.id) else {
            report.push_diagnostic(RuntimeModuleLoadDiagnostic::UnknownPlugin {
                id: selection.id.clone(),
                required: selection.required,
            });
            continue;
        };
        if report
            .runtime_plugin_availability
            .contains(RuntimePluginAvailabilityCategory::Linked, runtime_id)
            || report
                .runtime_plugin_availability
                .contains(RuntimePluginAvailabilityCategory::NativeDynamic, runtime_id)
        {
            continue;
        }
        if let Some(module) = module_for_plugin(runtime_id) {
            report.modules.push(module);
        }
    }

    match sort_runtime_modules_by_descriptor_order(std::mem::take(&mut report.modules)) {
        Ok(modules) => report.modules = modules,
        Err(error) => report.push_diagnostic(RuntimeModuleLoadDiagnostic::Core(error)),
    }
    report
}
