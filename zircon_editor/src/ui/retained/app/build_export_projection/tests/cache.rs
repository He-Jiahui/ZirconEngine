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
