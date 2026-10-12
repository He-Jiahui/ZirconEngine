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
                        path == &callback_manifest || path.starts_with(&callback_export_directory)
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
#[path = "tests/cache.rs"]
mod tests;
