use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use super::super::HostPaintImagePixels;
use crate::ui::retained_host::host_contract::chrome_command_stream::invalidate_editor_icon_atlas;

// The byte budget is the primary guard. The entry budget must cover all editor icons and
// their common DPI/tint variants so sequential painting cannot evict the next frame's inputs.
const MAX_VISUAL_ASSET_CACHE_ENTRIES: usize = 4096;
const MAX_VISUAL_ASSET_CACHE_BYTES: usize = 64 * 1024 * 1024;

struct VisualAssetCacheEntry {
    base_key: String,
    pixels: Option<HostPaintImagePixels>,
    byte_size: usize,
    last_used: u64,
}

#[derive(Default)]
struct VisualAssetCache {
    entries: BTreeMap<Arc<str>, VisualAssetCacheEntry>,
    lru_order: BTreeMap<u64, Arc<str>>,
    base_entry_keys: BTreeMap<String, BTreeSet<Arc<str>>>,
    source_paths: BTreeMap<String, BTreeSet<PathBuf>>,
    source_fingerprints: BTreeMap<String, BTreeMap<PathBuf, SourceFingerprint>>,
    source_base_keys: BTreeMap<String, BTreeSet<String>>,
    source_generations: BTreeMap<String, u64>,
    pending_base_loads: BTreeMap<(String, u64, u64), usize>,
    resident_bytes: usize,
    access_clock: u64,
}

pub(super) struct VisualAssetSourceSnapshot {
    clear_epoch: u64,
    base_key: String,
    source_generation: u64,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn cached_visual_asset_pixels(
    key: &str,
) -> Option<Option<HostPaintImagePixels>> {
    let cache = visual_asset_cache();
    zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_lookup");
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(key)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn store_visual_asset_pixels(
    key: String,
    base_key: &str,
    source_paths: impl IntoIterator<Item = PathBuf>,
    pixels: Option<HostPaintImagePixels>,
) {
    let cache = visual_asset_cache();
    zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_store");
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(key, base_key, source_paths, pixels);
}

pub(super) fn begin_visual_asset_source_load(
    base_key: &str,
    source_paths: &[PathBuf],
) -> VisualAssetSourceSnapshot {
    let cache = visual_asset_cache();
    let mut cache = cache.lock().unwrap_or_else(|poison| poison.into_inner());
    let clear_epoch = visual_asset_cache_epoch();
    let source_generation = cache.begin_source_load(base_key, source_paths, clear_epoch);
    VisualAssetSourceSnapshot {
        clear_epoch,
        base_key: base_key.to_owned(),
        source_generation,
    }
}

pub(super) fn finish_visual_asset_source_load(snapshot: VisualAssetSourceSnapshot) {
    let cache = visual_asset_cache();
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .finish_source_load(
            &snapshot.base_key,
            snapshot.clear_epoch,
            snapshot.source_generation,
        );
}

pub(super) fn store_visual_asset_pixels_if_snapshot(
    snapshot: VisualAssetSourceSnapshot,
    key: String,
    source_paths: impl IntoIterator<Item = PathBuf>,
    pixels: Option<HostPaintImagePixels>,
) -> bool {
    let cache = visual_asset_cache();
    let mut cache = cache.lock().unwrap_or_else(|poison| poison.into_inner());
    let is_current = visual_asset_cache_epoch() == snapshot.clear_epoch
        && cache.source_snapshot_is_current(&snapshot.base_key, snapshot.source_generation);
    if is_current {
        cache.insert(key, &snapshot.base_key, source_paths, pixels);
    }
    cache.finish_source_load(
        &snapshot.base_key,
        snapshot.clear_epoch,
        snapshot.source_generation,
    );
    is_current
}

pub(super) fn visual_asset_cache_epoch() -> u64 {
    VISUAL_ASSET_CACHE_EPOCH.load(Ordering::Acquire)
}

fn advance_visual_asset_cache_epoch() {
    VISUAL_ASSET_CACHE_EPOCH.fetch_add(1, Ordering::AcqRel);
}

pub(in crate::ui::retained_host) fn invalidate_visual_asset_pixel_paths(paths: &[String]) -> usize {
    let Some(cache) = VISUAL_ASSET_CACHE.get() else {
        return 0;
    };
    let invalidated = cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .invalidate_paths(paths);
    invalidated
}

pub(in crate::ui::retained_host) fn reconcile_visual_asset_pixel_sources() -> usize {
    let Some(cache) = VISUAL_ASSET_CACHE.get() else {
        return 0;
    };
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .reconcile_source_fingerprints()
}

pub(in crate::ui::retained_host) fn clear_visual_asset_pixels_cache() {
    invalidate_editor_icon_atlas();
    let cache = visual_asset_cache();
    let mut cache = cache.lock().unwrap_or_else(|poison| poison.into_inner());
    advance_visual_asset_cache_epoch();
    cache.clear();
}

impl VisualAssetCache {
    fn clear(&mut self) {
        *self = Self::default();
    }

    fn get(&mut self, key: &str) -> Option<Option<HostPaintImagePixels>> {
        let (cache_key, previous_last_used) = self
            .entries
            .get_key_value(key)
            .map(|(cache_key, entry)| (Arc::clone(cache_key), entry.last_used))?;
        let last_used = self.next_access();
        self.lru_order.remove(&previous_last_used);
        self.lru_order.insert(last_used, cache_key);
        let entry = self
            .entries
            .get_mut(key)
            .expect("visual asset cache entry disappeared during LRU update");
        entry.last_used = last_used;
        Some(entry.pixels.clone())
    }

    fn begin_source_load(
        &mut self,
        base_key: &str,
        source_paths: &[PathBuf],
        clear_epoch: u64,
    ) -> u64 {
        self.track_base_sources(base_key, source_paths.iter().cloned());
        let source_generation = self.source_generation(base_key);
        *self
            .pending_base_loads
            .entry((base_key.to_owned(), clear_epoch, source_generation))
            .or_default() += 1;
        source_generation
    }

    fn finish_source_load(&mut self, base_key: &str, clear_epoch: u64, source_generation: u64) {
        let remove_pending = self
            .pending_base_loads
            .get_mut(&(base_key.to_owned(), clear_epoch, source_generation))
            .is_some_and(|pending| {
                *pending = pending.saturating_sub(1);
                *pending == 0
            });
        if remove_pending {
            self.pending_base_loads
                .remove(&(base_key.to_owned(), clear_epoch, source_generation));
        }
        self.maybe_remove_base_tracking(base_key);
    }

    fn source_snapshot_is_current(&self, base_key: &str, generation: u64) -> bool {
        self.source_generations.get(base_key).copied() == Some(generation)
    }

    fn source_generation(&self, base_key: &str) -> u64 {
        self.source_generations.get(base_key).copied().unwrap_or(1)
    }

    fn advance_source_generation(&mut self, base_key: &str) {
        let generation = self
            .source_generations
            .entry(base_key.to_owned())
            .or_insert(1);
        *generation = generation.saturating_add(1);
    }

    fn track_base_sources(
        &mut self,
        base_key: &str,
        source_paths: impl IntoIterator<Item = PathBuf>,
    ) {
        self.source_generations
            .entry(base_key.to_owned())
            .or_insert(1);
        let source_paths = source_paths.into_iter().collect::<BTreeSet<_>>();
        self.source_paths
            .entry(base_key.to_owned())
            .or_default()
            .extend(source_paths.iter().cloned());
        for source_path in source_paths {
            self.source_fingerprints
                .entry(base_key.to_owned())
                .or_default()
                .entry(source_path.clone())
                .or_insert_with(|| SourceFingerprint::read(&source_path));
            for alias in path_aliases(&source_path.to_string_lossy()) {
                self.source_base_keys
                    .entry(alias)
                    .or_default()
                    .insert(base_key.to_owned());
            }
        }
    }

    fn insert(
        &mut self,
        key: String,
        base_key: &str,
        source_paths: impl IntoIterator<Item = PathBuf>,
        pixels: Option<HostPaintImagePixels>,
    ) {
        self.remove(&key);
        let source_paths = source_paths.into_iter().collect::<BTreeSet<_>>();
        let byte_size = visual_asset_byte_size(&pixels);
        if byte_size > MAX_VISUAL_ASSET_CACHE_BYTES {
            return;
        }
        while !self.entries.is_empty()
            && (self.entries.len() >= MAX_VISUAL_ASSET_CACHE_ENTRIES
                || self.resident_bytes.saturating_add(byte_size) > MAX_VISUAL_ASSET_CACHE_BYTES)
        {
            let Some(evicted_key) = self
                .lru_order
                .first_key_value()
                .map(|(_, key)| Arc::clone(key))
            else {
                break;
            };
            self.remove(&evicted_key);
        }
        self.track_base_sources(base_key, source_paths);
        let last_used = self.next_access();
        let cache_key: Arc<str> = key.into();
        self.resident_bytes = self.resident_bytes.saturating_add(byte_size);
        self.base_entry_keys
            .entry(base_key.to_owned())
            .or_default()
            .insert(Arc::clone(&cache_key));
        self.entries.insert(
            Arc::clone(&cache_key),
            VisualAssetCacheEntry {
                base_key: base_key.to_owned(),
                pixels,
                byte_size,
                last_used,
            },
        );
        self.lru_order.insert(last_used, cache_key);
    }

    fn invalidate_paths(&mut self, paths: &[String]) -> usize {
        let changed_aliases = paths
            .iter()
            .flat_map(|path| path_aliases(path))
            .collect::<BTreeSet<_>>();
        if changed_aliases.is_empty() {
            return 0;
        }
        let affected_base_keys = changed_aliases
            .iter()
            .filter_map(|alias| self.source_base_keys.get(alias))
            .flat_map(|base_keys| base_keys.iter().cloned())
            .collect::<BTreeSet<_>>();
        if affected_base_keys.is_empty() {
            return 0;
        }
        let affected_count = affected_base_keys.len();
        let mut current_fingerprints = BTreeMap::new();
        let changed_base_keys = affected_base_keys
            .into_iter()
            .filter(|base_key| {
                self.base_key_content_changed(base_key, &changed_aliases, &mut current_fingerprints)
            })
            .collect::<BTreeSet<_>>();
        let unchanged_count = affected_count.saturating_sub(changed_base_keys.len());
        if unchanged_count > 0 {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.visual_asset_cache.unchanged_path_event_count",
                unchanged_count
            );
        }
        self.refresh_source_fingerprint_baseline(&changed_base_keys, &current_fingerprints);
        for base_key in &changed_base_keys {
            self.advance_source_generation(base_key);
        }
        let entry_keys = changed_base_keys
            .iter()
            .filter_map(|base_key| self.base_entry_keys.get(base_key))
            .flat_map(|entry_keys| entry_keys.iter().cloned())
            .collect::<Vec<_>>();
        for key in entry_keys {
            self.remove(&key);
        }
        changed_base_keys.len()
    }

    fn reconcile_source_fingerprints(&mut self) -> usize {
        let mut current_fingerprints = BTreeMap::new();
        let changed_base_keys = self
            .source_fingerprints
            .iter()
            .filter_map(|(base_key, source_fingerprints)| {
                let mut changed = false;
                for (path, previous) in source_fingerprints {
                    let current = *current_fingerprints
                        .entry(path.clone())
                        .or_insert_with(|| SourceFingerprint::read(path));
                    changed |= current != *previous;
                }
                changed.then(|| base_key.clone())
            })
            .collect::<BTreeSet<_>>();
        self.refresh_source_fingerprint_baseline(&changed_base_keys, &current_fingerprints);
        for base_key in &changed_base_keys {
            self.advance_source_generation(base_key);
        }
        let entry_keys = changed_base_keys
            .iter()
            .filter_map(|base_key| self.base_entry_keys.get(base_key))
            .flat_map(|entry_keys| entry_keys.iter().cloned())
            .collect::<Vec<_>>();
        for key in entry_keys {
            self.remove(&key);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.asset_refresh.visual_asset_reconcile_source_visit_count",
            current_fingerprints.len()
        );
        changed_base_keys.len()
    }

    fn base_key_content_changed(
        &self,
        base_key: &str,
        changed_aliases: &BTreeSet<String>,
        current_fingerprints: &mut BTreeMap<PathBuf, SourceFingerprint>,
    ) -> bool {
        let Some(source_fingerprints) = self.source_fingerprints.get(base_key) else {
            return true;
        };
        let mut changed = false;
        for (path, previous) in source_fingerprints {
            if path_aliases(&path.to_string_lossy())
                .iter()
                .any(|alias| changed_aliases.contains(alias))
            {
                let current = *current_fingerprints
                    .entry(path.clone())
                    .or_insert_with(|| SourceFingerprint::read(path));
                changed |= current != *previous;
            }
        }
        changed
    }

    fn refresh_source_fingerprint_baseline(
        &mut self,
        base_keys: &BTreeSet<String>,
        current_fingerprints: &BTreeMap<PathBuf, SourceFingerprint>,
    ) {
        for base_key in base_keys {
            let Some(source_fingerprints) = self.source_fingerprints.get_mut(base_key) else {
                continue;
            };
            for (path, current) in current_fingerprints {
                if let Some(previous) = source_fingerprints.get_mut(path) {
                    *previous = *current;
                }
            }
        }
    }

    fn remove(&mut self, key: &str) {
        if let Some(entry) = self.entries.remove(key) {
            self.lru_order.remove(&entry.last_used);
            self.resident_bytes = self.resident_bytes.saturating_sub(entry.byte_size);
            let remove_base_key =
                self.base_entry_keys
                    .get_mut(&entry.base_key)
                    .is_some_and(|entry_keys| {
                        entry_keys.remove(key);
                        entry_keys.is_empty()
                    });
            if remove_base_key {
                self.base_entry_keys.remove(&entry.base_key);
                self.maybe_remove_base_tracking(&entry.base_key);
            }
        }
    }

    fn maybe_remove_base_tracking(&mut self, base_key: &str) {
        if self.base_entry_keys.contains_key(base_key)
            || self
                .pending_base_loads
                .keys()
                .any(|(pending_base_key, _)| pending_base_key == base_key)
        {
            return;
        }
        self.source_fingerprints.remove(base_key);
        self.source_generations.remove(base_key);
        if let Some(source_paths) = self.source_paths.remove(base_key) {
            let aliases = source_paths
                .iter()
                .flat_map(|source_path| path_aliases(&source_path.to_string_lossy()))
                .collect::<BTreeSet<_>>();
            for alias in aliases {
                let remove_alias = self
                    .source_base_keys
                    .get_mut(&alias)
                    .is_some_and(|base_keys| {
                        base_keys.remove(base_key);
                        base_keys.is_empty()
                    });
                if remove_alias {
                    self.source_base_keys.remove(&alias);
                }
            }
        }
    }

    fn next_access(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceFingerprint {
    Missing,
    Content([u8; 32]),
}

impl SourceFingerprint {
    fn read(path: &std::path::Path) -> Self {
        std::fs::read(path).map_or(Self::Missing, |bytes| {
            Self::Content(*blake3::hash(&bytes).as_bytes())
        })
    }
}

fn visual_asset_byte_size(pixels: &Option<HostPaintImagePixels>) -> usize {
    pixels.as_ref().map_or(0, |pixels| {
        pixels.rgba.len().saturating_add(
            pixels
                .atlas
                .as_ref()
                .and_then(|atlas| atlas.rgba.as_ref())
                .map_or(0, Vec::len),
        )
    })
}

fn visual_asset_cache() -> &'static Mutex<VisualAssetCache> {
    VISUAL_ASSET_CACHE.get_or_init(|| Mutex::new(VisualAssetCache::default()))
}

static VISUAL_ASSET_CACHE: OnceLock<Mutex<VisualAssetCache>> = OnceLock::new();
static VISUAL_ASSET_CACHE_EPOCH: AtomicU64 = AtomicU64::new(1);

fn path_aliases(path: &str) -> Vec<String> {
    let path = path.split_once('#').map_or(path, |(path, _)| path).trim();
    if path.is_empty() {
        return Vec::new();
    }
    let mut path = path.replace('\\', "/").to_ascii_lowercase();
    for prefix in ["res://", "asset://", "assets://", "file://"] {
        if let Some(stripped) = path.strip_prefix(prefix) {
            path = stripped.to_owned();
            break;
        }
    }
    let path = path.trim_start_matches('/').to_owned();
    let mut aliases = vec![path.clone()];
    for marker in ["/assets/", "/.zircon/cache/"] {
        if let Some(marker_offset) = path.find(marker) {
            let marker_path = &path[marker_offset + 1..];
            if !aliases.iter().any(|candidate| candidate == marker_path) {
                aliases.push(marker_path.to_owned());
            }
            if marker == "/assets/" {
                let asset_relative = &path[marker_offset + marker.len()..];
                if !aliases.iter().any(|candidate| candidate == asset_relative) {
                    aliases.push(asset_relative.to_owned());
                }
            }
        }
    }
    aliases
}

#[cfg(test)]
mod tests {
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
        static NEXT_TEST_SOURCE: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(1);
        let sequence = NEXT_TEST_SOURCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "zircon-visual-asset-cache-{label}-{}-{sequence}.svg",
            std::process::id()
        ))
    }
}
