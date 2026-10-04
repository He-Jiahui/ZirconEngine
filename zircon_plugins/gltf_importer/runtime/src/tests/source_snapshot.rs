use super::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use zircon_runtime::asset::{AssetImportContext, AssetUri, ImportedAsset};

#[test]
fn importer_reserves_root_and_mesh_outputs_before_projection() {
    let mut roots = Vec::new();
    let mesh_outputs = reserve_gltf_mesh_outputs(&mut roots, 3);

    assert!(roots.capacity() >= 3);
    assert!(mesh_outputs.capacity() >= 3);
    assert!(mesh_outputs.is_empty());
}

#[test]
fn importer_uses_embedded_source_snapshot_without_reopening_path() {
    let source_path = PathBuf::from("target/zircon-gltf-importer/missing/embedded.gltf");
    let source_bytes = br#"{
        "asset": { "version": "2.0" },
        "buffers": [{
            "uri": "data:application/octet-stream;base64,AAAAAA==",
            "byteLength": 4
        }]
    }"#
    .to_vec();
    let context = AssetImportContext::new(
        source_path,
        AssetUri::parse("res://models/embedded.gltf").unwrap(),
        source_bytes,
        toml::Table::new(),
    );

    let outcome = import_gltf(&context)
        .expect("embedded glTF must import from the transaction-owned source bytes");

    assert!(matches!(
        outcome.root_entry().map(|entry| &entry.asset),
        Some(ImportedAsset::Model(_))
    ));
}

#[test]
fn importer_uses_admitted_external_buffer_and_image_after_disk_replacement() {
    let root = std::env::temp_dir().join(format!(
        "zircon-plugin-gltf-snapshot-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let model_dir = root.join("assets/models");
    fs::create_dir_all(&model_dir).unwrap();
    let source_path = crate::test_fixtures::write_external_texture_gltf(&model_dir);
    let buffer_path = model_dir.join("external_texture.bin");
    let image_path = model_dir.join("external_albedo.png");
    let snapshots = BTreeMap::from([
        (buffer_path.clone(), fs::read(&buffer_path).unwrap()),
        (image_path.clone(), fs::read(&image_path).unwrap()),
    ]);
    fs::remove_file(&buffer_path).unwrap();
    fs::write(&image_path, b"replaced image on disk").unwrap();

    let context = AssetImportContext::new(
        source_path.clone(),
        AssetUri::parse("res://models/external_texture.gltf").unwrap(),
        fs::read(&source_path).unwrap(),
        toml::Table::new(),
    )
    .with_source_file_snapshots(snapshots);
    let outcome = import_gltf(&context).expect("both companion files must use admitted bytes");
    assert!(outcome.entries.iter().any(|entry| {
        matches!(&entry.asset, ImportedAsset::Texture(texture) if texture.width == 1 && texture.height == 1)
    }));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn importer_rejects_missing_authoritative_companion_without_disk_fallback() {
    let root = std::env::temp_dir().join(format!(
        "zircon-plugin-gltf-missing-snapshot-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let model_dir = root.join("assets/models");
    fs::create_dir_all(&model_dir).unwrap();
    let source_path = crate::test_fixtures::write_external_texture_gltf(&model_dir);
    let context = AssetImportContext::new(
        source_path.clone(),
        AssetUri::parse("res://models/external_texture.gltf").unwrap(),
        fs::read(&source_path).unwrap(),
        toml::Table::new(),
    )
    .with_source_file_snapshots(BTreeMap::new());

    let error = import_gltf(&context).expect_err("authoritative omissions must not read disk");
    assert!(
        error.to_string().contains("snapshot does not contain"),
        "{error}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn importer_rejects_external_buffer_uri_outside_asset_root() {
    let root = std::env::temp_dir().join(format!(
        "zircon-plugin-gltf-escape-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let model_dir = root.join("assets/models");
    fs::create_dir_all(&model_dir).unwrap();
    let context = AssetImportContext::new(
        model_dir.join("escape.gltf"),
        AssetUri::parse("res://models/escape.gltf").unwrap(),
        br#"{"asset":{"version":"2.0"},"buffers":[{"uri":"..%2F..%2Foutside.bin","byteLength":4}]}"#.to_vec(),
        toml::Table::new(),
    );

    let error = import_gltf(&context).expect_err("encoded root escape must be rejected");
    assert!(error.to_string().contains("escapes asset root"), "{error}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn importer_rejects_required_extensions_outside_its_stable_policy() {
    let context = AssetImportContext::new(
        PathBuf::from("target/zircon-gltf-importer/missing/extension.gltf"),
        AssetUri::parse("res://models/extension.gltf").unwrap(),
        br#"{"asset":{"version":"2.0"},"extensionsUsed":["EXT_meshopt_compression"],"extensionsRequired":["EXT_meshopt_compression"]}"#.to_vec(),
        toml::Table::new(),
    );

    let error = import_gltf(&context).expect_err("plugin must retain its narrower policy");
    assert!(
        error
            .to_string()
            .contains("requires unsupported extension `EXT_meshopt_compression`"),
        "{error}"
    );
}
