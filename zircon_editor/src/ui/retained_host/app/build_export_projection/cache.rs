use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::core::export::{FileMetadataIdentity, file_metadata_identity};
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, BuildExportTargetViewData,
};

#[derive(Clone)]
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
    next_base_revision: u64,
}

struct CachedBuildExportBase {
    project_path: PathBuf,
    source_identity: BuildExportSourceIdentity,
    revision: u64,
    projection: BuildExportBaseProjection,
}

struct CachedBuildExportPane {
    base_revision: u64,
    overlay_generation: u64,
    pane: BuildExportPaneViewData,
}

#[derive(PartialEq, Eq)]
struct BuildExportSourceIdentity {
    project_manifest: Option<FileMetadataIdentity>,
    preset_directory: Option<DirectoryMetadataIdentity>,
    preset_files: Vec<(PathBuf, Option<FileMetadataIdentity>)>,
}

#[derive(PartialEq, Eq)]
struct DirectoryMetadataIdentity {
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
}

impl BuildExportProjectionCache {
    pub(in crate::ui::retained_host::app) fn cached_base(
        &self,
        project_path: &Path,
    ) -> Option<(u64, BuildExportBaseProjection)> {
        let cached = self.base.as_ref()?;
        if cached.project_path != project_path {
            return None;
        }
        let current = capture_source_identity(
            &cached.projection.project_root,
            &cached.projection.preset_paths,
        )
        .ok()?;
        (current == cached.source_identity).then(|| (cached.revision, cached.projection.clone()))
    }

    pub(in crate::ui::retained_host::app) fn store_base(
        &mut self,
        project_path: &Path,
        projection: BuildExportBaseProjection,
    ) -> Option<u64> {
        if !projection.cacheable {
            self.base = None;
            self.rendered = None;
            return None;
        }
        let source_identity =
            capture_source_identity(&projection.project_root, &projection.preset_paths).ok()?;
        self.next_base_revision = self.next_base_revision.saturating_add(1);
        let revision = self.next_base_revision;
        self.base = Some(CachedBuildExportBase {
            project_path: project_path.to_path_buf(),
            source_identity,
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
}

fn capture_source_identity(
    project_root: &Path,
    preset_paths: &[PathBuf],
) -> std::io::Result<BuildExportSourceIdentity> {
    Ok(BuildExportSourceIdentity {
        project_manifest: optional_file_identity(&project_root.join("zircon-project.toml"))?,
        preset_directory: optional_directory_identity(&project_root.join("export"))?,
        preset_files: preset_paths
            .iter()
            .map(|path| Ok((path.clone(), optional_file_identity(path)?)))
            .collect::<std::io::Result<Vec<_>>>()?,
    })
}

fn optional_file_identity(path: &Path) -> std::io::Result<Option<FileMetadataIdentity>> {
    match file_metadata_identity(path) {
        Ok(identity) if identity.is_cacheable() => Ok(Some(identity)),
        Ok(_) => Err(std::io::Error::other(format!(
            "build/export projection source identity is not cacheable: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn optional_directory_identity(path: &Path) -> std::io::Result<Option<DirectoryMetadataIdentity>> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(Some(DirectoryMetadataIdentity {
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_source_identity_reuses_cached_projection_without_read_dir() {
        let fixture = ProjectionCacheFixture::new();
        let mut cache = BuildExportProjectionCache::default();
        let revision = cache
            .store_base(&fixture.root, fixture.projection())
            .expect("fixture source should be cacheable");

        let cached = cache
            .cached_base(&fixture.root)
            .expect("unchanged source should reuse the base projection");

        assert_eq!(cached.0, revision);
        assert_eq!(cached.1.targets.len(), 1);
    }

    #[test]
    fn changed_preset_invalidates_cached_projection_once() {
        let fixture = ProjectionCacheFixture::new();
        let mut cache = BuildExportProjectionCache::default();
        let first_revision = cache
            .store_base(&fixture.root, fixture.projection())
            .expect("fixture source should be cacheable");
        std::thread::sleep(std::time::Duration::from_millis(2));
        std::fs::write(&fixture.preset, b"changed preset bytes").unwrap();

        assert!(cache.cached_base(&fixture.root).is_none());
        let second_revision = cache
            .store_base(&fixture.root, fixture.projection())
            .expect("changed fixture source should establish one successor generation");

        assert_eq!(second_revision, first_revision + 1);
        assert_eq!(cache.cached_base(&fixture.root).unwrap().0, second_revision);
    }

    struct ProjectionCacheFixture {
        root: PathBuf,
        preset: PathBuf,
    }

    impl ProjectionCacheFixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "zircon-editor-build-export-cache-{}-{:x}",
                std::process::id(),
                fixture_nonce()
            ));
            let _ = std::fs::remove_dir_all(&root);
            let export_dir = root.join("export");
            std::fs::create_dir_all(&export_dir).unwrap();
            std::fs::write(root.join("zircon-project.toml"), b"project").unwrap();
            let preset = export_dir.join("desktop.zpreset");
            std::fs::write(&preset, b"preset").unwrap();
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
}
