use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetId, AssetKind, AssetUri, AssetUuid};
use zircon_runtime_interface::project::RelPath;

use super::{
    AssetCatalogRecord, AssetImportError, AssetMetaDocument, EditorAssetChangeHub,
    EditorAssetChangeKind, EditorAssetState, EditorJobSpec, JobCategory, PreviewCache,
    PreviewRefreshEditorJob, PreviewRefreshJob, PreviewState, ProjectManager,
};
use crate::core::jobs::test_job_system;
use crate::ui::host::editor_asset_manager::manager::catalog_generation::record_to_view;
use crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration;

const ADMISSION_CAP: usize = 64;
const JOB_DEADLINE: Duration = Duration::from_secs(30);

#[test]
fn editor04_preview_publication_ready_releases_admission_and_refills_capacity() {
    published_preview_releases_admission(false);
}

#[test]
fn editor04_preview_publication_error_releases_admission_and_refills_capacity() {
    published_preview_releases_admission(true);
}

#[test]
#[ignore = "Release preview publication/admission regression diagnostic"]
fn editor04_preview_release_admission_refill_diagnostic() {
    assert!(
        !cfg!(debug_assertions),
        "run the diagnostic in Release mode"
    );
    published_preview_releases_admission(false);
    published_preview_releases_admission(true);
    println!(
        "EDITOR04_PREVIEW_RELEASE_ADMISSION_REFILL_V1 cap={ADMISSION_CAP} ready_publication=true error_publication=true completed_token_released=true waiting_uuid_refilled=true cap_retained=true os={} arch={} package_version={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION")
    );
}

fn published_preview_releases_admission(preview_error: bool) {
    let fixture = PreviewProjectFixture::new();
    let paths = ProjectPaths::from_root(&fixture.root).expect("temporary project paths");
    paths
        .ensure_layout(&[RelPath::project_assets()])
        .expect("temporary project layout");
    ProjectManifest::new(
        "Preview Admission Regression",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .expect("temporary project manifest");
    let project = ProjectManager::open(&fixture.root).expect("open temporary project");
    let cache = PreviewCache::new(paths.cache_root()).expect("preview cache");
    let records = (0..ADMISSION_CAP + 2)
        .map(|index| record(&paths, index, preview_error && index == 0))
        .collect::<Vec<_>>();
    let generation = Arc::new(catalog_generation(&records));
    let state = Arc::new(RwLock::new(EditorAssetState {
        catalog_generation: Arc::clone(&generation),
        project: Some(project.clone()),
        preview_cache: Some(cache.clone()),
        ..EditorAssetState::default()
    }));
    let completed_uuid = records[0].asset_uuid;
    let waiting_uuid = records[ADMISSION_CAP].asset_uuid;
    let overflow_uuid = records[ADMISSION_CAP + 1].asset_uuid;
    let admissions = {
        let mut state = state.write().expect("preview state lock");
        for record in &records {
            state.preview_scheduler.mark_dirty(record.asset_uuid);
        }
        let admissions = records[..ADMISSION_CAP]
            .iter()
            .map(|record| {
                let token = state
                    .preview_scheduler
                    .request_refresh(record.asset_uuid, true)
                    .expect("fill every admission slot");
                (record.asset_uuid, token)
            })
            .collect::<Vec<_>>();
        assert!(state
            .preview_scheduler
            .request_refresh(waiting_uuid, true)
            .is_none());
        admissions
    };
    let completed_token = admissions[0].1;
    let completed_uuid_text = completed_uuid.to_string();
    let change_stream = EditorAssetChangeHub::default();
    let changes = change_stream.subscribe();
    let jobs = test_job_system();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("Preview admission regression", JobCategory::Thumbnail),
            PreviewRefreshEditorJob {
                state: Arc::clone(&state),
                publish_gate: Arc::new(Mutex::new(())),
                change_stream,
                job: PreviewRefreshJob {
                    asset_uuid: completed_uuid,
                    asset_uuid_text: completed_uuid_text.clone(),
                    source_hash: records[0].source_hash.clone(),
                    source_digest: records[0].meta.source_digest.clone(),
                    meta_path: records[0].meta_path.clone(),
                    project,
                    cache,
                    record: records[0].clone(),
                    catalog_revision: generation.catalog_revision,
                    asset_row: generation
                        .asset_shared(&completed_uuid_text)
                        .expect("real catalog row identity"),
                    admission_token: completed_token,
                },
                admission_armed: true,
            },
        )
        .expect("submit the actual preview job");
    let result = ticket.wait_until(Instant::now() + JOB_DEADLINE);
    if result.is_none() {
        jobs.cancel(ticket.id());
    }
    let result = result.expect("preview job must complete before its test deadline");
    if preview_error {
        let error = result.expect_err("invalid source image must fail preview generation");
        assert!(error.downcast_ref::<AssetImportError>().is_some());
        assert!(error.to_string().contains("failed to decode preview image"));
    } else {
        assert_eq!(result, Ok(()));
    }

    let change = changes.try_recv().expect("preview publication receipt");
    assert_eq!(change.change.kind, EditorAssetChangeKind::PreviewChanged);
    assert_eq!(
        change.change.uuid.as_deref(),
        Some(completed_uuid_text.as_str())
    );
    assert_eq!(change.change.catalog_revision, generation.catalog_revision);
    assert!(changes.try_recv().is_none());

    let mut state = state.write().expect("preview state lock");
    assert_eq!(
        state.catalog_generation.catalog_revision,
        generation.catalog_revision
    );
    assert_eq!(
        state.catalog_generation.publish_epoch,
        generation.publish_epoch + 1
    );
    let published = state
        .catalog_generation
        .catalog_record(&completed_uuid_text)
        .expect("published preview record");
    let expected_state = if preview_error {
        PreviewState::Error
    } else {
        PreviewState::Ready
    };
    assert_eq!(published.preview_state, expected_state);
    assert_eq!(published.meta.preview_state, expected_state);
    assert!(!published.dirty);
    if !preview_error {
        assert!(published.preview_artifact_path.is_file());
    }
    assert!(!state
        .preview_scheduler
        .owns_refresh(completed_uuid, completed_token));
    for &(uuid, token) in &admissions[1..] {
        assert!(state.preview_scheduler.owns_refresh(uuid, token));
    }
    assert!(state
        .preview_scheduler
        .request_refresh(completed_uuid, true)
        .is_none());
    let waiting_token = state
        .preview_scheduler
        .request_refresh(waiting_uuid, true)
        .expect("a published preview must free one slot for the waiting UUID");
    assert!(state
        .preview_scheduler
        .owns_refresh(waiting_uuid, waiting_token));
    assert!(state
        .preview_scheduler
        .request_refresh(overflow_uuid, true)
        .is_none());
}

fn record(paths: &ProjectPaths, index: usize, invalid_texture: bool) -> AssetCatalogRecord {
    let asset_uuid = AssetUuid::from_stable_label(&format!("editor04-release-preview-{index:03}"));
    let kind = if invalid_texture {
        AssetKind::Texture
    } else {
        AssetKind::Data
    };
    let extension = if invalid_texture { "png" } else { "dat" };
    let file_name = format!("preview_{index:03}.{extension}");
    let source_path = paths
        .asset_root(&RelPath::project_assets())
        .join(&file_name);
    let locator = AssetUri::parse(&format!("res://{file_name}")).unwrap();
    let mut meta = AssetMetaDocument::new(asset_uuid, locator.clone(), kind);
    meta.source_digest = format!("preview-source-{index:03}");
    let meta_path = source_path.with_extension(format!("{extension}.zmeta"));
    if index == 0 {
        fs::write(&source_path, b"preview admission fixture, not a PNG")
            .expect("published source fixture");
        meta.save(&meta_path).expect("matching preview metadata");
    }
    AssetCatalogRecord {
        asset_uuid,
        asset_id: AssetId::from_asset_uuid(asset_uuid),
        locator,
        kind,
        display_name: file_name.clone(),
        file_name,
        extension: extension.to_owned(),
        meta_path,
        source_mtime_unix_ms: 0,
        source_hash: meta.source_digest.clone(),
        preview_state: PreviewState::Dirty,
        preview_artifact_path: PathBuf::new(),
        dirty: true,
        diagnostics: Vec::new(),
        direct_references: Vec::new(),
        meta,
    }
}

fn catalog_generation(records: &[AssetCatalogRecord]) -> EditorAssetCatalogGeneration {
    let by_uuid = records
        .iter()
        .cloned()
        .map(|record| (record.asset_uuid, record))
        .collect::<HashMap<_, _>>();
    let by_locator = records
        .iter()
        .map(|record| (record.locator.clone(), record.asset_uuid))
        .collect::<HashMap<_, _>>();
    let assets = records
        .iter()
        .map(|record| Arc::new(record_to_view(record, &by_uuid, &by_locator)))
        .collect();
    let catalog_records = records
        .iter()
        .cloned()
        .map(|record| Some(Arc::new(record)))
        .collect();
    EditorAssetCatalogGeneration::from_parts(
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        1,
        1,
        Vec::new(),
        assets,
        vec![None; records.len()],
        catalog_records,
    )
}

struct PreviewProjectFixture {
    root: PathBuf,
}

impl PreviewProjectFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon_editor_preview_release_{}_{}",
            std::process::id(),
            AssetUuid::new()
        ));
        fs::create_dir_all(&root).expect("temporary project root");
        Self { root }
    }
}

impl Drop for PreviewProjectFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
