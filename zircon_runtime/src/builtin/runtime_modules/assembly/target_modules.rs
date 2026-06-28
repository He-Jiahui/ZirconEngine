use std::collections::HashSet;

use crate::plugin::ProjectPluginManifest;

use super::super::availability::target_manifest_availability;
use super::super::core_modules::runtime_core_modules_for_target_with_render_features;
use super::super::ids::RuntimeTargetMode;
use super::super::load_report::RuntimeModuleLoadReport;
use super::super::manifest::manifest_with_mode_baseline;
use super::super::plugin_modules::{
    builtin_runtime_domain_is_available, builtin_runtime_domain_message,
    linked_plugin_is_available, module_for_plugin,
};
use super::registration_inputs::RuntimeModuleRegistrationInputs;

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
    let mut report =
        RuntimeModuleLoadReport::new(runtime_core_modules_for_target_with_render_features(
            target,
            inputs.asset_importers(),
            inputs.render_features(),
            inputs.shading_models(),
            inputs.render_pass_executors(),
            inputs.runtime_prepare_collectors(),
            inputs.hybrid_gi_runtime_providers(),
            inputs.solari_runtime_providers(),
            inputs.virtual_geometry_runtime_providers(),
        ));
    report.runtime_plugin_availability =
        target_manifest_availability(target, manifest, linked_plugin_ids.iter());

    for selection in manifest.enabled_for_target(target) {
        let Some(runtime_id) = selection.runtime_id() else {
            let reason = format!("plugin {} has no known runtime id", selection.id);
            if selection.required {
                report.errors.push(format!(
                    "required runtime plugin {} is unavailable: {}",
                    selection.id, reason
                ));
            } else {
                report.warnings.push(reason);
            }
            continue;
        };
        if builtin_runtime_domain_is_available(runtime_id) {
            report
                .warnings
                .push(builtin_runtime_domain_message(runtime_id.key()));
            continue;
        }
        if linked_plugin_is_available(selection, runtime_id, &linked_plugin_ids) {
            continue;
        }
        let warning_start = report.warnings.len();
        if let Some(module) = module_for_plugin(runtime_id, &mut report.warnings) {
            report.modules.push(module);
            continue;
        }
        if selection.required {
            let reason = report.warnings[warning_start..]
                .last()
                .cloned()
                .unwrap_or_else(|| format!("plugin {} is unavailable", runtime_id.label()));
            let message = format!(
                "required runtime plugin {} is unavailable: {}",
                runtime_id.label(),
                reason.clone()
            );
            report.push_required_missing(runtime_id, reason);
            report.errors.push(message);
        }
    }
    report
}
