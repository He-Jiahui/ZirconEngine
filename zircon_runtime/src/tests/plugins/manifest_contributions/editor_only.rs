use super::*;

#[test]
fn editor_only_plugin_tomls_declare_package_level_targets_and_capabilities() {
    let plugins_root = plugins_workspace_root();

    for (id, category, editor_crate, capabilities) in [
        (
            "material_editor",
            "authoring",
            "zircon_plugin_material_editor_editor",
            vec!["editor.extension.material_editor_authoring"],
        ),
        (
            "timeline_sequence",
            "authoring",
            "zircon_plugin_timeline_sequence_editor",
            vec!["editor.extension.timeline_sequence_authoring"],
        ),
        (
            "animation_graph",
            "authoring",
            "zircon_plugin_animation_graph_editor",
            vec!["editor.extension.animation_graph_authoring"],
        ),
        (
            "runtime_diagnostics",
            "diagnostics",
            "zircon_plugin_runtime_diagnostics_editor",
            vec!["editor.extension.runtime_diagnostics"],
        ),
        (
            "ui_asset_authoring",
            "authoring",
            "zircon_plugin_ui_asset_authoring_editor",
            vec!["editor.extension.ui_asset_authoring"],
        ),
        (
            "native_window_hosting",
            "platform",
            "zircon_plugin_native_window_hosting_editor",
            vec!["editor.extension.native_window_hosting"],
        ),
        (
            "editor_build_export_desktop",
            "platform",
            "zircon_plugin_editor_build_export_desktop_editor",
            vec![
                "editor.extension.build_export_desktop",
                "editor.extension.build_export_desktop.diagnostics",
                "editor.extension.build_export_desktop.native_dynamic_report",
            ],
        ),
        (
            "plugin_sdk_examples",
            "sdk",
            "zircon_plugin_sdk_examples_editor",
            vec![
                "editor.extension.plugin_sdk_examples",
                "editor.extension.plugin_sdk_examples.window",
                "editor.extension.plugin_sdk_examples.asset_fixture",
            ],
        ),
    ] {
        let manifest = read_plugin_manifest(&plugins_root, id);
        let editor_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .expect("editor-only plugin should declare an editor module");
        let capabilities = capabilities
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(manifest.category, category);
        assert_eq!(
            manifest.supported_targets,
            vec![RuntimeTargetMode::EditorHost],
            "editor-only plugin `{id}` should publish editor_host as package metadata"
        );
        assert_eq!(
            manifest.capabilities, capabilities,
            "editor-only plugin `{id}` should publish editor capability as package metadata"
        );
        assert_eq!(editor_module.crate_name, editor_crate);
        assert_eq!(editor_module.capabilities, manifest.capabilities);
    }
}

#[test]
fn low_overlap_editor_only_plugin_tomls_declare_explicit_experimental_maturity() {
    let plugins_root = plugins_workspace_root();

    for id in [
        "runtime_diagnostics",
        "native_window_hosting",
        "editor_build_export_desktop",
        "plugin_sdk_examples",
    ] {
        let manifest_source = fs::read_to_string(plugins_root.join(id).join("plugin.toml"))
            .expect("editor-only plugin manifest source");
        let manifest = read_plugin_manifest(&plugins_root, id);

        assert!(
            manifest_source.contains(r#"maturity = "experimental""#),
            "editor-only plugin `{id}` should explicitly declare experimental maturity"
        );
        assert_eq!(
            manifest.maturity,
            crate::plugin::PluginMaturity::Experimental
        );
    }
}

#[test]
fn editor_authoring_plugin_tomls_declare_explicit_experimental_maturity() {
    let plugins_root = plugins_workspace_root();

    for id in [
        "material_editor",
        "timeline_sequence",
        "animation_graph",
        "ui_asset_authoring",
    ] {
        let manifest_source = fs::read_to_string(plugins_root.join(id).join("plugin.toml"))
            .expect("editor-only authoring plugin manifest source");
        let manifest = read_plugin_manifest(&plugins_root, id);

        assert!(
            manifest_source.contains(r#"maturity = "experimental""#),
            "editor-only authoring plugin `{id}` should explicitly declare experimental maturity"
        );
        assert_eq!(
            manifest.maturity,
            crate::plugin::PluginMaturity::Experimental
        );
    }
}
