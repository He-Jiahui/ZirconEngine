use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, BuildExportTargetViewData,
};
use crate::ui::workbench::project::project_root_path;

pub(in crate::ui::retained_host::app) struct BuildExportBaseProjection {
    pub project_root: PathBuf,
    pub targets: Vec<BuildExportTargetViewData>,
    pub diagnostics: Vec<String>,
    pub preset_paths: Vec<PathBuf>,
    pub cacheable: bool,
}

impl BuildExportBaseProjection {
    pub(super) fn uncacheable(diagnostic: String) -> Self {
        Self {
            project_root: PathBuf::new(),
            targets: Vec::new(),
            diagnostics: vec![diagnostic],
            preset_paths: Vec::new(),
            cacheable: false,
        }
    }
}

#[derive(Default)]
pub(in crate::ui::retained_host::app) struct BuildExportProjectionCache {
    base: Option<CachedBuildExportBase>,
    rendered: Option<CachedBuildExportPane>,
    source_watch: Option<BuildExportSourceWatch>,
    next_base_revision: u64,
}

struct CachedBuildExportBase {
    project_path: PathBuf,
    source_generation: u64,
    revision: u64,
    projection: Arc<BuildExportBaseProjection>,
}

struct CachedBuildExportPane {
    base_revision: u64,
    overlay_generation: u64,
    pane: BuildExportPaneViewData,
}

pub(in crate::ui::retained_host::app) enum BuildExportBaseLookup {
    Hit {
        revision: u64,
        projection: Arc<BuildExportBaseProjection>,
    },
    Miss(Option<BuildExportBaseBuildToken>),
}

pub(in crate::ui::retained_host::app) struct BuildExportBaseBuildToken {
    project_path: PathBuf,
    project_root: PathBuf,
    source_generation: u64,
}

struct BuildExportSourceWatch {
    project_path: PathBuf,
    project_root: PathBuf,
    export_directory: PathBuf,
    source_generation: Arc<AtomicU64>,
    configured_generation: u64,
    export_watched: bool,
    watcher: RecommendedWatcher,
}

impl BuildExportProjectionCache {
    pub(in crate::ui::retained_host::app) fn lookup_base(
        &mut self,
        project_path: &Path,
    ) -> BuildExportBaseLookup {
        let watch_matches = self
            .source_watch
            .as_ref()
            .is_some_and(|watch| watch.project_path == project_path);
        if !watch_matches {
            self.invalidate_source();
            let Ok(project_root) = project_root_path(project_path) else {
                self.source_watch = None;
                return BuildExportBaseLookup::Miss(None);
            };
            self.source_watch =
                BuildExportSourceWatch::start(project_path.to_path_buf(), project_root).ok();
        } else if self
            .source_watch
            .as_mut()
            .is_some_and(|watch| watch.refresh_after_change().is_err())
        {
            self.invalidate_source();
            self.source_watch = None;
            return BuildExportBaseLookup::Miss(None);
        }

        let Some(watch) = self.source_watch.as_ref() else {
            return BuildExportBaseLookup::Miss(None);
        };
        let source_generation = watch.source_generation();
        if let Some(cached) = self.base.as_ref().filter(|cached| {
            cached.project_path == project_path && cached.source_generation == source_generation
        }) {
            return BuildExportBaseLookup::Hit {
                revision: cached.revision,
                projection: Arc::clone(&cached.projection),
            };
        }
        self.base = None;
        self.rendered = None;
        BuildExportBaseLookup::Miss(Some(BuildExportBaseBuildToken {
            project_path: project_path.to_path_buf(),
            project_root: watch.project_root.clone(),
            source_generation,
        }))
    }

    pub(in crate::ui::retained_host::app) fn store_base(
        &mut self,
        token: Option<BuildExportBaseBuildToken>,
        projection: Arc<BuildExportBaseProjection>,
    ) -> Option<u64> {
        let Some(token) = token else {
            self.base = None;
            self.rendered = None;
            return None;
        };
        let Some(watch) = self.source_watch.as_ref() else {
            return None;
        };
        let source_generation = watch.source_generation();
        let source_generation_unchanged = source_generation == token.source_generation;
        if !projection.cacheable
            || watch.project_path != token.project_path
            || watch.project_root != token.project_root
            || projection.project_root != token.project_root
            || !source_generation_unchanged
        {
            self.base = None;
            self.rendered = None;
            return None;
        }
        self.next_base_revision = self.next_base_revision.saturating_add(1);
        let revision = self.next_base_revision;
        self.base = Some(CachedBuildExportBase {
            project_path: token.project_path,
            source_generation,
            revision,
            projection,
        });
        self.rendered = None;
        Some(revision)
    }

    pub(in crate::ui::retained_host::app) fn cached_rendered(
        &self,
        base_revision: u64,
        overlay_generation: u64,
    ) -> Option<BuildExportPaneViewData> {
        self.rendered
            .as_ref()
            .filter(|cached| {
                cached.base_revision == base_revision
                    && cached.overlay_generation == overlay_generation
            })
            .map(|cached| cached.pane.clone())
    }

    pub(in crate::ui::retained_host::app) fn store_rendered(
        &mut self,
        base_revision: u64,
        overlay_generation: u64,
        pane: BuildExportPaneViewData,
    ) {
        self.rendered = Some(CachedBuildExportPane {
            base_revision,
            overlay_generation,
            pane,
        });
    }

    pub(in crate::ui::retained_host::app) fn invalidate_source(&mut self) {
        self.base = None;
        self.rendered = None;
    }

    pub(in crate::ui::retained_host::app) fn invalidate_overlay(&mut self) {
        self.rendered = None;
    }

    #[cfg(test)]
    fn mark_source_changed_for_test(&self) {
        if let Some(watch) = &self.source_watch {
            watch.source_generation.fetch_add(1, Ordering::AcqRel);
        }
    }
}

impl BuildExportSourceWatch {
    fn start(project_path: PathBuf, project_root: PathBuf) -> std::io::Result<Self> {
        let source_generation = Arc::new(AtomicU64::new(0));
        let manifest_path = project_root.join("zircon-project.toml");
        let export_directory = project_root.join("export");
        let callback_generation = Arc::clone(&source_generation);
        let callback_manifest = manifest_path.clone();
        let callback_export_directory = export_directory.clone();
        let mut watcher = notify::recommended_watcher(
            move |result: notify::Result<notify::Event>| match result {
                Ok(event)
                    if event.paths.iter().any(|path| {
                        path == callback_manifest || path.starts_with(&callback_export_directory)
                    }) =>
                {
                    callback_generation.fetch_add(1, Ordering::AcqRel);
                }
                Err(_) => {
                    callback_generation.fetch_add(1, Ordering::AcqRel);
                }
                _ => {}
            },
        )
        .map_err(notify_error)?;
        watcher
            .watch(&project_root, RecursiveMode::NonRecursive)
            .map_err(notify_error)?;
        let export_watched = export_directory.is_dir();
        if export_watched {
            watcher
                .watch(&export_directory, RecursiveMode::Recursive)
                .map_err(notify_error)?;
        }
        let configured_generation = source_generation.load(Ordering::Acquire);
        Ok(Self {
            project_path,
            project_root,
            export_directory,
            source_generation,
            configured_generation,
            export_watched,
            watcher,
        })
    }

    fn source_generation(&self) -> u64 {
        self.source_generation.load(Ordering::Acquire)
    }

    fn refresh_after_change(&mut self) -> std::io::Result<()> {
        let source_generation = self.source_generation();
        if source_generation == self.configured_generation {
            return Ok(());
        }
        let export_exists = self.export_directory.is_dir();
        if export_exists && !self.export_watched {
            self.watcher
                .watch(&self.export_directory, RecursiveMode::Recursive)
                .map_err(notify_error)?;
            self.export_watched = true;
        } else if !export_exists && self.export_watched {
            let _ = self.watcher.unwatch(&self.export_directory);
            self.export_watched = false;
        }
        self.configured_generation = source_generation;
        Ok(())
    }
}

fn notify_error(error: notify::Error) -> std::io::Error {
    std::io::Error::other(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_source_generation_reuses_cached_projection_without_filesystem_probes() {
        let fixture = ProjectionCacheFixture::new();
        let mut cache = BuildExportProjectionCache::default();
        let token = build_token(&mut cache, &fixture.root);
        let revision = cache
            .store_base(token, Arc::new(fixture.projection()))
            .expect("fixture source should be cacheable");

        let BuildExportBaseLookup::Hit {
            revision: cached_revision,
            projection,
        } = cache.lookup_base(&fixture.root)
        else {
            panic!("unchanged source should reuse the base projection");
        };

        assert_eq!(cached_revision, revision);
        assert_eq!(projection.targets.len(), 1);
    }

    #[test]
    fn cached_base_reuses_the_same_projection_allocation() {
        let fixture = ProjectionCacheFixture::new();
        let mut cache = BuildExportProjectionCache::default();
        let token = build_token(&mut cache, &fixture.root);
        cache
            .store_base(token, Arc::new(fixture.projection()))
            .expect("fixture source should be cacheable");

        let first = cached_projection(&mut cache, &fixture.root);
        let second = cached_projection(&mut cache, &fixture.root);

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn changed_source_generation_invalidates_cached_projection_once() {
        let fixture = ProjectionCacheFixture::new();
        let mut cache = BuildExportProjectionCache::default();
        let token = build_token(&mut cache, &fixture.root);
        let first_revision = cache
            .store_base(token, Arc::new(fixture.projection()))
            .expect("fixture source should be cacheable");
        cache.mark_source_changed_for_test();

        let token = build_token(&mut cache, &fixture.root);
        let second_revision = cache
            .store_base(token, Arc::new(fixture.projection()))
            .expect("changed fixture source should establish one successor generation");

        assert_eq!(second_revision, first_revision + 1);
        let BuildExportBaseLookup::Hit { revision, .. } = cache.lookup_base(&fixture.root) else {
            panic!("successor generation should be cached");
        };
        assert_eq!(revision, second_revision);
    }

    #[test]
    fn source_change_during_build_rejects_stale_projection_publication() {
        let fixture = ProjectionCacheFixture::new();
        let mut cache = BuildExportProjectionCache::default();
        let token = build_token(&mut cache, &fixture.root);

        cache.mark_source_changed_for_test();

        assert_eq!(
            cache.store_base(token, Arc::new(fixture.projection())),
            None
        );
    }

    #[test]
    fn preset_write_advances_the_watcher_generation() {
        let fixture = ProjectionCacheFixture::new();
        let mut cache = BuildExportProjectionCache::default();
        let token = build_token(&mut cache, &fixture.root);
        cache
            .store_base(token, Arc::new(fixture.projection()))
            .expect("fixture source should be cacheable");

        std::fs::write(&fixture.preset, b"changed preset bytes").unwrap();

        assert!(wait_for_source_miss(&mut cache, &fixture.root).is_some());
    }

    #[test]
    fn created_export_directory_is_watched_for_followup_preset_changes() {
        let fixture = ProjectionCacheFixture::without_export_directory();
        let mut cache = BuildExportProjectionCache::default();
        let token = build_token(&mut cache, &fixture.root);
        cache
            .store_base(token, Arc::new(fixture.projection()))
            .expect("fixture source should be cacheable");

        std::fs::create_dir_all(fixture.root.join("export")).unwrap();
        std::fs::write(&fixture.preset, b"first preset bytes").unwrap();
        let token = wait_for_source_miss(&mut cache, &fixture.root)
            .expect("export directory creation should invalidate the base");
        cache
            .store_base(Some(token), Arc::new(fixture.projection()))
            .expect("new export directory generation should be cacheable");
        settle_source_generation(&mut cache, &fixture);

        std::fs::write(&fixture.preset, b"second preset bytes").unwrap();

        assert!(wait_for_source_miss(&mut cache, &fixture.root).is_some());
    }

    struct ProjectionCacheFixture {
        root: PathBuf,
        preset: PathBuf,
    }

    impl ProjectionCacheFixture {
        fn new() -> Self {
            Self::create(true)
        }

        fn without_export_directory() -> Self {
            Self::create(false)
        }

        fn create(with_export_directory: bool) -> Self {
            let target_directory = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .expect("build/export cache tests require managed CARGO_TARGET_DIR");
            let root = target_directory.join(format!(
                "zircon-editor-build-export-cache-{}-{:x}",
                std::process::id(),
                fixture_nonce()
            ));
            let _ = std::fs::remove_dir_all(&root);
            let export_dir = root.join("export");
            if with_export_directory {
                std::fs::create_dir_all(&export_dir).unwrap();
            } else {
                std::fs::create_dir_all(&root).unwrap();
            }
            std::fs::write(root.join("zircon-project.toml"), b"project").unwrap();
            let preset = export_dir.join("desktop.zpreset");
            if with_export_directory {
                std::fs::write(&preset, b"preset").unwrap();
            }
            Self { root, preset }
        }

        fn projection(&self) -> BuildExportBaseProjection {
            BuildExportBaseProjection {
                project_root: self.root.clone(),
                targets: vec![BuildExportTargetViewData::default()],
                diagnostics: Vec::new(),
                preset_paths: vec![self.preset.clone()],
                cacheable: true,
            }
        }
    }

    impl Drop for ProjectionCacheFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn fixture_nonce() -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        std::time::SystemTime::now().hash(&mut hasher);
        hasher.finish()
    }

    fn build_token(
        cache: &mut BuildExportProjectionCache,
        project_path: &Path,
    ) -> Option<BuildExportBaseBuildToken> {
        let BuildExportBaseLookup::Miss(token) = cache.lookup_base(project_path) else {
            panic!("uncached fixture should issue a build token");
        };
        token
    }

    fn cached_projection(
        cache: &mut BuildExportProjectionCache,
        project_path: &Path,
    ) -> Arc<BuildExportBaseProjection> {
        let BuildExportBaseLookup::Hit { projection, .. } = cache.lookup_base(project_path) else {
            panic!("fixture should return the cached projection");
        };
        projection
    }

    fn wait_for_source_miss(
        cache: &mut BuildExportProjectionCache,
        project_path: &Path,
    ) -> Option<BuildExportBaseBuildToken> {
        for _ in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if let BuildExportBaseLookup::Miss(token) = cache.lookup_base(project_path) {
                return token;
            }
        }
        panic!("source watcher did not publish a changed generation");
    }

    fn settle_source_generation(
        cache: &mut BuildExportProjectionCache,
        fixture: &ProjectionCacheFixture,
    ) {
        let mut consecutive_hits = 0;
        for _ in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            match cache.lookup_base(&fixture.root) {
                BuildExportBaseLookup::Hit { .. } => {
                    consecutive_hits += 1;
                    if consecutive_hits == 3 {
                        return;
                    }
                }
                BuildExportBaseLookup::Miss(token) => {
                    consecutive_hits = 0;
                    cache
                        .store_base(token, Arc::new(fixture.projection()))
                        .expect("settled source generation should be cacheable");
                }
            }
        }
        panic!("source watcher generation did not settle");
    }
}
