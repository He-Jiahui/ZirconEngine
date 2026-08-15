use std::collections::{BTreeMap, BTreeSet};
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
    unavailable_roots: BTreeSet<PathBuf>,
    source_roots: BTreeMap<PathBuf, Option<PathBuf>>,
}

impl ViewTemplateStoreInvalidation {
    fn new() -> Self {
        Self {
            changed: Arc::new(AtomicBool::new(false)),
            watchers: BTreeMap::new(),
            unavailable_roots: BTreeSet::new(),
            source_roots: BTreeMap::new(),
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
    use super::*;

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
}
