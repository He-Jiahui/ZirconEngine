use super::{path_aliases, VisualAssetCache, MAX_VISUAL_ASSET_CACHE_ENTRIES};
use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::HostPaintImagePixels;
use std::path::PathBuf;

#[test]
fn changed_source_rejects_only_its_pending_background_product() {
    let changed_source = unique_test_source("pending-changed");
    let stable_source = unique_test_source("pending-stable");
    std::fs::write(&changed_source, b"changed-v1").expect("write changed source");
    std::fs::write(&stable_source, b"stable-v1").expect("write stable source");
    let mut cache = VisualAssetCache::default();
    let changed_clear_epoch = super::visual_asset_cache_epoch();
    let changed_generation = cache.begin_source_load(
        "icon:changed",
        std::slice::from_ref(&changed_source),
        changed_clear_epoch,
    );
    let stable_clear_epoch = super::visual_asset_cache_epoch();
    let stable_generation = cache.begin_source_load(
        "icon:stable",
        std::slice::from_ref(&stable_source),
        stable_clear_epoch,
    );

    std::fs::write(&changed_source, b"changed-v2").expect("change source");
    assert_eq!(
        cache.invalidate_paths(&[changed_source.to_string_lossy().into_owned()]),
        1
    );

    assert!(!cache.source_snapshot_is_current("icon:changed", changed_generation));
    assert!(cache.source_snapshot_is_current("icon:stable", stable_generation));
    cache.finish_source_load("icon:changed", changed_clear_epoch, changed_generation);
    cache.finish_source_load("icon:stable", stable_clear_epoch, stable_generation);
    let _ = std::fs::remove_file(changed_source);
    let _ = std::fs::remove_file(stable_source);
}

#[test]
fn unchanged_source_event_preserves_pending_background_product() {
    let source = unique_test_source("pending-unchanged");
    std::fs::write(&source, b"same-svg-bytes").expect("write source");
    let mut cache = VisualAssetCache::default();
    let clear_epoch = super::visual_asset_cache_epoch();
    let generation =
        cache.begin_source_load("icon:save", std::slice::from_ref(&source), clear_epoch);

    assert_eq!(
        cache.invalidate_paths(&[source.to_string_lossy().into_owned()]),
        0
    );
    assert!(cache.source_snapshot_is_current("icon:save", generation));
    cache.finish_source_load("icon:save", clear_epoch, generation);
    let _ = std::fs::remove_file(source);
}

#[test]
fn changed_source_establishes_the_next_pending_generation_fingerprint() {
    let source = unique_test_source("pending-next-generation");
    std::fs::write(&source, b"svg-v1").expect("write source");
    let mut cache = VisualAssetCache::default();
    let stale_clear_epoch = super::visual_asset_cache_epoch();
    let stale_generation = cache.begin_source_load(
        "icon:save",
        std::slice::from_ref(&source),
        stale_clear_epoch,
    );

    std::fs::write(&source, b"svg-v2").expect("change source");
    assert_eq!(
        cache.invalidate_paths(&[source.to_string_lossy().into_owned()]),
        1
    );
    let next_generation = cache.begin_source_load(
        "icon:save",
        std::slice::from_ref(&source),
        stale_clear_epoch,
    );

    assert_eq!(
        cache.invalidate_paths(&[source.to_string_lossy().into_owned()]),
        0
    );
    assert!(!cache.source_snapshot_is_current("icon:save", stale_generation));
    assert!(cache.source_snapshot_is_current("icon:save", next_generation));
    cache.finish_source_load("icon:save", stale_clear_epoch, stale_generation);
    cache.finish_source_load("icon:save", stale_clear_epoch, next_generation);
    let _ = std::fs::remove_file(source);
}

#[test]
fn stale_completion_after_cache_clear_cannot_release_a_new_generation() {
    let source = unique_test_source("pending-clear-race");
    std::fs::write(&source, b"svg").expect("write source");
    let mut cache = VisualAssetCache::default();
    let old_clear_epoch = super::visual_asset_cache_epoch();
    let old_generation =
        cache.begin_source_load("icon:save", std::slice::from_ref(&source), old_clear_epoch);

    super::advance_visual_asset_cache_epoch();
    cache.clear();
    let new_clear_epoch = super::visual_asset_cache_epoch();
    let new_generation =
        cache.begin_source_load("icon:save", std::slice::from_ref(&source), new_clear_epoch);
    cache.finish_source_load("icon:save", old_clear_epoch, old_generation);
    assert!(cache.pending_base_loads.contains_key(&(
        "icon:save".to_owned(),
        new_clear_epoch,
        new_generation
    )));

    cache.finish_source_load("icon:save", new_clear_epoch, new_generation);
    assert!(cache.pending_base_loads.is_empty());
    let _ = std::fs::remove_file(source);
}

#[test]
fn cache_clear_drops_all_raster_and_source_indices() {
    let mut cache = VisualAssetCache::default();
    cache.insert(
        "icon:save".to_string(),
        "icon:save",
        [PathBuf::from("assets/icons/save.svg")],
        None,
    );

    cache.clear();

    assert!(cache.entries.is_empty());
    assert!(cache.lru_order.is_empty());
    assert!(cache.base_entry_keys.is_empty());
    assert!(cache.source_paths.is_empty());
    assert!(cache.source_fingerprints.is_empty());
    assert!(cache.source_base_keys.is_empty());
    assert!(cache.source_generations.is_empty());
    assert!(cache.pending_base_loads.is_empty());
}

#[test]
fn cache_evicts_the_least_recently_used_entry_at_the_entry_budget() {
    let mut cache = VisualAssetCache::default();
    for index in 0..MAX_VISUAL_ASSET_CACHE_ENTRIES {
        cache.insert(
            format!("resource-{index:03}"),
            &format!("resource-{index:03}"),
            std::iter::empty(),
            None,
        );
    }
    assert!(cache.get("resource-000").is_some());

    cache.insert(
        "resource-new".to_string(),
        "resource-new",
        std::iter::empty(),
        None,
    );

    assert_eq!(cache.entries.len(), MAX_VISUAL_ASSET_CACHE_ENTRIES);
    assert!(cache.get("resource-000").is_some());
    assert!(cache.get("resource-001").is_none());
    assert!(cache.get("resource-new").is_some());
}

#[test]
fn cache_hits_share_the_raster_payload_instead_of_copying_rgba() {
    let mut cache = VisualAssetCache::default();
    cache.insert(
        "icon:save".to_string(),
        "icon:save",
        [PathBuf::from("assets/icons/save.svg")],
        Some(HostPaintImagePixels {
            resource_key: "icon:save".to_string(),
            width: 1,
            height: 1,
            rgba: vec![1, 2, 3, 255].into(),
            atlas: None,
        }),
    );

    let first = cache.get("icon:save").flatten().expect("first cache hit");
    let second = cache.get("icon:save").flatten().expect("second cache hit");

    assert_eq!(first.rgba.as_ptr(), second.rgba.as_ptr());
    assert_eq!(cache.lru_order.len(), cache.entries.len());
}

#[test]
fn path_invalidation_removes_only_dependent_rasters() {
    let save_source = unique_test_source("targeted-save");
    let close_source = unique_test_source("targeted-close");
    std::fs::write(&save_source, b"save-v1").expect("write save source");
    std::fs::write(&close_source, b"close-v1").expect("write close source");
    let mut cache = VisualAssetCache::default();
    cache.insert(
        "icon:save:16".to_string(),
        "icon:save",
        [save_source.clone()],
        None,
    );
    cache.insert(
        "icon:close:16".to_string(),
        "icon:close",
        [close_source.clone()],
        None,
    );
    std::fs::write(&save_source, b"save-v2").expect("change save source");
    assert_eq!(
        cache.invalidate_paths(&[save_source.to_string_lossy().into_owned()]),
        1
    );

    assert!(cache.get("icon:save:16").is_none());
    assert!(cache.get("icon:close:16").is_some());
    let _ = std::fs::remove_file(save_source);
    let _ = std::fs::remove_file(close_source);
}

#[test]
fn unchanged_source_event_preserves_raster_entries() {
    let source = unique_test_source("unchanged");
    std::fs::write(&source, b"same-svg-bytes").expect("write source");
    let mut cache = VisualAssetCache::default();
    cache.insert(
        "icon:save:16".to_string(),
        "icon:save",
        [source.clone()],
        None,
    );

    assert_eq!(
        cache.invalidate_paths(&[source.to_string_lossy().into_owned()]),
        0
    );
    assert!(cache.get("icon:save:16").is_some());
    let _ = std::fs::remove_file(source);
}

#[test]
fn changed_source_event_invalidates_only_that_logical_asset() {
    let source = unique_test_source("changed");
    std::fs::write(&source, b"first-svg-bytes").expect("write source");
    let mut cache = VisualAssetCache::default();
    cache.insert(
        "icon:save:16".to_string(),
        "icon:save",
        [source.clone()],
        None,
    );
    std::fs::write(&source, b"second-svg-bytes").expect("change source");

    assert_eq!(
        cache.invalidate_paths(&[source.to_string_lossy().into_owned()]),
        1
    );
    assert!(cache.get("icon:save:16").is_none());
    let _ = std::fs::remove_file(source);
}

#[test]
fn lag_reconciliation_invalidates_only_sources_whose_content_changed() {
    let changed_source = unique_test_source("lag-changed");
    let stable_source = unique_test_source("lag-stable");
    std::fs::write(&changed_source, b"changed-v1").expect("write changed source");
    std::fs::write(&stable_source, b"stable-v1").expect("write stable source");
    let mut cache = VisualAssetCache::default();
    cache.insert(
        "icon:changed:16".to_string(),
        "icon:changed",
        [changed_source.clone()],
        None,
    );
    cache.insert(
        "icon:stable:16".to_string(),
        "icon:stable",
        [stable_source.clone()],
        None,
    );
    std::fs::write(&changed_source, b"changed-v2").expect("change source");

    assert_eq!(cache.reconcile_source_fingerprints(), 1);
    assert!(cache.get("icon:changed:16").is_none());
    assert!(cache.get("icon:stable:16").is_some());
    let _ = std::fs::remove_file(changed_source);
    let _ = std::fs::remove_file(stable_source);
}

#[test]
fn unchanged_missing_source_event_preserves_the_cached_fallback() {
    let mut cache = VisualAssetCache::default();
    cache.insert(
        "icon:save:missing-source".to_string(),
        "icon:save",
        [PathBuf::from("assets/icons/save.svg")],
        None,
    );
    cache.insert(
        "icon:save:fallback".to_string(),
        "icon:save",
        std::iter::empty(),
        None,
    );

    assert_eq!(
        cache.invalidate_paths(&["assets/icons/save.svg".to_string()]),
        0
    );
    assert!(cache.get("icon:save:missing-source").is_some());
    assert!(cache.get("icon:save:fallback").is_some());
    assert!(cache.source_paths.contains_key("icon:save"));
}

#[test]
fn missing_source_appearing_invalidates_cached_fallback_variants() {
    let source = unique_test_source("appearing");
    let _ = std::fs::remove_file(&source);
    let mut cache = VisualAssetCache::default();
    cache.insert(
        "icon:save:missing-source".to_string(),
        "icon:save",
        [source.clone()],
        None,
    );
    cache.insert(
        "icon:save:fallback".to_string(),
        "icon:save",
        std::iter::empty(),
        None,
    );
    std::fs::write(&source, b"now-present").expect("create source");

    assert_eq!(
        cache.invalidate_paths(&[source.to_string_lossy().into_owned()]),
        1
    );
    assert!(cache.get("icon:save:missing-source").is_none());
    assert!(cache.get("icon:save:fallback").is_none());
    assert!(!cache.source_paths.contains_key("icon:save"));
    let _ = std::fs::remove_file(source);
}

#[test]
fn path_aliases_match_absolute_and_resource_relative_locator_forms() {
    let aliases = path_aliases("E:/project/assets/icons/save.svg");

    assert!(aliases.contains(&"e:/project/assets/icons/save.svg".to_string()));
    assert!(aliases.contains(&"assets/icons/save.svg".to_string()));
    assert!(aliases.contains(&"icons/save.svg".to_string()));
    assert_eq!(path_aliases("res://icons/save.svg")[0], "icons/save.svg");
    assert!(!aliases.contains(&"save.svg".to_string()));
    assert!(!aliases.contains(&"autosave.svg".to_string()));
}

fn unique_test_source(label: &str) -> PathBuf {
    static NEXT_TEST_SOURCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let sequence = NEXT_TEST_SOURCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon-visual-asset-cache-{label}-{}-{sequence}.svg",
        std::process::id()
    ))
}
