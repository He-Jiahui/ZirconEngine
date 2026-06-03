use crate::{plugin::ProjectPluginManifest, RuntimeTargetMode};

use super::tokens::is_lowercase_project_runtime_crate;

pub(in crate::plugin::export_build_plan) fn sanitize_invalid_project_crate_overrides(
    manifest: &mut ProjectPluginManifest,
    target: RuntimeTargetMode,
) {
    for selection in manifest
        .selections
        .iter_mut()
        .filter(|selection| selection.enabled && selection.supports_target(target))
    {
        if !project_selection_runtime_crate_override_is_valid(selection.runtime_crate.as_deref()) {
            selection.runtime_crate = None;
        }
        if !project_runtime_crate_override_is_valid(selection.editor_crate.as_deref()) {
            selection.editor_crate = None;
        }
        for feature in selection
            .features
            .iter_mut()
            .filter(|feature| feature.enabled && feature.supports_target(target))
        {
            if !project_runtime_crate_override_is_valid(feature.runtime_crate.as_deref()) {
                feature.runtime_crate = None;
            }
            if !project_runtime_crate_override_is_valid(feature.editor_crate.as_deref()) {
                feature.editor_crate = None;
            }
        }
    }
}

pub(in crate::plugin::export_build_plan) fn project_runtime_crate_diagnostics(
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
        if let Some(crate_name) = selection.runtime_crate.as_deref() {
            let first_diagnostic = diagnostics.len();
            validate_project_selection_runtime_crate_name(
                &format!("project plugin {} runtime_crate", selection.id),
                crate_name,
                &mut diagnostics,
            );
            if selection.required {
                fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
            }
        }
        for feature in selection
            .features
            .iter()
            .filter(|feature| feature.enabled && feature.supports_target(target))
        {
            let Some(crate_name) = feature.runtime_crate.as_deref() else {
                continue;
            };
            let first_diagnostic = diagnostics.len();
            validate_project_runtime_crate_name(
                &format!("project plugin feature {} runtime_crate", feature.id),
                crate_name,
                &mut diagnostics,
            );
            if feature.required {
                fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
            }
        }
    }
    (diagnostics, fatal_diagnostics)
}

pub(in crate::plugin::export_build_plan) fn project_editor_crate_diagnostics(
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
        if let Some(crate_name) = selection.editor_crate.as_deref() {
            let first_diagnostic = diagnostics.len();
            validate_project_runtime_crate_name(
                &format!("project plugin {} editor_crate", selection.id),
                crate_name,
                &mut diagnostics,
            );
            if selection.required {
                fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
            }
        }
        for feature in selection
            .features
            .iter()
            .filter(|feature| feature.enabled && feature.supports_target(target))
        {
            let Some(crate_name) = feature.editor_crate.as_deref() else {
                continue;
            };
            let first_diagnostic = diagnostics.len();
            validate_project_runtime_crate_name(
                &format!("project plugin feature {} editor_crate", feature.id),
                crate_name,
                &mut diagnostics,
            );
            if feature.required {
                fatal_diagnostics.extend(diagnostics[first_diagnostic..].iter().cloned());
            }
        }
    }
    (diagnostics, fatal_diagnostics)
}

fn validate_project_runtime_crate_name(
    context: &str,
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    if crate_name.trim().is_empty() || crate_name.trim() != crate_name {
        diagnostics.push(format!(
            "{context} `{crate_name}` must be non-empty and trimmed"
        ));
    }
    if !crate_name.starts_with("zircon_plugin_") || !is_lowercase_project_runtime_crate(crate_name)
    {
        diagnostics.push(format!(
            "{context} `{crate_name}` must use `zircon_plugin_` prefix and contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
    if crate_name.ends_with('_') || crate_name.contains("__") {
        diagnostics.push(format!(
            "{context} `{crate_name}` must not end with an underscore or contain repeated underscores"
        ));
    }
}

fn validate_project_selection_runtime_crate_name(
    context: &str,
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    if crate_name.trim().is_empty() || crate_name.trim() != crate_name {
        diagnostics.push(format!(
            "{context} `{crate_name}` must be non-empty and trimmed"
        ));
    }
    if !project_selection_runtime_crate_name_prefix_is_valid(crate_name)
        || !is_lowercase_project_runtime_crate(crate_name)
    {
        diagnostics.push(format!(
            "{context} `{crate_name}` must use `zircon_plugin_` crate prefix or `builtin_` runtime-domain prefix and contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
    if crate_name.ends_with('_') || crate_name.contains("__") {
        diagnostics.push(format!(
            "{context} `{crate_name}` must not end with an underscore or contain repeated underscores"
        ));
    }
}

pub(in crate::plugin::export_build_plan) fn project_runtime_crate_override_is_valid(
    crate_name: Option<&str>,
) -> bool {
    match crate_name {
        Some(crate_name) => project_runtime_crate_name_is_valid(crate_name),
        None => true,
    }
}

fn project_selection_runtime_crate_override_is_valid(crate_name: Option<&str>) -> bool {
    match crate_name {
        Some(crate_name) => project_selection_runtime_crate_name_is_valid(crate_name),
        None => true,
    }
}

pub(in crate::plugin::export_build_plan) fn project_runtime_crate_name_is_valid(
    crate_name: &str,
) -> bool {
    !crate_name.trim().is_empty()
        && crate_name.trim() == crate_name
        && crate_name.starts_with("zircon_plugin_")
        && is_lowercase_project_runtime_crate(crate_name)
        && !crate_name.ends_with('_')
        && !crate_name.contains("__")
}

fn project_selection_runtime_crate_name_is_valid(crate_name: &str) -> bool {
    !crate_name.trim().is_empty()
        && crate_name.trim() == crate_name
        && project_selection_runtime_crate_name_prefix_is_valid(crate_name)
        && is_lowercase_project_runtime_crate(crate_name)
        && !crate_name.ends_with('_')
        && !crate_name.contains("__")
}

fn project_selection_runtime_crate_name_prefix_is_valid(crate_name: &str) -> bool {
    crate_name.starts_with("zircon_plugin_") || crate_name.starts_with("builtin_")
}
