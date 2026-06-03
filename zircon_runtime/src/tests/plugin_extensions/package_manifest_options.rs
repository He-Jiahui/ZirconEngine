use std::fs;
use std::path::{Path, PathBuf};

use crate::plugin::{PluginOptionManifest, PluginPackageManifest};

#[test]
fn static_plugin_manifest_enum_options_roundtrip_runtime_value_sets() {
    let plugins_root = plugins_workspace_root();
    let mut manifest_paths = Vec::new();
    collect_plugin_manifest_paths(&plugins_root, &mut manifest_paths);
    manifest_paths.sort();

    let mut enum_option_count = 0;
    for manifest_path in manifest_paths {
        let source = fs::read_to_string(&manifest_path)
            .unwrap_or_else(|error| panic!("plugin manifest {manifest_path:?}: {error}"));
        let static_value: toml::Value = toml::from_str(&source)
            .unwrap_or_else(|error| panic!("plugin manifest {manifest_path:?}: {error}"));
        let runtime_manifest: PluginPackageManifest = toml::from_str(&source)
            .unwrap_or_else(|error| panic!("plugin manifest {manifest_path:?}: {error}"));
        let relative_path = manifest_path
            .strip_prefix(&plugins_root)
            .unwrap_or(&manifest_path);

        let Some(static_options) = static_value.get("options").and_then(toml::Value::as_array)
        else {
            continue;
        };

        for static_option in static_options {
            let static_option = static_option.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} option should be a table")
            });
            let key = static_option
                .get("key")
                .and_then(toml::Value::as_str)
                .unwrap_or_else(|| {
                    panic!("plugin manifest {relative_path:?} option should declare key")
                });
            let value_type = static_option
                .get("value_type")
                .and_then(toml::Value::as_str)
                .unwrap_or_else(|| {
                    panic!("plugin manifest {relative_path:?} option `{key}` should declare value_type")
                });

            let runtime_option = runtime_manifest
                .options
                .iter()
                .find(|option| option.key.as_str() == key)
                .unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} option `{key}` should deserialize into PluginPackageManifest"
                    )
                });

            if value_type == "enum" {
                enum_option_count += 1;
                let static_enum_values =
                    string_array_values(static_option, relative_path, key, "enum_values");
                assert_eq!(
                    runtime_option.enum_values, static_enum_values,
                    "plugin manifest {relative_path:?} enum option `{key}` should preserve enum_values through PluginPackageManifest deserialization"
                );
            } else {
                assert!(
                    runtime_option.enum_values.is_empty(),
                    "plugin manifest {relative_path:?} non-enum option `{key}` should deserialize with empty enum_values"
                );
            }
        }
    }

    assert!(
        enum_option_count > 0,
        "static plugin manifests should keep at least one enum option to exercise enum_values runtime parsing"
    );
}

#[test]
fn plugin_option_manifest_builder_roundtrips_enum_value_sets() {
    let expected_enum_values = vec![
        "disabled".to_string(),
        "preview".to_string(),
        "balanced".to_string(),
        "cinematic".to_string(),
    ];
    let manifest = PluginPackageManifest::new("sound", "Sound")
        .with_option(
            PluginOptionManifest::new(
                "sound.ray_tracing_quality",
                "Ray tracing quality",
                "enum",
                "balanced",
            )
            .with_enum_values(expected_enum_values.iter().cloned()),
        )
        .with_option(PluginOptionManifest::new(
            "sound.enabled",
            "Enabled",
            "bool",
            "true",
        ));

    let toml_source = toml::to_string(&manifest)
        .expect("PluginPackageManifest with enum option values should serialize to TOML");
    let toml_value: toml::Value = toml::from_str(&toml_source)
        .expect("serialized PluginPackageManifest should parse as TOML");
    let option_rows = toml_value
        .get("options")
        .and_then(toml::Value::as_array)
        .expect("serialized PluginPackageManifest should include option rows");
    let enum_option = option_rows[0]
        .as_table()
        .expect("serialized enum option should be a TOML table");
    assert_eq!(
        string_array_values(
            enum_option,
            Path::new("generated/package_manifest.toml"),
            "sound.ray_tracing_quality",
            "enum_values"
        ),
        expected_enum_values
    );
    assert!(
        option_rows[1]
            .as_table()
            .expect("serialized bool option should be a TOML table")
            .get("enum_values")
            .is_none(),
        "non-enum options should omit empty enum_values when serialized"
    );

    let decoded: PluginPackageManifest = toml::from_str(&toml_source)
        .expect("serialized PluginPackageManifest should deserialize through production manifest");
    assert_eq!(decoded.options[0].enum_values, expected_enum_values);
    assert!(
        decoded.options[1].enum_values.is_empty(),
        "non-enum option should keep the default empty enum_values set after deserialization"
    );
}

fn string_array_values(
    table: &toml::Table,
    relative_path: &Path,
    key: &str,
    field_name: &str,
) -> Vec<String> {
    table
        .get(field_name)
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} option `{key}` should declare `{field_name}`")
        })
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} option `{key}` `{field_name}` entries should be strings"
                    )
                })
                .to_string()
        })
        .collect()
}

fn plugins_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have a repository parent")
        .join("zircon_plugins")
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
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("plugin.toml") {
            manifest_paths.push(path);
        }
    }
}
