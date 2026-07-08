use std::fs;
use std::path::Path;

use super::support::relative_path;

#[test]
fn export_entry_templates_delegate_to_app_export_bootstrap_facade() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let entry_template_paths = [
        manifest_root
            .join("src")
            .join("plugin")
            .join("export_build_plan")
            .join("main_template.rs"),
        manifest_root
            .join("src")
            .join("plugin")
            .join("export_build_plan")
            .join("platform_host_files.rs"),
    ];
    let forbidden_entry_tokens = [
        "EntryRunner::",
        "EntryConfig::new",
        "NativePluginLoader",
        "load_runtime_from_load_manifest",
        "zircon_plugins::runtime_plugin_registrations()",
        "zircon_plugins::runtime_plugin_feature_registrations()",
    ];
    let mut violations = Vec::new();
    let mut combined_source = String::new();

    for path in entry_template_paths {
        let relative = relative_path(manifest_root, &path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        combined_source.push_str(&source);
        combined_source.push('\n');

        for (line_index, line) in source.lines().enumerate() {
            for token in forbidden_entry_tokens {
                if line.contains(token) {
                    violations.push(format!(
                        "{}:{}: `{}` in {}",
                        relative,
                        line_index + 1,
                        token,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "generated entry templates must call the handwritten app export bootstrap facade instead of owning startup/native-loader behavior:\n{}",
        violations.join("\n")
    );
    for required in [
        "zircon_app::bootstrap_export_runtime",
        "zircon_app::bootstrap_export_runtime_with_native_plugins_from_export_root",
        "zircon_app::discover_export_root()?",
        "zircon_plugins::export_runtime_bootstrap_config()",
    ] {
        assert!(
            combined_source.contains(required),
            "generated entry templates should keep the thin export-bootstrap facade call `{required}`"
        );
    }
}

#[test]
fn export_plugin_selection_template_delegates_registration_execution_to_app_providers() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let template_path = manifest_root
        .join("src")
        .join("plugin")
        .join("export_build_plan")
        .join("plugin_selection_template.rs");
    let source = fs::read_to_string(&template_path).unwrap_or_else(|error| {
        panic!(
            "failed to read {}: {error}",
            relative_path(manifest_root, &template_path)
        )
    });

    for forbidden in ["plugin_registration()", "plugin_feature_registration()"] {
        assert!(
            !source.contains(forbidden),
            "plugin selection templates must pass registration providers to the app facade instead of directly calling `{forbidden}`"
        );
    }
    for required in [
        "ExportRuntimePluginRegistrationProvider::new",
        "ExportRuntimePluginFeatureRegistrationProvider::new",
        ".with_runtime_plugin_registration_providers(runtime_plugin_registration_providers())",
        ".with_runtime_plugin_feature_registration_providers(runtime_plugin_feature_registration_providers())",
    ] {
        assert!(
            source.contains(required),
            "plugin selection templates should keep provider-table handoff `{required}`"
        );
    }
}
