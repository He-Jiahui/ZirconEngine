use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::ui::v2::{UiV2PrototypeStoreFileCache, UiV2PrototypeStoreLoadOutcome};
use zircon_runtime_interface::ui::v2::UiV2AssetError;

pub(super) fn load_view_v2_store(
    layout_asset_path: &str,
    style_imports: &[(&str, &str)],
) -> Result<UiV2PrototypeStoreLoadOutcome, UiV2AssetError> {
    view_v2_store_file_cache()
        .lock()
        .map_err(|_| UiV2AssetError::InvalidDocument {
            asset_id: layout_asset_path.to_string(),
            detail: "view v2 store cache mutex poisoned".to_string(),
        })?
        .load_store(layout_asset_path, style_imports)
}

struct ViewV2StoreFileCache {
    source_cache: UiV2PrototypeStoreFileCache,
    invalidation: ViewTemplateStoreInvalidation,
    source_paths: BTreeMap<ViewV2StoreRequest, Vec<PathBuf>>,
}

impl ViewV2StoreFileCache {
    fn new() -> Self {
        Self {
            source_cache: UiV2PrototypeStoreFileCache::new(),
            invalidation: ViewTemplateStoreInvalidation::new(),
            source_paths: BTreeMap::new(),
        }
    }

    fn load_store(
        &mut self,
        layout_asset_path: &str,
        style_imports: &[(&str, &str)],
    ) -> Result<UiV2PrototypeStoreLoadOutcome, UiV2AssetError> {
        let request = ViewV2StoreRequest::new(layout_asset_path, style_imports);
        let paths = self
            .source_paths
            .entry(request)
            .or_insert_with(|| v2_source_paths(layout_asset_path, style_imports))
            .clone();
        let event_driven = self.invalidation.ensure_watched(&paths);
        if self.invalidation.take_changed() {
            self.source_cache.clear();
            zircon_runtime::profile_counter!(
                "editor",
                "ui.template_store.file_event_invalidation_count",
                1
            );
        }
        if event_driven {
            self.source_cache.load_store_cached(paths)
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.template_store.metadata_validation_fallback_count",
                1
            );
            self.source_cache.load_store(paths)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ViewV2StoreRequest {
    layout_asset_path: String,
    style_asset_paths: Vec<String>,
}

impl ViewV2StoreRequest {
    fn new(layout_asset_path: &str, style_imports: &[(&str, &str)]) -> Self {
        Self {
            layout_asset_path: layout_asset_path.to_string(),
            style_asset_paths: style_imports
                .iter()
                .map(|(_, style_path)| (*style_path).to_string())
                .collect(),
        }
    }
}

fn view_v2_store_file_cache() -> &'static Mutex<ViewV2StoreFileCache> {
    static CACHE: OnceLock<Mutex<ViewV2StoreFileCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(ViewV2StoreFileCache::new()))
}

#[cfg(test)]
pub(super) fn clear_for_tests() {
    let mut cache = view_v2_store_file_cache()
        .lock()
        .expect("view v2 store cache mutex should not be poisoned");
    cache.source_cache.clear();
    cache.source_paths.clear();
}

#[cfg(test)]
pub(super) fn cached_store_count_for_tests() -> usize {
    view_v2_store_file_cache()
        .lock()
        .expect("view v2 store cache mutex should not be poisoned")
        .source_cache
        .len()
}

struct ViewTemplateStoreInvalidation {
    changed: Arc<AtomicBool>,
    watchers: BTreeMap<PathBuf, RecommendedWatcher>,
    unavailable_roots: HashSet<PathBuf>,
    source_roots: HashMap<PathBuf, Option<PathBuf>>,
}

impl ViewTemplateStoreInvalidation {
    fn new() -> Self {
        Self {
            changed: Arc::new(AtomicBool::new(false)),
            watchers: BTreeMap::new(),
            unavailable_roots: HashSet::new(),
            source_roots: HashMap::new(),
        }
    }

    fn ensure_watched(&mut self, source_paths: &[PathBuf]) -> bool {
        let mut roots = BTreeSet::new();
        for path in source_paths {
            let root = self
                .source_roots
                .entry(path.clone())
                .or_insert_with(|| asset_root_for_source(path));
            if let Some(root) = root {
                roots.insert(root.clone());
            }
        }
        if roots.is_empty() {
            return false;
        }

        for root in &roots {
            if self.watchers.contains_key(root) || self.unavailable_roots.contains(root) {
                continue;
            }
            match watch_asset_root(root, Arc::clone(&self.changed)) {
                Ok(watcher) => {
                    self.watchers.insert(root.clone(), watcher);
                }
                Err(_) => {
                    self.unavailable_roots.insert(root.clone());
                }
            }
        }

        roots.iter().all(|root| self.watchers.contains_key(root))
    }

    fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::AcqRel)
    }
}

fn watch_asset_root(root: &Path, changed: Arc<AtomicBool>) -> notify::Result<RecommendedWatcher> {
    let mut watcher =
        notify::recommended_watcher(move |event: notify::Result<Event>| match event {
            Ok(event) if affects_v2_source(&event) => changed.store(true, Ordering::Release),
            Err(_) => changed.store(true, Ordering::Release),
            _ => {}
        })?;
    watcher.watch(root, RecursiveMode::Recursive)?;
    Ok(watcher)
}

fn affects_v2_source(event: &Event) -> bool {
    if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
        return false;
    }
    event.paths.iter().any(|path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_none_or(|extension| extension.eq_ignore_ascii_case("zui"))
    })
}

fn v2_source_paths(layout_asset_path: &str, style_imports: &[(&str, &str)]) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(1 + style_imports.len());
    paths.push(asset_path(layout_asset_path));
    paths.extend(
        style_imports
            .iter()
            .map(|(_, style_path)| asset_path(style_path)),
    );
    paths
}

fn asset_path(relative: &str) -> PathBuf {
    runtime_asset_path_with_dev_asset_root(relative, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn asset_root_for_source(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| {
            ancestor.file_name().and_then(|name| name.to_str()) == Some("assets")
                && ancestor.join("ui").is_dir()
        })
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const ROOT_COUNT: usize = 8_192;
    const ROOT_LOOKUP_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn roots() -> Vec<PathBuf> {
        (0..ROOT_COUNT)
            .map(|index| {
                PathBuf::from(format!(
                    "generated/projects/with/a/long/shared/prefix/project_{index:05}/assets"
                ))
            })
            .collect()
    }

    fn lookups(roots: &[PathBuf]) -> Vec<PathBuf> {
        (0..ROOT_LOOKUP_COUNT)
            .map(|index| roots[(index * 4_099) % roots.len()].clone())
            .collect()
    }

    fn ordered_match_count(
        source_roots: &BTreeMap<PathBuf, Option<PathBuf>>,
        unavailable_roots: &BTreeSet<PathBuf>,
        lookups: &[PathBuf],
    ) -> usize {
        lookups
            .iter()
            .filter(|root| {
                source_roots.contains_key(root.as_path())
                    && unavailable_roots.contains(root.as_path())
            })
            .count()
    }

    fn hash_match_count(
        source_roots: &HashMap<PathBuf, Option<PathBuf>>,
        unavailable_roots: &HashSet<PathBuf>,
        lookups: &[PathBuf],
    ) -> usize {
        lookups
            .iter()
            .filter(|root| {
                source_roots.contains_key(root.as_path())
                    && unavailable_roots.contains(root.as_path())
            })
            .count()
    }

    #[test]
    fn v2_store_invalidation_ignores_non_zui_file_events() {
        assert!(!affects_v2_source(
            &Event::new(EventKind::Modify(notify::event::ModifyKind::Any))
                .add_path(PathBuf::from("icon.svg"))
        ));
        assert!(affects_v2_source(
            &Event::new(EventKind::Modify(notify::event::ModifyKind::Any))
                .add_path(PathBuf::from("layout.zui"))
        ));
    }

    #[test]
    fn optimization_batch_20260826ad_editor01_hash_root_caches_preserve_unavailable_membership() {
        let source = PathBuf::from("project/assets/ui/views/main.zui");
        let root = PathBuf::from("project/assets");
        let mut invalidation = ViewTemplateStoreInvalidation::new();
        invalidation
            .source_roots
            .insert(source.clone(), Some(root.clone()));
        invalidation.unavailable_roots.insert(root.clone());

        assert_eq!(
            invalidation.source_roots.get(&source),
            Some(&Some(root.clone()))
        );
        assert!(invalidation.unavailable_roots.contains(&root));
        assert!(invalidation.watchers.is_empty());
    }

    #[test]
    fn optimization_batch_20260826ad_editor01_view_store_uses_hash_caches_and_ordered_watch_roots()
    {
        let source = include_str!("store_cache.rs");

        assert!(source.contains("use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};"));
        assert!(source.contains("unavailable_roots: HashSet<PathBuf>"));
        assert!(source.contains("source_roots: HashMap<PathBuf, Option<PathBuf>>"));
        assert!(source.contains("let mut roots = BTreeSet::new();"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826ad_editor01_view_store_root_hash_caches_performance_evidence() {
        let roots = roots();
        let lookups = lookups(&roots);
        let ordered_source_roots = roots
            .iter()
            .cloned()
            .map(|root| (root.clone(), Some(root)))
            .collect::<BTreeMap<_, _>>();
        let ordered_unavailable_roots = roots.iter().cloned().collect::<BTreeSet<_>>();
        let hash_source_roots = roots
            .iter()
            .cloned()
            .map(|root| (root.clone(), Some(root)))
            .collect::<HashMap<_, _>>();
        let hash_unavailable_roots = roots.iter().cloned().collect::<HashSet<_>>();
        assert_eq!(
            ordered_match_count(&ordered_source_roots, &ordered_unavailable_roots, &lookups,),
            hash_match_count(&hash_source_roots, &hash_unavailable_roots, &lookups)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_match_count(
                    black_box(&ordered_source_roots),
                    black_box(&ordered_unavailable_roots),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_match_count(
                    black_box(&hash_source_roots),
                    black_box(&hash_unavailable_roots),
                    black_box(&lookups),
                ));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_match_count(
                    black_box(&hash_source_roots),
                    black_box(&hash_unavailable_roots),
                    black_box(&lookups),
                ));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_match_count(
                    black_box(&ordered_source_roots),
                    black_box(&ordered_unavailable_roots),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "EDITOR01_VIEW_STORE_ROOT_HASH_CACHES_BENCH_V1 \
             cached_roots={ROOT_COUNT} lookups={ROOT_LOOKUP_COUNT} \
             ordered_watch_registration=true ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-cache P95 {:?} exceeded 60% of ordered-cache P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
