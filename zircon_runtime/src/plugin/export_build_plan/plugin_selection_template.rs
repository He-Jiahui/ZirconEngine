use crate::{
    plugin::ExportPackagingStrategy, plugin::ExportProfile, plugin::ExportTargetPlatform,
    plugin::ProjectPluginFeatureSelection, plugin::ProjectPluginSelection, RuntimeTargetMode,
};

use super::{ExportLinkedRuntimeCrate, ExportRuntimeCrateRegistrationKind};

pub(super) fn plugin_selection_template(
    profile: &ExportProfile,
    project_plugin_selections: &[&ProjectPluginSelection],
    linked_runtime_crates: &[ExportLinkedRuntimeCrate],
) -> String {
    let selections = project_plugin_selections
        .iter()
        .map(|selection| selection_template(selection))
        .collect::<Vec<_>>()
        .join(",\n");
    let strategies = profile
        .strategies
        .iter()
        .map(|strategy| packaging_strategy_expr(*strategy))
        .collect::<Vec<_>>()
        .join(", ");
    let registration_calls = linked_runtime_crates
        .iter()
        .filter(|linked_crate| {
            linked_crate.registration_kind == ExportRuntimeCrateRegistrationKind::RuntimePlugin
        })
        .map(|linked_crate| format!("{}::plugin_registration()", linked_crate.crate_name))
        .collect::<Vec<_>>()
        .join(",\n");
    let feature_registration_calls = linked_runtime_crates
        .iter()
        .filter(|linked_crate| {
            linked_crate.registration_kind == ExportRuntimeCrateRegistrationKind::RuntimeFeature
        })
        .map(feature_registration_call)
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "use zircon_runtime::{{plugin::ExportPackagingStrategy, plugin::ExportProfile, plugin::ExportTargetPlatform, plugin::ProjectPluginFeatureSelection, plugin::ProjectPluginManifest, plugin::ProjectPluginSelection, plugin::RuntimePluginFeatureRegistrationReport, plugin::RuntimePluginRegistrationReport, RuntimeTargetMode}};\n\npub fn target_mode() -> RuntimeTargetMode {{\n    {}\n}}\n\npub fn export_profile() -> ExportProfile {{\n    ExportProfile {{\n        name: {:?}.to_string(),\n        target_mode: target_mode(),\n        target_platform: {},\n        strategies: vec![{}],\n        output_name: {:?}.to_string(),\n    }}\n}}\n\npub fn project_plugins() -> ProjectPluginManifest {{\n    ProjectPluginManifest {{\n        selections: vec![\n{}\n        ],\n    }}\n}}\n\npub fn runtime_plugin_registrations() -> Vec<RuntimePluginRegistrationReport> {{\n    vec![\n{}\n    ]\n}}\n\npub fn runtime_plugin_feature_registrations() -> Vec<RuntimePluginFeatureRegistrationReport> {{\n    vec![\n{}\n    ]\n}}\n",
        target_mode_expr(profile.target_mode),
        profile.name,
        target_platform_expr(profile.target_platform),
        strategies,
        profile.output_name,
        indent_lines(&selections, 12),
        indent_lines(&registration_calls, 8),
        indent_lines(&feature_registration_calls, 8)
    )
}

fn feature_registration_call(linked_crate: &ExportLinkedRuntimeCrate) -> String {
    let call = format!("{}::plugin_feature_registration()", linked_crate.crate_name);
    match linked_crate.provider_package_id.as_deref() {
        Some(provider_package_id) => {
            format!("{call}.with_provider_package_id({provider_package_id:?})")
        }
        None => call,
    }
}

fn selection_template(selection: &ProjectPluginSelection) -> String {
    let features = selection
        .features
        .iter()
        .map(feature_selection_template)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "ProjectPluginSelection {{ id: {:?}.to_string(), enabled: {}, required: {}, target_modes: vec![{}], packaging: {}, runtime_crate: {}, editor_crate: {}, features: vec![{}] }}",
        selection.id,
        selection.enabled,
        selection.required,
        selection
            .target_modes
            .iter()
            .map(|target| target_mode_expr(*target))
            .collect::<Vec<_>>()
            .join(", "),
        packaging_strategy_expr(selection.packaging),
        option_string_expr(selection.runtime_crate.as_deref()),
        option_string_expr(selection.editor_crate.as_deref()),
        features
    )
}

fn feature_selection_template(selection: &ProjectPluginFeatureSelection) -> String {
    format!(
        "ProjectPluginFeatureSelection {{ id: {:?}.to_string(), enabled: {}, required: {}, target_modes: vec![{}], packaging: {}, runtime_crate: {}, editor_crate: {}, provider_package_id: {} }}",
        selection.id,
        selection.enabled,
        selection.required,
        selection
            .target_modes
            .iter()
            .map(|target| target_mode_expr(*target))
            .collect::<Vec<_>>()
            .join(", "),
        packaging_strategy_expr(selection.packaging),
        option_string_expr(selection.runtime_crate.as_deref()),
        option_string_expr(selection.editor_crate.as_deref()),
        option_string_expr(selection.provider_package_id.as_deref())
    )
}

fn target_mode_expr(target_mode: RuntimeTargetMode) -> &'static str {
    match target_mode {
        RuntimeTargetMode::ClientRuntime => "RuntimeTargetMode::ClientRuntime",
        RuntimeTargetMode::ServerRuntime => "RuntimeTargetMode::ServerRuntime",
        RuntimeTargetMode::EditorHost => "RuntimeTargetMode::EditorHost",
    }
}

fn target_platform_expr(platform: ExportTargetPlatform) -> &'static str {
    match platform {
        ExportTargetPlatform::Windows => "ExportTargetPlatform::Windows",
        ExportTargetPlatform::Linux => "ExportTargetPlatform::Linux",
        ExportTargetPlatform::Macos => "ExportTargetPlatform::Macos",
        ExportTargetPlatform::Android => "ExportTargetPlatform::Android",
        ExportTargetPlatform::Ios => "ExportTargetPlatform::Ios",
        ExportTargetPlatform::WebGpu => "ExportTargetPlatform::WebGpu",
        ExportTargetPlatform::Wasm => "ExportTargetPlatform::Wasm",
    }
}

fn packaging_strategy_expr(strategy: ExportPackagingStrategy) -> &'static str {
    match strategy {
        ExportPackagingStrategy::SourceTemplate => "ExportPackagingStrategy::SourceTemplate",
        ExportPackagingStrategy::LibraryEmbed => "ExportPackagingStrategy::LibraryEmbed",
        ExportPackagingStrategy::NativeDynamic => "ExportPackagingStrategy::NativeDynamic",
    }
}

fn option_string_expr(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("Some({value:?}.to_string())"),
        None => "None".to_string(),
    }
}

fn indent_lines(value: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
