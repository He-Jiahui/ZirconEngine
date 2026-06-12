use crate::plugin::ExportProfile;

pub(super) fn main_template(_profile: &ExportProfile, has_native_dynamic_plugins: bool) -> String {
    if has_native_dynamic_plugins {
        return "mod zircon_plugins;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    let _bootstrap = zircon_app::bootstrap_export_runtime_with_native_plugins_from_export_root(\n        zircon_plugins::export_runtime_bootstrap_config(),\n        zircon_app::discover_export_root()?,\n    )?;\n    Ok(())\n}\n"
            .to_string();
    }
    "mod zircon_plugins;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    let _core = zircon_app::bootstrap_export_runtime(\n        zircon_plugins::export_runtime_bootstrap_config(),\n    )?;\n    Ok(())\n}\n"
        .to_string()
}
