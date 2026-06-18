use crate::builtin::RuntimeTargetMode;
use crate::plugin::ProjectPluginManifest;

use super::tokens::{target_consumes_feature, target_consumes_selection};

pub(in crate::plugin::export_build_plan) fn sanitize_project_target_mode_rows(
    manifest: &mut ProjectPluginManifest,
    target: RuntimeTargetMode,
) {
    for selection in manifest
        .selections
        .iter_mut()
        .filter(|selection| target_consumes_selection(selection, target))
    {
        deduplicate_target_modes(&mut selection.target_modes);
        for feature in selection
            .features
            .iter_mut()
            .filter(|feature| target_consumes_feature(feature, target))
        {
            deduplicate_target_modes(&mut feature.target_modes);
        }
    }
}

pub(in crate::plugin::export_build_plan) fn project_target_mode_diagnostics(
    manifest: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    for selection in manifest
        .selections
        .iter()
        .filter(|selection| selection.enabled && selection.supports_target(target))
    {
        let first_diagnostic = diagnostics.len();
        validate_project_target_modes(
            &format!("project plugin {} target_modes", selection.id),
            &selection.target_modes,
            &mut diagnostics,
        );
        if selection.required {
            fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
        }
        for feature in selection
            .features
            .iter()
            .filter(|feature| feature.enabled && feature.supports_target(target))
        {
            let first_diagnostic = diagnostics.len();
            validate_project_target_modes(
                &format!("project plugin feature {} target_modes", feature.id),
                &feature.target_modes,
                &mut diagnostics,
            );
            if feature.required {
                fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
            }
        }
    }
    (diagnostics, fatal_diagnostics)
}

fn validate_project_target_modes(
    context: &str,
    target_modes: &[RuntimeTargetMode],
    diagnostics: &mut Vec<String>,
) {
    let mut seen = Vec::new();
    for target_mode in target_modes.iter().copied() {
        if seen.contains(&target_mode) {
            diagnostics.push(format!(
                "{context} must not repeat target mode {target_mode:?}"
            ));
        } else {
            seen.push(target_mode);
        }
    }
}

fn deduplicate_target_modes(target_modes: &mut Vec<RuntimeTargetMode>) {
    let mut seen = Vec::new();
    target_modes.retain(|target_mode| {
        if seen.contains(target_mode) {
            false
        } else {
            seen.push(*target_mode);
            true
        }
    });
}
