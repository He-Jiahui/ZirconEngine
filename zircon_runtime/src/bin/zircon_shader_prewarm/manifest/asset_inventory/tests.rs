use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use zircon_runtime::core::framework::render::{GEOMETRY_SOURCE_ID_STATIC_MESH, ShaderQualityTier};

use super::super::{
    asset_root_manifest_from_inventory_with_resource_registry_revisions,
    asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs,
};
use super::ShaderPrewarmAssetInventory;
use crate::error::ShaderPrewarmAssetScanError;
use zircon_runtime::asset::project::AssetMetaDocument;
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};

#[test]
fn shader_prewarm_inventory_collects_paths_and_meta_candidates_in_stable_order() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_inventory_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("nested")).expect("fixture directory should be created");
    fs::write(root.join("z.wgsl"), "fn z() {}\n").expect("fixture source should be written");
    let mut meta = AssetMetaDocument::new(
        AssetUuid::new(),
        AssetUri::parse("res://a.wgsl").expect("fixture URI should parse"),
        AssetKind::Shader,
    );
    meta.source_digest = "inventory-fixture".to_string();
    meta.save(root.join("a.wgsl.zmeta"))
        .expect("fixture metadata should be written");
    fs::write(root.join("nested/m.zmaterial"), "material\n")
        .expect("fixture material should be written");

    let inventory = ShaderPrewarmAssetInventory::collect(&root)
        .expect("inventory should collect the fixture tree");
    let paths = inventory
        .paths()
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .expect("path should remain in fixture root")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec![
            std::path::Path::new("a.wgsl.zmeta"),
            std::path::Path::new("nested/m.zmaterial"),
            std::path::Path::new("z.wgsl"),
        ]
    );
    assert_eq!(inventory.meta_paths(), &[root.join("a.wgsl.zmeta")]);
    assert_eq!(
        inventory
            .metadata(&root.join("a.wgsl.zmeta"))
            .expect("metadata should be parsed during inventory collection")
            .source_digest,
        "inventory-fixture"
    );

    fs::remove_dir_all(root).expect("fixture directory should be removed");
}

#[test]
fn shader_prewarm_inventory_rejects_directory_links_before_following_them() {
    let root = unique_root("link");
    let outside = unique_root("outside");
    fs::create_dir_all(&root).expect("fixture root should be created");
    fs::create_dir_all(&outside).expect("outside fixture should be created");
    fs::write(outside.join("escaped.wgsl"), "fn escaped() {}\n")
        .expect("outside source should be written");
    let linked = root.join("linked");
    if !create_directory_link(&outside, &linked) {
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
        return;
    }

    let error = ShaderPrewarmAssetInventory::collect(&root)
        .expect_err("inventory must reject directory links");
    assert!(matches!(
        error,
        ShaderPrewarmAssetScanError::UnsafeAssetInventoryLink { path, .. } if path == linked
    ));

    fs::remove_dir_all(root).expect("fixture root should be removed");
    fs::remove_dir_all(outside).expect("outside fixture should be removed");
}

#[test]
fn shader_prewarm_warm_snapshot_index_rejects_replaced_directory_link() {
    let root = unique_root("warm_snapshot_link");
    let snapshot_root = unique_root("warm_snapshot_link_cache");
    let outside = unique_root("warm_snapshot_link_outside");
    let source_directory = root.join("shaders");
    fs::create_dir_all(&source_directory).expect("fixture source directory should be created");
    fs::write(source_directory.join("surface.wgsl"), "fn main() {}\n")
        .expect("fixture source should be written");
    ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("cold inventory should persist a warm snapshot");
    remove_snapshot_directory_entry(
        &snapshot_path_for(&root, &snapshot_root)
            .expect("fixture root should have a payload snapshot path"),
        Path::new("shaders"),
    );
    remove_snapshot_directory_entry(
        &snapshot_index_path_for(&root, &snapshot_root)
            .expect("fixture root should have a compact snapshot path"),
        Path::new("shaders"),
    );

    fs::remove_dir_all(&source_directory).expect("fixture source directory should be removed");
    fs::create_dir_all(&outside).expect("outside fixture directory should be created");
    fs::write(outside.join("surface.wgsl"), "fn escaped() {}\n")
        .expect("outside source should be written");
    if !create_directory_link(&outside, &source_directory) {
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(snapshot_root);
        let _ = fs::remove_dir_all(outside);
        return;
    }

    assert!(
        !ShaderPrewarmAssetInventory::warm_snapshot_is_current_excluding(
            &root,
            &snapshot_root,
            None,
            64 * 1024 * 1024,
        ),
        "the compact index must reject a snapshot missing the linked directory ancestor"
    );
    let error = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect_err("full hydration must reject a snapshot missing the linked directory ancestor");
    assert!(matches!(
        error,
        ShaderPrewarmAssetScanError::UnsafeAssetInventoryLink { path, .. } if path == source_directory
    ));

    fs::remove_dir_all(root).expect("fixture root should be removed");
    fs::remove_dir_all(snapshot_root).expect("fixture snapshot root should be removed");
    fs::remove_dir_all(outside).expect("outside fixture should be removed");
}

#[test]
fn shader_prewarm_inventory_reuses_warm_snapshot_and_invalidates_changed_files() {
    let root = unique_root("warm_snapshot");
    let snapshot_root = unique_root("warm_snapshot_cache");
    fs::create_dir_all(&root).expect("fixture root should be created");
    fs::write(root.join("surface.wgsl"), "fn main() {}\n")
        .expect("fixture source should be written");

    let first = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("cold inventory should write a snapshot");
    assert_eq!(
        first.changed_paths(),
        &BTreeSet::from([root.join("surface.wgsl")]),
        "a cold inventory must schedule every discovered source for prewarm"
    );
    let snapshot_path = snapshot_path_for(&root, &snapshot_root)
        .expect("existing fixture root should have a snapshot path");
    assert!(snapshot_path.exists());
    let warm = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("unchanged inventory should load the warm snapshot");
    assert_eq!(warm.paths(), first.paths());
    assert!(
        warm.changed_paths().is_empty(),
        "an unchanged warm inventory must not schedule redundant prewarm work"
    );
    assert_eq!(
        warm.text(&root.join("surface.wgsl")),
        Some("fn main() {}\n")
    );
    assert!(
        !temporary_snapshot_path(&snapshot_path).exists(),
        "a completed atomic snapshot write must not leave its temporary file behind"
    );

    fs::write(root.join("surface.wgsl"), "fn changed_main() {}\n")
        .expect("changed fixture source should be written");
    let changed = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("changed inventory should refresh the snapshot");
    assert_eq!(changed.paths(), &[root.join("surface.wgsl")]);
    assert_eq!(
        changed.changed_paths(),
        &BTreeSet::from([root.join("surface.wgsl")]),
        "only the changed inventory record should enter the incremental closure"
    );

    fs::remove_dir_all(root).expect("fixture root should be removed");
    fs::remove_dir_all(snapshot_root).expect("snapshot root should be removed");
}

#[test]
fn shader_prewarm_warm_snapshot_index_is_current_without_loading_payload() {
    let root = unique_root("warm_snapshot_index");
    let snapshot_root = unique_root("warm_snapshot_index_cache");
    fs::create_dir_all(&root).expect("fixture root should be created");
    fs::write(root.join("surface.wgsl"), "fn main() {}\n")
        .expect("fixture source should be written");

    ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("cold inventory should persist a warm snapshot");
    let snapshot_path =
        snapshot_path_for(&root, &snapshot_root).expect("fixture root should have a payload path");
    let snapshot_index_path = snapshot_index_path_for(&root, &snapshot_root)
        .expect("fixture root should have a compact index path");
    assert!(snapshot_index_path.exists());

    fs::remove_file(snapshot_path).expect("fixture payload should be removable");
    assert!(
        ShaderPrewarmAssetInventory::warm_snapshot_is_current_excluding(
            &root,
            &snapshot_root,
            None,
            64 * 1024 * 1024,
        ),
        "the unchanged decision must depend on the compact index, not deserialized source text"
    );

    fs::remove_dir_all(root).expect("fixture root should be removed");
    fs::remove_dir_all(snapshot_root).expect("snapshot root should be removed");
}

#[test]
fn shader_prewarm_warm_snapshot_rejects_tampered_relative_paths_and_map_keys() {
    let root = unique_root("warm_snapshot_tampered_path");
    let snapshot_root = unique_root("warm_snapshot_tampered_path_cache");
    let outside = unique_root("warm_snapshot_tampered_path_outside");
    fs::create_dir_all(&root).expect("fixture root should be created");
    fs::create_dir_all(&outside).expect("outside fixture directory should be created");
    fs::write(root.join("surface.wgsl"), "fn root_surface() {}\n")
        .expect("fixture source should be written");
    fs::write(outside.join("surface.wgsl"), "fn outside_surface() {}\n")
        .expect("outside source should be written");
    ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("cold inventory should persist a warm snapshot");

    let snapshot_path =
        snapshot_path_for(&root, &snapshot_root).expect("fixture root should have a payload path");
    let snapshot_index_path = snapshot_index_path_for(&root, &snapshot_root)
        .expect("fixture root should have a compact index path");
    let escaped_relative_path = Path::new("..")
        .join(
            outside
                .file_name()
                .expect("outside fixture must have a name"),
        )
        .join("surface.wgsl");
    let outside_metadata = fs::metadata(outside.join("surface.wgsl"))
        .expect("outside fixture source should have metadata");
    rewrite_snapshot_paths(
        &snapshot_path,
        &escaped_relative_path,
        &outside_metadata,
        true,
    );
    rewrite_snapshot_paths(
        &snapshot_index_path,
        &escaped_relative_path,
        &outside_metadata,
        false,
    );

    assert!(
        !ShaderPrewarmAssetInventory::warm_snapshot_is_current_excluding(
            &root,
            &snapshot_root,
            None,
            64 * 1024 * 1024,
        ),
        "a compact snapshot index must reject a path outside the scanned root"
    );
    let recovered = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("a tampered warm snapshot should fall back to a cold root scan");
    assert_eq!(recovered.paths(), &[root.join("surface.wgsl")]);
    assert_eq!(
        recovered.text(&root.join("surface.wgsl")),
        Some("fn root_surface() {}\n")
    );

    fs::remove_dir_all(root).expect("fixture root should be removed");
    fs::remove_dir_all(snapshot_root).expect("snapshot root should be removed");
    fs::remove_dir_all(outside).expect("outside fixture root should be removed");
}

#[test]
fn shader_prewarm_inventory_excludes_nested_variant_cache_from_warm_fingerprint() {
    let root = unique_root("nested_cache_root");
    let cache_root = root.join(".zircon/cache/shader_variants");
    let snapshot_root = cache_root.join("asset_inventories");
    fs::create_dir_all(&cache_root).expect("nested cache root should be created");
    fs::write(root.join("surface.wgsl"), "fn main() {}\n")
        .expect("fixture source should be written");

    let first = ShaderPrewarmAssetInventory::collect_with_warm_snapshot_excluding(
        &root,
        &snapshot_root,
        Some(&cache_root),
        64 * 1024 * 1024,
    )
    .expect("cold inventory should exclude the variant cache");
    assert_eq!(first.paths(), &[root.join("surface.wgsl")]);

    fs::write(
        cache_root.join("variant-cache-entry.bin"),
        "compiled variant",
    )
    .expect("variant cache artifact should be written");
    let warm = ShaderPrewarmAssetInventory::collect_with_warm_snapshot_excluding(
        &root,
        &snapshot_root,
        Some(&cache_root),
        64 * 1024 * 1024,
    )
    .expect("cache writes must not invalidate the asset inventory snapshot");
    assert_eq!(warm.paths(), &[root.join("surface.wgsl")]);
    assert!(
        warm.changed_paths().is_empty(),
        "variant cache writes must not schedule redundant source prewarm work"
    );

    let full_scan = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("a scan mode change must rebuild instead of reusing an excluded snapshot");
    assert!(
        full_scan
            .paths()
            .iter()
            .any(|path| path == &cache_root.join("variant-cache-entry.bin")),
        "a snapshot written for an excluded cache root must not serve a full asset scan"
    );

    fs::remove_dir_all(root).expect("fixture root should be removed");
}

#[test]
fn shader_prewarm_inventory_rejects_text_above_the_resident_budget() {
    let root = unique_root("text_budget");
    let snapshot_root = unique_root("text_budget_cache");
    fs::create_dir_all(&root).expect("fixture root should be created");
    fs::write(root.join("surface.wgsl"), "fn main() {}\n")
        .expect("fixture source should be written");

    let error = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(&root, &snapshot_root, 1)
        .expect_err("inventory must reject text above the configured resident budget");
    assert!(matches!(
        error,
        ShaderPrewarmAssetScanError::AssetInventoryTextBudgetExceeded { .. }
    ));

    fs::remove_dir_all(root).expect("fixture root should be removed");
    let _ = fs::remove_dir_all(snapshot_root);
}

#[test]
fn shader_prewarm_warm_inventory_emits_only_changed_source_work() {
    let root = unique_root("incremental_manifest");
    let snapshot_root = unique_root("incremental_manifest_cache");
    fs::create_dir_all(&root).expect("fixture root should be created");
    fs::write(root.join("surface.wgsl"), "fn surface() {}\n")
        .expect("fixture source should be written");

    let first = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("cold inventory should be collected");
    let first_manifest = manifest_for_inventory(&root, &first);
    assert_eq!(first_manifest.variants.len(), 6);

    let warm = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("unchanged inventory should be collected");
    let warm_manifest = manifest_for_inventory(&root, &warm);
    assert!(
        warm_manifest.variants.is_empty(),
        "an unchanged warm inventory must not rebuild shader prewarm requests"
    );
    let external_input_manifest = manifest_for_inventory_with_external_inputs(&root, &warm, true);
    assert_eq!(
        external_input_manifest.variants.len(),
        6,
        "a changed external permutation input must rebuild an unchanged warm asset root"
    );

    fs::write(root.join("surface.wgsl"), "fn changed_surface() {}\n")
        .expect("fixture source change should be written");
    let changed = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("changed inventory should be collected");
    let changed_manifest = manifest_for_inventory(&root, &changed);
    assert_eq!(changed_manifest.variants.len(), 6);

    fs::remove_dir_all(root).expect("fixture root should be removed");
    fs::remove_dir_all(snapshot_root).expect("snapshot root should be removed");
}

#[test]
#[ignore = "scale gate: 10k-file cold/warm inventory and one-percent change closure"]
fn shader_prewarm_inventory_scale_10k_cold_warm_and_one_percent_change() {
    const FILE_COUNT: usize = 10_000;
    const CHANGED_FILE_COUNT: usize = FILE_COUNT / 100;

    let root = unique_root("scale_10k");
    let snapshot_root = unique_root("scale_10k_snapshot");
    fs::create_dir_all(&root).expect("scale fixture root should be created");
    for index in 0..FILE_COUNT {
        fs::write(
            root.join(format!("source_{index:05}.wgsl")),
            format!("fn source_{index}() {{}}\n"),
        )
        .expect("scale fixture source should be written");
    }

    let cold = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("cold scale inventory should be collected");
    assert_eq!(cold.paths().len(), FILE_COUNT);
    assert_eq!(cold.changed_paths().len(), FILE_COUNT);

    let warm = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("warm scale inventory should reuse the snapshot");
    assert_eq!(warm.paths().len(), FILE_COUNT);
    assert!(warm.changed_paths().is_empty());

    let changed_paths = (0..CHANGED_FILE_COUNT)
        .map(|index| root.join(format!("source_{index:05}.wgsl")))
        .collect::<BTreeSet<_>>();
    for (index, path) in changed_paths.iter().enumerate() {
        fs::write(
            path,
            format!("fn changed_source_{index}_with_larger_body() {{}}\n"),
        )
        .expect("changed scale fixture source should be written");
    }

    let changed = ShaderPrewarmAssetInventory::collect_with_warm_snapshot(
        &root,
        &snapshot_root,
        64 * 1024 * 1024,
    )
    .expect("one-percent scale change should refresh the snapshot");
    assert_eq!(changed.paths().len(), FILE_COUNT);
    assert_eq!(changed.changed_paths(), &changed_paths);

    fs::remove_dir_all(root).expect("scale fixture root should be removed");
    fs::remove_dir_all(snapshot_root).expect("scale snapshot root should be removed");
}

fn manifest_for_inventory(
    root: &Path,
    inventory: &ShaderPrewarmAssetInventory,
) -> zircon_runtime::core::framework::render::ShaderVariantPrewarmManifest {
    manifest_for_inventory_with_external_inputs(root, inventory, false)
}

fn manifest_for_inventory_with_external_inputs(
    root: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    has_external_permutation_inputs: bool,
) -> zircon_runtime::core::framework::render::ShaderVariantPrewarmManifest {
    if has_external_permutation_inputs {
        return asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs(
            root,
            inventory,
            &[ShaderQualityTier::Medium],
            &[GEOMETRY_SOURCE_ID_STATIC_MESH],
            &Default::default(),
            &Default::default(),
            &Default::default(),
            None,
            true,
        )
        .expect("external inputs should force a fixture prewarm manifest");
    }
    asset_root_manifest_from_inventory_with_resource_registry_revisions(
        root,
        inventory,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &Default::default(),
        &Default::default(),
        &Default::default(),
        None,
    )
    .expect("fixture inventory should produce a prewarm manifest")
}

fn unique_root(suffix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_inventory_{suffix}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos()
    ))
}

fn rewrite_snapshot_paths(
    snapshot_path: &Path,
    escaped_relative_path: &Path,
    outside_metadata: &fs::Metadata,
    include_payload_map_keys: bool,
) {
    let mut snapshot: serde_json::Value = serde_json::from_slice(
        &fs::read(snapshot_path).expect("snapshot fixture should be readable"),
    )
    .expect("snapshot fixture should be valid JSON");
    let relative_path = escaped_relative_path.to_string_lossy().into_owned();
    let modified_nanos: u64 = outside_metadata
        .modified()
        .expect("outside fixture should have a modification time")
        .duration_since(std::time::UNIX_EPOCH)
        .expect("outside fixture timestamp should be after the Unix epoch")
        .as_nanos()
        .try_into()
        .expect("outside fixture timestamp should fit the snapshot representation");
    let entry = snapshot["files"]
        .as_array_mut()
        .and_then(|files| files.first_mut())
        .expect("snapshot fixture should record its source file");
    entry["relative_path"] = serde_json::Value::String(relative_path.clone());
    entry["byte_len"] = serde_json::Value::Number(outside_metadata.len().into());
    entry["modified_nanos"] = serde_json::Value::Number(modified_nanos.into());
    if include_payload_map_keys {
        let map = snapshot["text_by_relative_path"]
            .as_object_mut()
            .expect("payload snapshot should contain source text");
        let source = map
            .remove("surface.wgsl")
            .expect("payload snapshot should retain its recorded source key");
        map.insert(relative_path, source);
        map.insert(
            "untracked.wgsl".to_owned(),
            serde_json::Value::String("fn untracked() {}\n".to_owned()),
        );
    }
    fs::write(
        snapshot_path,
        serde_json::to_vec(&snapshot).expect("snapshot fixture should serialize"),
    )
    .expect("snapshot fixture should be rewritten");
}

fn remove_snapshot_directory_entry(snapshot_path: &Path, relative_path: &Path) {
    let mut snapshot: serde_json::Value = serde_json::from_slice(
        &fs::read(snapshot_path).expect("snapshot fixture should be readable"),
    )
    .expect("snapshot fixture should be valid JSON");
    let directories = snapshot["directories"]
        .as_array_mut()
        .expect("snapshot fixture should record directories");
    let expected_path = relative_path.to_string_lossy();
    let original_count = directories.len();
    directories.retain(|entry| entry["relative_path"].as_str() != Some(expected_path.as_ref()));
    assert!(
        directories.len() < original_count,
        "snapshot fixture should contain the requested directory"
    );
    fs::write(
        snapshot_path,
        serde_json::to_vec(&snapshot).expect("snapshot fixture should serialize"),
    )
    .expect("snapshot fixture should be rewritten");
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) -> bool {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => false,
        Err(error) => panic!("create directory reparse fixture failed: {error}"),
    }
}
