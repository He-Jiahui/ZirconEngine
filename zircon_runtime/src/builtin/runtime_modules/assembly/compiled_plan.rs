use std::collections::HashMap;

use crate::core::framework::project::RuntimeProfileId;
use crate::core::CoreError;
use crate::plugin::{CompiledProjectPluginPlan, RuntimeProfileDescriptor};

use super::super::availability::{
    runtime_profile_compiled_plan_availability, target_compiled_plan_availability,
};
use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};
use super::super::plugin_modules::descriptor_backed_module;
use super::registration_inputs::registration_inputs_for_extension_registry;
use super::target_modules::{
    runtime_modules_for_profile_with_registration_inputs_for_manifest_and_availability,
    runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability,
};

pub(in crate::builtin::runtime_modules) fn assemble_compiled_project_plugin_plan_candidate(
    plan: &CompiledProjectPluginPlan,
    profile_id: Option<RuntimeProfileId>,
) -> RuntimeModuleLoadReport {
    let profile = profile_id.map(RuntimeProfileDescriptor::for_id);
    if let Some(profile) = profile.as_ref() {
        if profile.target_mode != plan.target_mode() {
            return RuntimeModuleLoadReport::from_core_error(CoreError::Initialization(
                "runtime plugin plan target mismatch".to_string(),
                format!(
                    "profile {:?} requires {:?}, but compiled plan targets {:?}",
                    profile.id,
                    profile.target_mode,
                    plan.target_mode()
                ),
            ));
        }
    }

    let manifest = plan.completed_manifest();
    let inputs = registration_inputs_for_extension_registry(&plan.runtime_extensions().registry);
    let availability = if let Some(profile) = profile.as_ref() {
        runtime_profile_compiled_plan_availability(profile, plan)
    } else {
        target_compiled_plan_availability(plan)
    };
    let mut report = if let Some(profile) = profile.as_ref() {
        runtime_modules_for_profile_with_registration_inputs_for_manifest_and_availability(
            profile,
            manifest,
            &inputs,
            availability,
        )
    } else {
        runtime_modules_for_target_with_registration_inputs_for_manifest_and_availability(
            plan.target_mode(),
            manifest,
            &inputs,
            availability,
        )
    };

    append_plan_diagnostics(plan, &mut report);
    if !report.has_fatal_diagnostics() {
        report.modules.extend(
            plan.module_proposals()
                .iter()
                .map(|proposal| descriptor_backed_module(proposal.descriptor().clone())),
        );
    }
    report
}

fn append_plan_diagnostics(plan: &CompiledProjectPluginPlan, report: &mut RuntimeModuleLoadReport) {
    let extensions = plan.runtime_extensions();
    let mut nonfatal_diagnostics = HashMap::new();
    for diagnostic in &extensions.diagnostics {
        nonfatal_diagnostics
            .entry(diagnostic.as_str())
            .and_modify(|count| *count += 1)
            .or_insert(1usize);
    }
    for diagnostic in &extensions.fatal_diagnostics {
        nonfatal_diagnostics
            .entry(diagnostic.as_str())
            .and_modify(|count| *count = count.saturating_sub(1))
            .or_insert(0usize);
    }
    for diagnostic in &extensions.diagnostics {
        let Some(count) = nonfatal_diagnostics.get_mut(diagnostic.as_str()) else {
            continue;
        };
        if *count == 0 {
            continue;
        }
        *count -= 1;
        report.push_diagnostic(RuntimeModuleLoadDiagnostic::RuntimePluginPlan {
            message: diagnostic.clone(),
            fatal: false,
        });
    }
    for diagnostic in &extensions.fatal_diagnostics {
        report.push_diagnostic(RuntimeModuleLoadDiagnostic::RuntimePluginPlan {
            message: diagnostic.clone(),
            fatal: true,
        });
    }
}
