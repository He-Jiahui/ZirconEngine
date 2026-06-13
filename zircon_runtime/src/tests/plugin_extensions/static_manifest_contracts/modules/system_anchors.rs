use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::super::{
    for_each_module_row, for_each_static_plugin_manifest, non_empty_string_value,
    plugins_workspace_root,
};

#[test]
fn declared_system_anchors_are_registered() {
    let crate_index = PluginCrateIndex::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_kind = non_empty_string_value(module, relative_path, module_context, "kind");
            if module_kind != "runtime" {
                return;
            }

            let Some(anchors) = optional_string_array(module, "system_anchors") else {
                return;
            };
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let crate_name =
                non_empty_string_value(module, relative_path, module_context, "crate_name");
            let source = crate_index.source_for(crate_name);

            assert!(
                source.contains("register_runtime_system")
                    || source.contains("register_native_system"),
                "plugin manifest {relative_path:?} {module_context} module `{module_name}` declares system anchors but crate `{crate_name}` does not expose a runtime system registration path"
            );

            for anchor in anchors {
                assert!(
                    source.contains(anchor),
                    "plugin manifest {relative_path:?} {module_context} module `{module_name}` declares system anchor `{anchor}` but crate `{crate_name}` source does not reference that registered system id"
                );
            }
        });
    });
}

fn optional_string_array<'a>(table: &'a toml::Table, field_name: &str) -> Option<Vec<&'a str>> {
    let values = table
        .get(field_name)?
        .as_array()
        .unwrap_or_else(|| panic!("manifest field `{field_name}` should be an array when present"));
    assert!(
        !values.is_empty(),
        "manifest field `{field_name}` should not be empty when present"
    );
    Some(
        values
            .iter()
            .map(|value| {
                value.as_str().unwrap_or_else(|| {
                    panic!("manifest field `{field_name}` entries should be strings")
                })
            })
            .collect(),
    )
}

struct PluginCrateIndex {
    crates: BTreeMap<String, PathBuf>,
}

impl PluginCrateIndex {
    fn new() -> Self {
        let root = plugins_workspace_root();
        let mut crates = BTreeMap::new();
        collect_crates(&root, &mut crates);
        Self { crates }
    }

    fn source_for(&self, crate_name: &str) -> String {
        let crate_root = self.crates.get(crate_name).unwrap_or_else(|| {
            panic!("zircon_plugins crate `{crate_name}` should be discoverable from Cargo.toml")
        });
        let mut source = String::new();
        collect_rust_source(&crate_root.join("src"), &mut source);
        source
    }
}

fn collect_crates(root: &Path, crates: &mut BTreeMap<String, PathBuf>) {
    for entry in fs::read_dir(root).unwrap_or_else(|error| panic!("read {root:?}: {error}")) {
        let entry = entry.unwrap_or_else(|error| panic!("read entry under {root:?}: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_crates(&path, crates);
            continue;
        }

        if path.file_name().and_then(|name| name.to_str()) != Some("Cargo.toml") {
            continue;
        }

        let manifest = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read crate manifest {path:?}: {error}"));
        let manifest: toml::Value = toml::from_str(&manifest)
            .unwrap_or_else(|error| panic!("parse crate manifest {path:?}: {error}"));
        let Some(package_name) = manifest
            .get("package")
            .and_then(toml::Value::as_table)
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str)
        else {
            continue;
        };
        let crate_root = path
            .parent()
            .unwrap_or_else(|| panic!("crate manifest {path:?} should have a parent"))
            .to_path_buf();
        crates.insert(package_name.to_string(), crate_root);
    }
}

fn collect_rust_source(root: &Path, source: &mut String) {
    for entry in fs::read_dir(root).unwrap_or_else(|error| panic!("read {root:?}: {error}")) {
        let entry = entry.unwrap_or_else(|error| panic!("read entry under {root:?}: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source(&path, source);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let file_source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read Rust source {path:?}: {error}"));
            source.push_str(&file_source);
            source.push('\n');
        }
    }
}
