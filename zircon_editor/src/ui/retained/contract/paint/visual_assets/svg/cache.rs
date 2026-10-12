use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use resvg::usvg;

use super::parse::parse_svg_tree_data;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

const MAX_SVG_TREE_CACHE_ENTRIES: usize = 1024;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_svg_tree(
    path: &Path,
) -> Option<Arc<usvg::Tree>> {
    load_svg_tree_with_parser(path, |source, resources_dir| {
        parse_svg_tree_data(source, resources_dir)
    })
}

/// 同一路径的已解析树供不同目标尺寸共用；资源监听失效或源指纹复核负责排除陈旧代际。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_svg_tree_with_parser(
    path: &Path,
    parser: impl FnOnce(&[u8], Option<PathBuf>) -> Option<usvg::Tree>,
) -> Option<Arc<usvg::Tree>> {
    let cache = svg_tree_cache();
    {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_svg_tree_cache_lookup"
        );
        if let Some(cached) = cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get_by_query_path(path)
        {
            zircon_runtime::profile_counter!("editor", "ui.svg_tree_cache.memory_hit_count", 1);
            record_current_ui_perf_counter(UiPerfCounter::SvgTreeCacheMemoryHitCount, 1.0);
            return cached;
        }
    }

    let key = SvgTreeCacheKey::from_path(path);
    {
        if let Some(cached) = cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get(&key)
        {
            return cached;
        }
    }
    zircon_runtime::profile_counter!("editor", "ui.svg_tree_cache.miss_count", 1);
    record_current_ui_perf_counter(UiPerfCounter::SvgTreeCacheMissCount, 1.0);
    let (tree, source_fingerprint) = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_parse");
        parse_svg_tree_file(&key.path, parser)
    };
    {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_svg_tree_cache_store"
        );
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert_with_fingerprint(key, tree.clone(), source_fingerprint);
    }
    tree
}

pub(in crate::ui::retained_host) fn invalidate_svg_tree_paths(paths: &[String]) -> usize {
    let Some(cache) = SVG_TREE_CACHE.get() else {
        return 0;
    };
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .invalidate_paths(paths)
}

pub(in crate::ui::retained_host) fn reconcile_svg_tree_sources() -> usize {
    let Some(cache) = SVG_TREE_CACHE.get() else {
        return 0;
    };
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .reconcile_source_fingerprints()
}

pub(in crate::ui::retained_host) fn clear_svg_tree_cache() {
    if let Some(cache) = SVG_TREE_CACHE.get() {
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clear();
    }
}

fn parse_svg_tree_file(
    path: &Path,
    parser: impl FnOnce(&[u8], Option<PathBuf>) -> Option<usvg::Tree>,
) -> (Option<Arc<usvg::Tree>>, SvgSourceFingerprint) {
    let Ok(svg) = fs::read(path) else {
        return (None, SvgSourceFingerprint::Missing);
    };
    let source_fingerprint = SvgSourceFingerprint::Content(*blake3::hash(&svg).as_bytes());
    let resources_dir = path.parent().map(Path::to_path_buf);
    (
        parser(&svg, resources_dir).map(Arc::new),
        source_fingerprint,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SvgTreeCacheKey {
    path: PathBuf,
    stamp: SvgFileStamp,
}

impl SvgTreeCacheKey {
    fn from_path(path: &Path) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let metadata = std::fs::metadata(&path).ok();
        let modified_unix_ns = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        Self {
            path,
            stamp: SvgFileStamp {
                modified_unix_ns,
                len,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SvgFileStamp {
    modified_unix_ns: Option<u128>,
    len: Option<u64>,
}

struct SvgTreeCacheEntry {
    stamp: SvgFileStamp,
    source_fingerprint: SvgSourceFingerprint,
    tree: Option<Arc<usvg::Tree>>,
    aliases: BTreeSet<String>,
    last_used: u64,
}

#[derive(Default)]
struct SvgTreeCache {
    entries: BTreeMap<PathBuf, SvgTreeCacheEntry>,
    lru_order: BTreeMap<u64, PathBuf>,
    alias_paths: BTreeMap<String, BTreeSet<PathBuf>>,
    access_clock: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SvgSourceFingerprint {
    Missing,
    Content([u8; 32]),
}

impl SvgSourceFingerprint {
    fn read(path: &Path) -> Self {
        fs::read(path).map_or(Self::Missing, |bytes| {
            Self::Content(*blake3::hash(&bytes).as_bytes())
        })
    }
}

impl SvgTreeCache {
    fn get_by_query_path(&mut self, path: &Path) -> Option<Option<Arc<usvg::Tree>>> {
        let alias = normalize_path(path.to_string_lossy().as_ref())?;
        let cached_paths = self.alias_paths.get(&alias)?;
        if cached_paths.len() != 1 {
            return None;
        }
        let cached_path = cached_paths.first()?.clone();
        let previous_last_used = self.entries.get(&cached_path)?.last_used;
        let last_used = self.next_access();
        self.lru_order.remove(&previous_last_used);
        let entry = self.entries.get_mut(&cached_path)?;
        entry.last_used = last_used;
        let tree = entry.tree.clone();
        self.lru_order.insert(last_used, cached_path);
        Some(tree)
    }

    fn get(&mut self, key: &SvgTreeCacheKey) -> Option<Option<Arc<usvg::Tree>>> {
        let entry = self.entries.get(&key.path)?;
        if entry.stamp != key.stamp {
            return None;
        }
        let previous_last_used = entry.last_used;
        let last_used = self.next_access();
        self.lru_order.remove(&previous_last_used);
        self.lru_order.insert(last_used, key.path.clone());
        let entry = self.entries.get_mut(&key.path)?;
        entry.last_used = last_used;
        Some(entry.tree.clone())
    }

    fn insert(&mut self, key: SvgTreeCacheKey, tree: Option<Arc<usvg::Tree>>) {
        let source_fingerprint = SvgSourceFingerprint::read(&key.path);
        self.insert_with_fingerprint(key, tree, source_fingerprint);
    }

    fn insert_with_fingerprint(
        &mut self,
        key: SvgTreeCacheKey,
        tree: Option<Arc<usvg::Tree>>,
        source_fingerprint: SvgSourceFingerprint,
    ) {
        let SvgTreeCacheKey { path, stamp } = key;
        self.remove(&path);
        if self.entries.len() >= MAX_SVG_TREE_CACHE_ENTRIES {
            let stale_path = self
                .lru_order
                .first_key_value()
                .map(|(_, path)| path.clone());
            if let Some(stale_path) = stale_path {
                self.remove(&stale_path);
            }
        }
        let aliases = path_aliases(&path.to_string_lossy());
        for alias in &aliases {
            self.alias_paths
                .entry(alias.clone())
                .or_default()
                .insert(path.clone());
        }
        let last_used = self.next_access();
        self.entries.insert(
            path.clone(),
            SvgTreeCacheEntry {
                stamp,
                source_fingerprint,
                tree,
                aliases,
                last_used,
            },
        );
        self.lru_order.insert(last_used, path);
    }

    fn invalidate_paths(&mut self, paths: &[String]) -> usize {
        let changed_aliases = paths
            .iter()
            .flat_map(|path| path_aliases(path))
            .collect::<BTreeSet<_>>();
        let affected_paths = changed_aliases
            .iter()
            .filter_map(|alias| self.alias_paths.get(alias))
            .flat_map(|paths| paths.iter().cloned())
            .collect::<BTreeSet<_>>();
        let changed_paths = affected_paths
            .into_iter()
            .filter(|path| {
                self.entries.get(path).is_none_or(|entry| {
                    SvgSourceFingerprint::read(path) != entry.source_fingerprint
                })
            })
            .collect::<BTreeSet<_>>();
        for path in &changed_paths {
            self.remove(path);
        }
        changed_paths.len()
    }

    fn reconcile_source_fingerprints(&mut self) -> usize {
        let changed_paths = self
            .entries
            .iter()
            .filter_map(|(path, entry)| {
                (SvgSourceFingerprint::read(path) != entry.source_fingerprint).then(|| path.clone())
            })
            .collect::<BTreeSet<_>>();
        let visited = self.entries.len();
        for path in &changed_paths {
            self.remove(path);
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.asset_refresh.svg_tree_reconcile_source_visit_count",
            visited
        );
        changed_paths.len()
    }

    fn remove(&mut self, path: &Path) {
        let Some(entry) = self.entries.remove(path) else {
            return;
        };
        self.lru_order.remove(&entry.last_used);
        for alias in entry.aliases {
            let remove_alias = self.alias_paths.get_mut(&alias).is_some_and(|paths| {
                paths.remove(path);
                paths.is_empty()
            });
            if remove_alias {
                self.alias_paths.remove(&alias);
            }
        }
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn next_access(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }
}

fn svg_tree_cache() -> &'static Mutex<SvgTreeCache> {
    SVG_TREE_CACHE.get_or_init(|| Mutex::new(SvgTreeCache::default()))
}

static SVG_TREE_CACHE: OnceLock<Mutex<SvgTreeCache>> = OnceLock::new();

fn path_aliases(path: &str) -> BTreeSet<String> {
    let Some(path) = normalize_path(path) else {
        return BTreeSet::new();
    };
    let mut aliases = BTreeSet::from([path.clone()]);
    for marker in ["/assets/", "/.zircon/cache/"] {
        if let Some(marker_offset) = path.find(marker) {
            aliases.insert(path[marker_offset + 1..].to_owned());
            if marker == "/assets/" {
                aliases.insert(path[marker_offset + marker.len()..].to_owned());
            }
        }
    }
    aliases
}

fn normalize_path(path: &str) -> Option<String> {
    let path = path.split_once('#').map_or(path, |(path, _)| path).trim();
    if path.is_empty() {
        return None;
    }
    let mut path = path.replace('\\', "/").to_ascii_lowercase();
    for prefix in ["res://", "asset://", "assets://", "file://"] {
        if let Some(stripped) = path.strip_prefix(prefix) {
            path = stripped.to_owned();
            break;
        }
    }
    if let Some(stripped) = path.strip_prefix("//?/unc/") {
        path = stripped.to_owned();
    } else if let Some(stripped) = path.strip_prefix("//?/") {
        path = stripped.to_owned();
    }
    Some(path.trim_start_matches('/').to_owned())
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;
