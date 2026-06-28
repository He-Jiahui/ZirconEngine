use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;
use zircon_runtime::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceRecord, ResourceState,
};

use super::*;

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
fn shader_resource_records_from_asset_roots_rejects_id_locator_conflicts() {
    let id = ResourceId::from_asset_uuid(uuid("00000000-0000-0000-0000-000000000049"));

    let error = deduplicate_shader_resource_records(vec![
        shader_record(id, "res://shaders/shared_a"),
        shader_record(id, "res://shaders/shared_b"),
    ])
    .unwrap_err();

    assert!(error.contains("maps both res://shaders/shared_a and res://shaders/shared_b"));
}

#[test]
fn shader_resource_records_from_asset_roots_rejects_locator_id_conflicts() {
    let first_id = ResourceId::from_asset_uuid(uuid("00000000-0000-0000-0000-000000000050"));
    let second_id = ResourceId::from_asset_uuid(uuid("00000000-0000-0000-0000-000000000051"));

    let error = deduplicate_shader_resource_records(vec![
        shader_record(first_id, "res://shaders/shared"),
        shader_record(second_id, "res://shaders/shared"),
    ])
    .unwrap_err();

    assert!(error.contains("shader resource registry locator res://shaders/shared maps both"));
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
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000048"
url = "res://shaders/shared"
asset_kind = "Shader"
unit = "single"
source_hash = "source-hash-shared-registry-export"
"#,
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

fn shader_record(id: ResourceId, locator: &str) -> ResourceRecord {
    ResourceRecord::new(
        id,
        ResourceKind::Shader,
        ResourceLocator::parse(locator).unwrap(),
    )
    .with_state(ResourceState::Ready)
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}
