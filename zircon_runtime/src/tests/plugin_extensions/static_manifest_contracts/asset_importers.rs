use std::collections::BTreeMap;
use std::path::Path;

use super::{
    assert_unique_static_identity, assert_unique_string_array_entries,
    for_each_static_plugin_manifest, integer_value, non_empty_string_array_values,
    non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_asset_importer_rows() {
    let mut importer_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(importers) = table.get("asset_importers") else {
            return;
        };
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let importers = importers.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} asset_importers should be an array")
        });
        assert!(
            !importers.is_empty(),
            "plugin manifest {relative_path:?} asset_importers should not be empty when declared"
        );

        for importer in importers {
            let importer = importer.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} asset importer should be a table")
            });
            let importer_id =
                non_empty_string_value(importer, relative_path, "asset importer", "id");
            let importer_context = format!("asset importer `{importer_id}`");
            assert_unique_static_identity(
                &mut importer_ids,
                importer_id,
                format!("{importer_context} in {}", relative_path.display()),
            );

            let plugin_id =
                non_empty_string_value(importer, relative_path, &importer_context, "plugin_id");
            assert_eq!(
                plugin_id, package_id,
                "plugin manifest {relative_path:?} {importer_context} plugin_id should match package id `{package_id}`"
            );

            let priority = integer_value(importer, relative_path, &importer_context, "priority");
            assert!(
                priority >= i64::from(i32::MIN) && priority <= i64::from(i32::MAX),
                "plugin manifest {relative_path:?} {importer_context} priority `{priority}` should fit i32"
            );
            let importer_version = integer_value(
                importer,
                relative_path,
                &importer_context,
                "importer_version",
            );
            assert!(
                importer_version > 0 && importer_version <= i64::from(u32::MAX),
                "plugin manifest {relative_path:?} {importer_context} importer_version `{importer_version}` should be a positive u32"
            );

            assert_known_asset_kind(importer, relative_path, &importer_context, "output_kind");
            assert_asset_importer_source_selectors(importer, relative_path, &importer_context);
            assert_unique_string_array_entries(
                importer,
                relative_path,
                &importer_context,
                "required_capabilities",
            );

            if importer.get("additional_output_kinds").is_some() {
                assert_unique_string_array_entries(
                    importer,
                    relative_path,
                    &importer_context,
                    "additional_output_kinds",
                );
                for output_kind in non_empty_string_array_values(
                    importer,
                    relative_path,
                    &importer_context,
                    "additional_output_kinds",
                ) {
                    assert_known_asset_kind_value(
                        output_kind,
                        relative_path,
                        &importer_context,
                        "additional_output_kinds",
                    );
                }
            }
        }
    });
}

#[test]
fn plugin_tomls_declare_asset_importer_ids_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(importers) = table.get("asset_importers") else {
            return;
        };
        let importers = importers.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} asset_importers should be an array")
        });

        for importer in importers {
            let importer = importer.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} asset importer should be a table")
            });
            let importer_id =
                non_empty_string_value(importer, relative_path, "asset importer", "id");
            assert_dot_namespaced_importer_id(relative_path, importer_id);
        }
    });
}

fn assert_known_asset_kind(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    let asset_kind = non_empty_string_value(table, relative_path, context, field_name);
    assert_known_asset_kind_value(asset_kind, relative_path, context, field_name);
}

fn assert_known_asset_kind_value(
    asset_kind: &str,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    assert!(
        matches!(
            asset_kind,
            "Data"
                | "Model"
                | "Mesh"
                | "Material"
                | "MaterialGraph"
                | "Texture"
                | "Shader"
                | "Scene"
                | "Sound"
                | "Font"
                | "PhysicsMaterial"
                | "NavMesh"
                | "NavigationSettings"
                | "Terrain"
                | "TerrainLayerStack"
                | "TileSet"
                | "TileMap"
                | "Prefab"
                | "AnimationSkeleton"
                | "AnimationClip"
                | "AnimationSequence"
                | "AnimationGraph"
                | "AnimationStateMachine"
                | "UiLayout"
                | "UiWidget"
                | "UiStyle"
        ),
        "plugin manifest {relative_path:?} {context} `{field_name}` asset kind `{asset_kind}` should be a known ResourceKind"
    );
}

fn assert_dot_namespaced_importer_id(relative_path: &Path, importer_id: &str) {
    assert_eq!(
        importer_id.trim(),
        importer_id,
        "plugin manifest {relative_path:?} asset importer id `{importer_id}` should not have leading or trailing whitespace"
    );

    let segments: Vec<_> = importer_id.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} asset importer id `{importer_id}` should use at least two dot-separated namespace segments"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} asset importer id `{importer_id}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} asset importer id `{importer_id}` should contain only lowercase ASCII letters, digits, underscores, and dots"
        );
    }
}

fn assert_asset_importer_source_selectors(
    importer: &toml::Table,
    relative_path: &Path,
    importer_context: &str,
) {
    let has_source_extensions = importer.get("source_extensions").is_some();
    let has_full_suffixes = importer.get("full_suffixes").is_some();
    assert!(
        has_source_extensions || has_full_suffixes,
        "plugin manifest {relative_path:?} {importer_context} should declare source_extensions or full_suffixes"
    );

    if has_source_extensions {
        assert_unique_string_array_entries(
            importer,
            relative_path,
            importer_context,
            "source_extensions",
        );
        for extension in non_empty_string_array_values(
            importer,
            relative_path,
            importer_context,
            "source_extensions",
        ) {
            let normalized = extension
                .trim()
                .trim_start_matches('.')
                .to_ascii_lowercase();
            assert_eq!(
                extension, normalized,
                "plugin manifest {relative_path:?} {importer_context} source extension `{extension}` should be lowercase without a leading dot"
            );
        }
    }

    if has_full_suffixes {
        assert_unique_string_array_entries(
            importer,
            relative_path,
            importer_context,
            "full_suffixes",
        );
        for suffix in non_empty_string_array_values(
            importer,
            relative_path,
            importer_context,
            "full_suffixes",
        ) {
            let normalized = if suffix.trim().starts_with('.') {
                suffix.trim().to_ascii_lowercase()
            } else {
                format!(".{}", suffix.trim().to_ascii_lowercase())
            };
            assert_eq!(
                suffix, normalized,
                "plugin manifest {relative_path:?} {importer_context} full suffix `{suffix}` should be lowercase and include the leading dot"
            );
            assert!(
                suffix.len() > 1,
                "plugin manifest {relative_path:?} {importer_context} full suffix `{suffix}` should include a suffix after the dot"
            );
        }
    }
}
