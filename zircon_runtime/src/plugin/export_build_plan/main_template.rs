use crate::core::framework::project::ExportProfile;

pub(super) fn main_template(_profile: &ExportProfile, has_native_dynamic_plugins: bool) -> String {
    if has_native_dynamic_plugins {
        return "mod zircon_plugins;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    let authority = zircon_runtime::plugin::native::NativePluginArtifactAuthority::from_embedded_build_json(include_str!(\"zircon_native_authority.json\"))?;\n    let composition = zircon_app::bootstrap_export_runtime_with_native_plugins_from_export_root(\n        zircon_plugins::export_runtime_bootstrap_config().with_native_plugin_artifact_authority(authority),\n        zircon_app::discover_export_root()?,\n    )?;\n    composition.close_until(std::time::Instant::now() + std::time::Duration::from_secs(5))?;\n    Ok(())\n}\n"
            .to_string();
    }
    "mod zircon_plugins;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    let composition = zircon_app::bootstrap_export_runtime(\n        zircon_plugins::export_runtime_bootstrap_config(),\n    )?;\n    composition.close_until(std::time::Instant::now() + std::time::Duration::from_secs(5))?;\n    Ok(())\n}\n"
        .to_string()
}
