use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

use crate::core::editor_plugin::EditorPluginDescriptor;

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
