use crate::core::framework::platform::RuntimeTargetMode;
use crate::{
    core::framework::project::ExportBuildMode, core::framework::project::ExportPackagingStrategy,
    core::framework::project::ExportProfile, core::framework::project::ExportTargetPlatform,
    core::framework::project::ProjectPluginFeatureSelection,
    core::framework::project::ProjectPluginSelection, core::framework::project::RuntimeProfileId,
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
        .map(|linked_crate| {
            format!(
                "ExportRuntimePluginRegistrationProvider::new({}::plugin_registration)",
                linked_crate.crate_name
            )
        })
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
        "use std::collections::BTreeMap;\nuse zircon_app::{{ExportRuntimeBootstrapConfig, ExportRuntimePluginFeatureRegistrationProvider, ExportRuntimePluginRegistrationProvider}};\nuse zircon_runtime::{{core::framework::platform::RuntimeTargetMode, core::framework::project::ExportBuildMode, core::framework::project::ExportPackagingStrategy, core::framework::project::ExportProfile, core::framework::project::ExportTargetPlatform, core::framework::project::ProjectPluginFeatureSelection, core::framework::project::ProjectPluginManifest, core::framework::project::ProjectPluginSelection, core::framework::project::RuntimeProfileId}};\n\npub fn export_runtime_bootstrap_config() -> ExportRuntimeBootstrapConfig {{\n    ExportRuntimeBootstrapConfig::new(\n        project_plugins(),\n        export_profile(),\n    )\n    .with_runtime_plugin_registration_providers(runtime_plugin_registration_providers())\n    .with_runtime_plugin_feature_registration_providers(runtime_plugin_feature_registration_providers())\n}}\n\npub fn target_mode() -> RuntimeTargetMode {{\n    {}\n}}\n\npub fn export_profile() -> ExportProfile {{\n    ExportProfile {{\n        name: {:?}.to_string(),\n        target_mode: target_mode(),\n        runtime_profile_id: {},\n        target_platform: {},\n        strategies: vec![{}],\n        build_mode: {},\n        output_name: {:?}.to_string(),\n        selected_plugins: vec![{}],\n        features: {},\n        asset_filter: {},\n    }}\n}}\n\npub fn project_plugins() -> ProjectPluginManifest {{\n    ProjectPluginManifest {{\n        selections: vec![\n{}\n        ],\n    }}\n}}\n\npub fn runtime_plugin_registration_providers() -> Vec<ExportRuntimePluginRegistrationProvider> {{\n    vec![\n{}\n    ]\n}}\n\npub fn runtime_plugin_feature_registration_providers() -> Vec<ExportRuntimePluginFeatureRegistrationProvider> {{\n    vec![\n{}\n    ]\n}}\n",
        target_mode_expr(profile.target_mode),
        profile.name,
        runtime_profile_id_expr(profile.runtime_profile_id),
        target_platform_expr(profile.target_platform),
        strategies,
        build_mode_expr(profile.build_mode),
        profile.output_name,
        string_vec_expr(&profile.selected_plugins),
        feature_map_expr(profile),
        option_string_expr(profile.asset_filter.as_deref()),
        indent_lines(&selections, 12),
        indent_lines(&registration_calls, 8),
        indent_lines(&feature_registration_calls, 8)
    )
}

fn feature_registration_call(linked_crate: &ExportLinkedRuntimeCrate) -> String {
    let call = format!(
        "ExportRuntimePluginFeatureRegistrationProvider::new({}::plugin_feature_registration)",
        linked_crate.crate_name
    );
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

fn runtime_profile_id_expr(profile_id: Option<RuntimeProfileId>) -> String {
    match profile_id {
        Some(RuntimeProfileId::Minimal) => "Some(RuntimeProfileId::Minimal)".to_string(),
        Some(RuntimeProfileId::Client2d) => "Some(RuntimeProfileId::Client2d)".to_string(),
        Some(RuntimeProfileId::Client3d) => "Some(RuntimeProfileId::Client3d)".to_string(),
        Some(RuntimeProfileId::Editor) => "Some(RuntimeProfileId::Editor)".to_string(),
        Some(RuntimeProfileId::Dev) => "Some(RuntimeProfileId::Dev)".to_string(),
        Some(RuntimeProfileId::Server) => "Some(RuntimeProfileId::Server)".to_string(),
        None => "None".to_string(),
    }
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
        ExportTargetPlatform::Headless => "ExportTargetPlatform::Headless",
    }
}

fn build_mode_expr(build_mode: ExportBuildMode) -> &'static str {
    match build_mode {
        ExportBuildMode::Debug => "ExportBuildMode::Debug",
        ExportBuildMode::Release => "ExportBuildMode::Release",
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

fn string_vec_expr(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("{value:?}.to_string()"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn feature_map_expr(profile: &ExportProfile) -> String {
    if profile.features.is_empty() {
        return "BTreeMap::new()".to_string();
    }

    let mut lines = vec!["BTreeMap::from([".to_string()];
    for (owner, features) in &profile.features {
        lines.push(format!(
            "    ({owner:?}.to_string(), vec![{}]),",
            string_vec_expr(features)
        ));
    }
    lines.push("])".to_string());
    lines.join("\n")
}

fn indent_lines(value: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
