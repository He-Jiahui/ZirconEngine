use std::collections::BTreeMap;
use std::path::Path;

use super::{for_each_static_plugin_manifest, non_empty_string_array_values};

#[test]
fn plugin_tomls_declare_known_supported_platforms() {
    for_each_static_plugin_manifest(|relative_path, table| {
        if table.get("supported_platforms").is_none() {
            return;
        }
        let platforms =
            non_empty_string_array_values(table, relative_path, "top-level", "supported_platforms");
        assert_unique_entries(
            relative_path,
            "top-level",
            "supported_platforms",
            &platforms,
        );

        for platform in platforms {
            assert!(
                matches!(
                    platform,
                    "windows" | "linux" | "macos" | "android" | "ios" | "web_gpu" | "wasm" | "headless"
                ),
                "plugin manifest {relative_path:?} top-level supported platform `{platform}` should be a known export target platform"
            );
        }
    });
}

#[test]
fn plugin_tomls_declare_package_root_arrays() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in ["asset_roots", "content_roots"] {
            let Some(value) = table.get(field_name) else {
                continue;
            };
            let roots = string_array_values(value, relative_path, "top-level", field_name);
            assert_unique_entries(relative_path, "top-level", field_name, &roots);

            for root in roots {
                assert_relative_package_root(relative_path, field_name, root);
            }
        }
    });
}

fn string_array_values<'a>(
    value: &'a toml::Value,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> Vec<&'a str> {
    value
        .as_array()
        .unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} {context} `{field_name}` should be an array")
        })
        .iter()
        .map(|entry| {
            entry.as_str().unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {context} `{field_name}` entries should be strings"
                )
            })
        })
        .inspect(|entry| {
            assert!(
                !entry.is_empty(),
                "plugin manifest {relative_path:?} {context} `{field_name}` entries should not be empty"
            );
        })
        .collect()
}

fn assert_unique_entries(relative_path: &Path, context: &str, field_name: &str, entries: &[&str]) {
    let mut seen = BTreeMap::new();
    for (index, entry) in entries.iter().enumerate() {
        if let Some(previous_index) = seen.insert((*entry).to_string(), index) {
            panic!(
                "plugin manifest {relative_path:?} {context} `{field_name}` entry `{entry}` should be unique; first declared at index {previous_index}, repeated at index {index}"
            );
        }
    }
}

fn assert_relative_package_root(relative_path: &Path, field_name: &str, root: &str) {
    assert_eq!(
        root.trim(),
        root,
        "plugin manifest {relative_path:?} top-level `{field_name}` root `{root}` should not have leading or trailing whitespace"
    );
    assert!(
        !root.starts_with('/') && !root.starts_with('\\'),
        "plugin manifest {relative_path:?} top-level `{field_name}` root `{root}` should be relative"
    );
    assert!(
        !root.contains('\\'),
        "plugin manifest {relative_path:?} top-level `{field_name}` root `{root}` should use forward slashes"
    );
    assert!(
        !root.split('/').any(|segment| segment.is_empty() || segment == "." || segment == ".."),
        "plugin manifest {relative_path:?} top-level `{field_name}` root `{root}` should not contain empty, current, or parent path segments"
    );
}
