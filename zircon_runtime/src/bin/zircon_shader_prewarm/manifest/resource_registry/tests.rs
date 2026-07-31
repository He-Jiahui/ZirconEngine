use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::resource::{
    AssetUuid, ResourceId, ResourceKind, ResourceLocator, ResourceRecord, ResourceState,
};

use crate::error::ShaderPrewarmResourceRegistryError;

use super::*;

#[test]
fn nested_resource_arrays_are_moved_out_of_the_json_document() {
    let source = include_str!("../resource_registry.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let function = source
        .split("fn resource_records_from_json_value")
        .nth(1)
        .unwrap()
        .split("impl From<")
        .next()
        .unwrap();

    assert!(function.contains(".remove(\"resources\")"));
    assert!(function.contains(".remove(\"records\")"));
    assert!(!function.contains("records.clone()"));
}

#[test]
fn shader_prewarm_resource_registry_read_reports_typed_read_error() {
    let missing_path = unique_root("zircon_shader_prewarm_missing_resource_registry")
        .join("shader_resource_records.json");

    let error = ShaderPrewarmResourceRegistryOverlay::read(&missing_path).unwrap_err();

    match error {
        ShaderPrewarmResourceRegistryError::Read { path, source: _ } => {
            assert_eq!(path, missing_path);
        }
        other => panic!("expected typed resource registry read error, got {other:?}"),
    }
}

#[test]
fn shader_prewarm_resource_registry_read_reports_typed_parse_error() {
    let root = unique_root("zircon_shader_prewarm_invalid_resource_registry");
    fs::create_dir_all(&root).unwrap();
    let registry_path = root.join("shader_resource_records.json");
    fs::write(&registry_path, "{not json").unwrap();

    let error = ShaderPrewarmResourceRegistryOverlay::read(&registry_path).unwrap_err();

    match error {
        ShaderPrewarmResourceRegistryError::Parse { path, source: _ } => {
            assert_eq!(path, registry_path);
        }
        other => panic!("expected typed resource registry parse error, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_resource_registry_read_reports_typed_decode_error() {
    let root = unique_root("zircon_shader_prewarm_bad_record_resource_registry");
    fs::create_dir_all(&root).unwrap();
    let registry_path = root.join("shader_resource_records.json");
    fs::write(&registry_path, r#"{"resources":[{"kind":"Shader"}]}"#).unwrap();

    let error = ShaderPrewarmResourceRegistryOverlay::read(&registry_path).unwrap_err();

    match error {
        ShaderPrewarmResourceRegistryError::DecodeRecords { path, source: _ } => {
            assert_eq!(path, registry_path);
        }
        other => panic!("expected typed resource registry decode error, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_resource_records_from_asset_roots_deduplicates_duplicate_shader_records() {
    let root = unique_root("zircon_shader_prewarm_multi_root_registry_export");
    let engine_root = root.join("engine_assets");
    let plugin_root = root.join("plugin_assets");
    write_shader_with_meta(&engine_root);
    write_shader_with_meta(&plugin_root);

    let records =
        shader_resource_records_from_asset_roots(&[engine_root.clone(), plugin_root.clone()])
            .unwrap();

    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.kind, ResourceKind::Shader);
    assert_eq!(record.state, ResourceState::Ready);
    assert_eq!(
        record.primary_locator,
        ResourceLocator::parse("res://shaders/shared").unwrap()
    );
    assert_ne!(record.revision, 0);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_resource_records_from_project_and_plugin_asset_roots_export_distinct_shader_sources() {
    let root = unique_root("zircon_shader_prewarm_project_plugin_registry_export");
    let project_root = root.join("project_assets");
    let plugin_root = root.join("plugin_assets");
    write_named_shader_with_meta(
        &project_root,
        "project",
        "00000000-0000-0000-0000-000000000052",
        "res://project/shaders/project",
        "source-hash-project-registry-export",
    );
    write_named_shader_with_meta(
        &plugin_root,
        "plugin",
        "00000000-0000-0000-0000-000000000053",
        "package://virtual_geometry/shaders/plugin",
        "source-hash-plugin-registry-export",
    );

    let records =
        shader_resource_records_from_asset_roots(&[project_root.clone(), plugin_root.clone()])
            .unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(
        records
            .iter()
            .map(|record| record.primary_locator.to_string())
            .collect::<Vec<_>>(),
        vec![
            "package://virtual_geometry/shaders/plugin".to_string(),
            "res://project/shaders/project".to_string()
        ]
    );
    assert!(records.iter().all(|record| {
        record.kind == ResourceKind::Shader
            && record.state == ResourceState::Ready
            && record.revision != 0
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_resource_records_from_asset_roots_rejects_id_locator_conflicts() {
    let id = ResourceId::from_asset_uuid(uuid("00000000-0000-0000-0000-000000000049"));
    let root = unique_root("zircon_shader_prewarm_id_locator_conflict");
    let first_root = root.join("first_assets");
    let second_root = root.join("second_assets");
    write_named_shader_with_meta(
        &first_root,
        "shared_a",
        "00000000-0000-0000-0000-000000000049",
        "res://shaders/shared_a",
        "source-hash-shared-a",
    );
    write_named_shader_with_meta(
        &second_root,
        "shared_b",
        "00000000-0000-0000-0000-000000000049",
        "res://shaders/shared_b",
        "source-hash-shared-b",
    );

    let error = shader_resource_records_from_asset_roots(&[first_root, second_root]).unwrap_err();

    match error {
        ShaderPrewarmResourceRegistryError::DuplicateRecordId {
            id: actual_id,
            existing_locator,
            new_locator,
        } => {
            assert_eq!(actual_id, id);
            assert_eq!(
                existing_locator,
                ResourceLocator::parse("res://shaders/shared_a").unwrap()
            );
            assert_eq!(
                new_locator,
                ResourceLocator::parse("res://shaders/shared_b").unwrap()
            );
        }
        other => panic!("expected typed duplicate resource id error, got {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_resource_records_from_asset_root_rejects_id_locator_conflicts() {
    let id = ResourceId::from_asset_uuid(uuid("00000000-0000-0000-0000-000000000054"));
    let root = unique_root("zircon_shader_prewarm_single_root_id_locator_conflict");
    write_named_shader_with_meta(
        &root,
        "shared_a",
        "00000000-0000-0000-0000-000000000054",
        "res://shaders/shared_a",
        "source-hash-single-root-shared-a",
    );
    write_named_shader_with_meta(
        &root,
        "shared_b",
        "00000000-0000-0000-0000-000000000054",
        "res://shaders/shared_b",
        "source-hash-single-root-shared-b",
    );

    let error = shader_resource_records_from_asset_root(&root).unwrap_err();

    match error {
        ShaderPrewarmResourceRegistryError::DuplicateRecordId {
            id: actual_id,
            existing_locator,
            new_locator,
        } => {
            assert_eq!(actual_id, id);
            assert_eq!(
                existing_locator,
                ResourceLocator::parse("res://shaders/shared_a").unwrap()
            );
            assert_eq!(
                new_locator,
                ResourceLocator::parse("res://shaders/shared_b").unwrap()
            );
        }
        other => panic!("expected typed duplicate resource id error, got {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_resource_records_from_asset_roots_rejects_locator_id_conflicts() {
    let first_id = ResourceId::from_asset_uuid(uuid("00000000-0000-0000-0000-000000000050"));
    let second_id = ResourceId::from_asset_uuid(uuid("00000000-0000-0000-0000-000000000051"));
    let root = unique_root("zircon_shader_prewarm_locator_id_conflict");
    let first_root = root.join("first_assets");
    let second_root = root.join("second_assets");
    write_named_shader_with_meta(
        &first_root,
        "shared",
        "00000000-0000-0000-0000-000000000050",
        "res://shaders/shared",
        "source-hash-shared-first",
    );
    write_named_shader_with_meta(
        &second_root,
        "shared",
        "00000000-0000-0000-0000-000000000051",
        "res://shaders/shared",
        "source-hash-shared-second",
    );

    let error = shader_resource_records_from_asset_roots(&[first_root, second_root]).unwrap_err();

    match error {
        ShaderPrewarmResourceRegistryError::DuplicateLocator {
            locator,
            existing_id,
            new_id,
        } => {
            assert_eq!(
                locator,
                ResourceLocator::parse("res://shaders/shared").unwrap()
            );
            assert_eq!(existing_id, first_id);
            assert_eq!(new_id, second_id);
        }
        other => panic!("expected typed duplicate locator error, got {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only() {
    let locator = ResourceLocator::parse("res://shaders/example").unwrap();
    let shader_id = ResourceId::from_locator(&locator);
    let mut error_record = ResourceRecord::new(shader_id, ResourceKind::Shader, locator.clone())
        .with_state(ResourceState::Error);
    error_record.revision = 99;

    let error_only_overlay =
        ShaderPrewarmResourceRegistryOverlay::from_records([error_record.clone()]);
    assert_eq!(
        error_only_overlay.revision_for(shader_id, "res://shaders/example"),
        None
    );

    let mut ready_record = ResourceRecord::new(shader_id, ResourceKind::Shader, locator)
        .with_state(ResourceState::Ready);
    ready_record.revision = 77;

    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records([error_record, ready_record]);

    assert_eq!(
        overlay.revision_for(shader_id, "res://shaders/example"),
        Some(77)
    );
}

fn write_shader_with_meta(asset_root: &Path) {
    fs::create_dir_all(asset_root.join("shaders")).unwrap();
    fs::write(asset_root.join("shaders/shared.wgsl"), "fn shared() {}\n").unwrap();
    fs::write(
        asset_root.join("shaders/shared.wgsl.zmeta"),
        r#"format_version = 7
uuid = "00000000-0000-0000-0000-000000000048"
url = "res://shaders/shared"
asset_kind = "Shader"
unit = "single"
source_digest = "source-hash-shared-registry-export"
"#,
    )
    .unwrap();
}

fn write_named_shader_with_meta(
    asset_root: &Path,
    name: &str,
    id: &str,
    locator: &str,
    source_hash: &str,
) {
    fs::create_dir_all(asset_root.join("shaders")).unwrap();
    fs::write(
        asset_root.join("shaders").join(format!("{name}.wgsl")),
        format!("fn {name}() {{}}\n"),
    )
    .unwrap();
    fs::write(
        asset_root
            .join("shaders")
            .join(format!("{name}.wgsl.zmeta")),
        format!(
            r#"format_version = 7
uuid = "{id}"
url = "{locator}"
asset_kind = "Shader"
unit = "single"
source_digest = "{source_hash}"
"#
        ),
    )
    .unwrap();
}

fn unique_root(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
}

fn uuid(value: &str) -> AssetUuid {
    value.parse().unwrap()
}
