use crate::{plugin::ExportProfile, RuntimeTargetMode};

pub(super) fn main_template(profile: &ExportProfile, has_native_dynamic_plugins: bool) -> String {
    let entry_profile = match profile.target_mode {
        RuntimeTargetMode::ClientRuntime => "Runtime",
        RuntimeTargetMode::ServerRuntime => "Headless",
        RuntimeTargetMode::EditorHost => "Editor",
    };
    if has_native_dynamic_plugins {
        return format!(
            "mod zircon_plugins;\n\nuse std::path::PathBuf;\n\nuse zircon_app::{{EntryConfig, EntryProfile, EntryRunner}};\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {{\n    let config = EntryConfig::new(EntryProfile::{entry_profile})\n        .with_target_mode(zircon_plugins::target_mode())\n        .with_project_plugins(zircon_plugins::project_plugins())\n        .with_export_profile(zircon_plugins::export_profile());\n    let mut registrations = zircon_plugins::runtime_plugin_registrations();\n    let native_report = zircon_runtime::plugin::NativePluginLoader.load_runtime_from_load_manifest(export_root()?);\n    registrations.extend(native_report.runtime_plugin_registration_reports());\n    let mut feature_registrations = zircon_plugins::runtime_plugin_feature_registrations();\n    feature_registrations.extend(native_report.runtime_plugin_feature_registration_reports());\n    let _core = EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations(\n        config,\n        registrations,\n        feature_registrations,\n    )?;\n    Ok(())\n}}\n\nfn export_root() -> Result<PathBuf, Box<dyn std::error::Error>> {{\n    let current_exe = std::env::current_exe()?;\n    let current_dir = std::env::current_dir()?;\n    let mut candidates = Vec::new();\n    if let Some(parent) = current_exe.parent() {{\n        candidates.extend(parent.ancestors().map(PathBuf::from));\n    }}\n    candidates.extend(current_dir.ancestors().map(PathBuf::from));\n    for root in candidates {{\n        if root.join(\"plugins/native_plugins.toml\").exists() {{\n            return Ok(root);\n        }}\n    }}\n    Ok(current_dir)\n}}\n"
        );
    }
    format!(
        "mod zircon_plugins;\n\nuse zircon_app::{{EntryConfig, EntryProfile, EntryRunner}};\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {{\n    let config = EntryConfig::new(EntryProfile::{entry_profile})\n        .with_target_mode(zircon_plugins::target_mode())\n        .with_project_plugins(zircon_plugins::project_plugins())\n        .with_export_profile(zircon_plugins::export_profile());\n    let _core = EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations(\n        config,\n        zircon_plugins::runtime_plugin_registrations(),\n        zircon_plugins::runtime_plugin_feature_registrations(),\n    )?;\n    Ok(())\n}}\n"
    )
}
