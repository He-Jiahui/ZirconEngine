use std::fs;
use std::path::{Path, PathBuf};

use super::plugins_workspace_root;

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn for_each_static_plugin_manifest(
    mut visit: impl FnMut(&Path, &toml::Table),
) {
    let plugins_root = plugins_workspace_root();
    let mut manifest_paths = Vec::new();
    collect_plugin_manifest_paths(&plugins_root, &mut manifest_paths);
    manifest_paths.sort();

    assert!(
        !manifest_paths.is_empty(),
        "zircon_plugins workspace should contain package manifests"
    );

    for manifest_path in manifest_paths {
        let source = fs::read_to_string(&manifest_path)
            .unwrap_or_else(|error| panic!("plugin manifest {manifest_path:?}: {error}"));
        let value: toml::Value = toml::from_str(&source)
            .unwrap_or_else(|error| panic!("plugin manifest {manifest_path:?}: {error}"));
        let table = value
            .as_table()
            .unwrap_or_else(|| panic!("plugin manifest {manifest_path:?} should be a table"));
        let relative_path = manifest_path
            .strip_prefix(&plugins_root)
            .unwrap_or(&manifest_path);

        visit(relative_path, table);
    }
}

fn collect_plugin_manifest_paths(root: &Path, manifest_paths: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("read plugin directory {root:?}: {error}"))
    {
        let entry =
            entry.unwrap_or_else(|error| panic!("read plugin directory entry {root:?}: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_plugin_manifest_paths(&path, manifest_paths);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("plugin.toml") {
            manifest_paths.push(path);
        }
    }
}
