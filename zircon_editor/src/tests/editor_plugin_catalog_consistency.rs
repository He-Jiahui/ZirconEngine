use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::{
    plugin::PluginModuleKind, plugin::PluginPackageManifest, plugin::RuntimePluginCatalog,
    RuntimeTargetMode,
};

use crate::core::editor_plugin::{EditorPluginCatalog, EditorPluginDescriptor};

#[test]
fn builtin_editor_catalog_entries_have_matching_plugin_manifests_and_workspace_members() {
    let plugins_root = plugins_workspace_root();
    let workspace_members = plugin_workspace_members(&plugins_root);

    for descriptor in EditorPluginDescriptor::builtin_catalog() {
        let manifest = read_plugin_manifest(&plugins_root, &descriptor.package_id);
        assert_eq!(manifest.id, descriptor.package_id);
        assert!(
            workspace_members.contains(&format!("{}/editor", descriptor.package_id)),
            "editor catalog entry `{}` is missing its zircon_plugins workspace editor member",
            descriptor.package_id
        );
        assert!(
            manifest.modules.iter().any(|module| {
                module.kind == PluginModuleKind::Editor
                    && module.crate_name == descriptor.crate_name
            }),
            "editor catalog entry `{}` is missing matching editor module crate `{}` in plugin.toml",
            descriptor.package_id,
            descriptor.crate_name
        );
    }
}

#[test]
fn editor_workspace_plugin_manifests_are_present_in_builtin_catalog() {
    let plugins_root = plugins_workspace_root();
    let workspace_members = plugin_workspace_members(&plugins_root);
    let catalog_ids = EditorPluginDescriptor::builtin_catalog()
        .into_iter()
        .map(|descriptor| descriptor.package_id)
        .collect::<BTreeSet<_>>();

    for manifest_path in plugin_manifest_paths(&plugins_root) {
        let manifest_source = fs::read_to_string(&manifest_path).expect("plugin manifest source");
        let manifest: PluginPackageManifest =
            toml::from_str(&manifest_source).expect("plugin manifest should parse");
        let has_workspace_editor_member =
            workspace_members.contains(&format!("{}/editor", manifest.id));
        let declares_editor_module = manifest
            .modules
            .iter()
            .any(|module| module.kind == PluginModuleKind::Editor);
        if has_workspace_editor_member && declares_editor_module {
            assert!(
                catalog_ids.contains(&manifest.id),
                "editor plugin `{}` is missing from EditorPluginDescriptor::builtin_catalog()",
                manifest.id
            );
        }
    }
}

#[test]
fn editor_only_builtin_catalog_projects_targets_and_capabilities_from_package_manifests() {
    let plugins_root = plugins_workspace_root();
    let catalog = EditorPluginCatalog::builtin(RuntimePluginCatalog::builtin().package_manifests());
    let catalog_manifests = catalog.package_manifests();

    for (package_id, category, capabilities) in [
        (
            "material_editor",
            "authoring",
            vec!["editor.extension.material_editor_authoring"],
        ),
        (
            "timeline_sequence",
            "authoring",
            vec!["editor.extension.timeline_sequence_authoring"],
        ),
        (
            "animation_graph",
            "authoring",
            vec!["editor.extension.animation_graph_authoring"],
        ),
        (
            "runtime_diagnostics",
            "diagnostics",
            vec!["editor.extension.runtime_diagnostics"],
        ),
        (
            "ui_asset_authoring",
            "authoring",
            vec!["editor.extension.ui_asset_authoring"],
        ),
        (
            "native_window_hosting",
            "platform",
            vec!["editor.extension.native_window_hosting"],
        ),
        (
            "editor_build_export_desktop",
            "platform",
            vec![
                "editor.extension.build_export_desktop",
                "editor.extension.build_export_desktop.diagnostics",
                "editor.extension.build_export_desktop.native_dynamic_report",
            ],
        ),
        (
            "plugin_sdk_examples",
            "sdk",
            vec![
                "editor.extension.plugin_sdk_examples",
                "editor.extension.plugin_sdk_examples.window",
                "editor.extension.plugin_sdk_examples.asset_fixture",
            ],
        ),
    ] {
        let static_manifest = read_plugin_manifest(&plugins_root, package_id);
        let catalog_manifest = catalog_manifests
            .iter()
            .find(|manifest| manifest.id == package_id)
            .expect("editor-only package should be in builtin editor catalog");
        let capabilities = capabilities
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(
            static_manifest.category, category,
            "static plugin.toml for `{package_id}` should declare the expected category"
        );
        assert_eq!(
            static_manifest.supported_targets,
            vec![RuntimeTargetMode::EditorHost],
            "static plugin.toml for `{package_id}` should declare editor_host support"
        );
        assert_eq!(
            static_manifest.capabilities, capabilities,
            "static plugin.toml for `{package_id}` should declare package-level editor capabilities"
        );
        assert_eq!(
            catalog_manifest.category, static_manifest.category,
            "builtin editor catalog for `{package_id}` should preserve static category"
        );
        assert_eq!(
            catalog_manifest.supported_targets, static_manifest.supported_targets,
            "builtin editor catalog for `{package_id}` should preserve static supported targets"
        );
        assert_eq!(
            catalog_manifest.capabilities, static_manifest.capabilities,
            "builtin editor catalog for `{package_id}` should preserve static package capabilities"
        );
    }
}

fn plugins_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should have a repository parent")
        .join("zircon_plugins")
}

fn plugin_workspace_members(plugins_root: &Path) -> BTreeSet<String> {
    let manifest = fs::read_to_string(plugins_root.join("Cargo.toml"))
        .expect("zircon_plugins workspace manifest");
    let manifest: toml::Value = toml::from_str(&manifest).expect("workspace manifest should parse");
    manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .expect("workspace members should be an array")
        .iter()
        .map(|member| {
            member
                .as_str()
                .expect("workspace member should be a string")
                .replace('\\', "/")
        })
        .collect()
}

fn plugin_manifest_paths(plugins_root: &Path) -> Vec<PathBuf> {
    fs::read_dir(plugins_root)
        .expect("zircon_plugins directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("plugin.toml"))
        .filter(|path| path.exists())
        .collect()
}

fn read_plugin_manifest(plugins_root: &Path, package_id: &str) -> PluginPackageManifest {
    let manifest_path = plugins_root.join(package_id).join("plugin.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("missing plugin manifest {manifest_path:?}: {error}"));
    toml::from_str(&manifest)
        .unwrap_or_else(|error| panic!("invalid plugin manifest {manifest_path:?}: {error}"))
}
