use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

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
    entries: BTreeMap<String, VisualAssetCacheEntry>,
    base_entry_keys: BTreeMap<String, BTreeSet<String>>,
    source_paths: BTreeMap<String, BTreeSet<PathBuf>>,
    source_fingerprints: BTreeMap<String, BTreeMap<PathBuf, SourceFingerprint>>,
    source_base_keys: BTreeMap<String, BTreeSet<String>>,
    resident_bytes: usize,
    access_clock: u64,
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
    if let Some(cache) = VISUAL_ASSET_CACHE.get() {
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clear();
    }
}

impl VisualAssetCache {
    fn clear(&mut self) {
        *self = Self::default();
    }

    fn get(&mut self, key: &str) -> Option<Option<HostPaintImagePixels>> {
        let last_used = self.next_access();
        self.entries.get_mut(key).map(|entry| {
            entry.last_used = last_used;
            entry.pixels.clone()
        })
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
                .entries
                .iter()
                .min_by_key(|(key, entry)| (entry.last_used, *key))
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.remove(&evicted_key);
        }
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
        let last_used = self.next_access();
        self.resident_bytes = self.resident_bytes.saturating_add(byte_size);
        self.base_entry_keys
            .entry(base_key.to_owned())
            .or_default()
            .insert(key.clone());
        self.entries.insert(
            key,
            VisualAssetCacheEntry {
                base_key: base_key.to_owned(),
                pixels,
                byte_size,
                last_used,
            },
        );
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
                source_fingerprints
                    .iter()
                    .any(|(path, previous)| {
                        *current_fingerprints
                            .entry(path.clone())
                            .or_insert_with(|| SourceFingerprint::read(path))
                            != *previous
                    })
                    .then(|| base_key.clone())
            })
            .collect::<BTreeSet<_>>();
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
        source_fingerprints.iter().any(|(path, previous)| {
            path_aliases(&path.to_string_lossy())
                .iter()
                .any(|alias| changed_aliases.contains(alias))
                && *current_fingerprints
                    .entry(path.clone())
                    .or_insert_with(|| SourceFingerprint::read(path))
                    != *previous
        })
    }

    fn remove(&mut self, key: &str) {
        if let Some(entry) = self.entries.remove(key) {
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
                self.source_fingerprints.remove(&entry.base_key);
                if let Some(source_paths) = self.source_paths.remove(&entry.base_key) {
                    let aliases = source_paths
                        .iter()
                        .flat_map(|source_path| path_aliases(&source_path.to_string_lossy()))
                        .collect::<BTreeSet<_>>();
                    for alias in aliases {
                        let remove_alias =
                            self.source_base_keys
                                .get_mut(&alias)
                                .is_some_and(|base_keys| {
                                    base_keys.remove(&entry.base_key);
                                    base_keys.is_empty()
                                });
                        if remove_alias {
                            self.source_base_keys.remove(&alias);
                        }
                    }
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
        assert!(cache.base_entry_keys.is_empty());
        assert!(cache.source_paths.is_empty());
        assert!(cache.source_fingerprints.is_empty());
        assert!(cache.source_base_keys.is_empty());
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
