use crate::core::framework::project::{ExportPackagingStrategy, ExportProfile};
use crate::plugin::RuntimeProfileDescriptor;

pub(super) fn export_profile_duplicate_name_fatal_diagnostics(
    profiles: &[ExportProfile],
    profile_name: &str,
) -> Vec<String> {
    let matching_profiles = profiles
        .iter()
        .filter(|profile| profile.name == profile_name)
        .count();
    if matching_profiles > 1 {
        vec![format!(
            "export profile name {profile_name:?} must be unique; found {matching_profiles} matching export profiles"
        )]
    } else {
        Vec::new()
    }
}

pub(super) fn export_profile_output_name_diagnostics(profile: &ExportProfile) -> Vec<String> {
    let normalized_output_name = normalized_export_profile_output_name(profile);
    if profile.output_name == normalized_output_name {
        return Vec::new();
    }

    vec![format!(
        "export profile {} output_name {:?} must be non-empty and trimmed; generated export metadata will use {:?}",
        profile.name, profile.output_name, normalized_output_name
    )]
}

pub(super) fn export_profile_runtime_profile_target_fatal_diagnostics(
    profile: &ExportProfile,
    runtime_profile: &RuntimeProfileDescriptor,
) -> Vec<String> {
    if profile.runtime_profile_id.is_some() && runtime_profile.target_mode != profile.target_mode {
        vec![format!(
            "export profile {} selects runtime profile {:?} with target mode {:?}, but export target mode is {:?}",
            profile.name, runtime_profile.id, runtime_profile.target_mode, profile.target_mode
        )]
    } else {
        Vec::new()
    }
}

pub(super) fn export_profile_runtime_profile_id_fatal_diagnostics(
    profile: &ExportProfile,
) -> Vec<String> {
    if profile.runtime_profile_id.is_none() {
        vec![format!(
            "export profile {:?} must declare runtime_profile_id explicitly",
            profile.name
        )]
    } else {
        Vec::new()
    }
}

pub(super) fn export_profile_name_fatal_diagnostics(profile: &ExportProfile) -> Vec<String> {
    if profile.name.trim().is_empty() || profile.name.trim() != profile.name {
        vec![format!(
            "export profile name {:?} must be non-empty and trimmed",
            profile.name
        )]
    } else {
        Vec::new()
    }
}

pub(super) fn export_profile_strategy_diagnostics(profile: &ExportProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut seen = Vec::new();
    for strategy in profile.strategies.iter().copied() {
        if seen.contains(&strategy) {
            diagnostics.push(format!(
                "export profile {} strategies must not repeat packaging strategy {strategy:?}",
                profile.name
            ));
        } else {
            seen.push(strategy);
        }
    }
    diagnostics
}

pub(super) fn export_profile_strategy_fatal_diagnostics(profile: &ExportProfile) -> Vec<String> {
    if profile.strategies.is_empty() {
        vec![format!(
            "export profile {} strategies must include at least one packaging strategy",
            profile.name
        )]
    } else {
        Vec::new()
    }
}

pub(super) fn sanitize_export_profile_output_name(profile: &mut ExportProfile) {
    profile.output_name = normalized_export_profile_output_name(profile);
}

pub(super) fn sanitize_export_profile_strategies(profile: &mut ExportProfile) {
    deduplicate_export_strategies(&mut profile.strategies);
}

fn normalized_export_profile_output_name(profile: &ExportProfile) -> String {
    let trimmed_output_name = profile.output_name.trim();
    if !trimmed_output_name.is_empty() {
        return trimmed_output_name.to_string();
    }

    let fallback_name = profile.name.trim();
    if fallback_name.is_empty() {
        "export".to_string()
    } else {
        fallback_name.to_string()
    }
}

fn deduplicate_export_strategies(strategies: &mut Vec<ExportPackagingStrategy>) {
    let mut seen = Vec::new();
    strategies.retain(|strategy| {
        if seen.contains(strategy) {
            false
        } else {
            seen.push(*strategy);
            true
        }
    });
}
