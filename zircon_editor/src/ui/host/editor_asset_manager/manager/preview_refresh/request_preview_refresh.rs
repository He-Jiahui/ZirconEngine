use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState, ProjectManager};
use zircon_runtime::asset::AssetUuid;

use super::super::catalog_generation::{record_to_view, update_asset_in_catalog_generation};
use super::super::default_editor_asset_manager::{DefaultEditorAssetManager, EditorAssetState};
use super::generate_preview_artifact::generate_preview_artifact;
use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, MutexGroup,
};
use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, EditorAssetCatalogRecord, EditorAssetChangeHub, EditorAssetChangeKind,
    EditorAssetChangeRecord, PreviewCache, PreviewJobToken,
};

struct PreviewRefreshJob {
    asset_uuid: AssetUuid,
    asset_uuid_text: String,
    source_hash: String,
    source_digest: String,
    meta_path: PathBuf,
    project: ProjectManager,
    cache: PreviewCache,
    record: AssetCatalogRecord,
    catalog_revision: u64,
    asset_row: Arc<EditorAssetCatalogRecord>,
    admission_token: PreviewJobToken,
}

struct PreviewRefreshEditorJob {
    state: Arc<RwLock<EditorAssetState>>,
    publish_gate: Arc<Mutex<()>>,
    change_stream: EditorAssetChangeHub,
    job: PreviewRefreshJob,
    admission_armed: bool,
}

impl EditorJob for PreviewRefreshEditorJob {
    type Output = ();

    fn run(mut self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let result = complete_preview_refresh_job(
            &self.state,
            &self.publish_gate,
            &self.change_stream,
            &self.job,
            &context,
        );
        if !matches!(&result, Err(JobError::Cancelled)) {
            self.admission_armed = false;
        }
        result
    }
}

impl Drop for PreviewRefreshEditorJob {
    fn drop(&mut self) {
        if self.admission_armed {
            release_preview_admission(&self.state, &self.change_stream, &self.job);
        }
    }
}

impl DefaultEditorAssetManager {
    pub fn request_preview_refresh(
        &self,
        asset_uuid: AssetUuid,
        visible: bool,
    ) -> Result<Option<AssetCatalogRecord>, AssetImportError> {
        let preview_jobs = self.preview_jobs.as_ref().ok_or_else(|| {
            AssetImportError::Parse("preview job system is not initialized".to_string())
        })?;
        let asset_uuid_text = asset_uuid.to_string();
        let mutex_group =
            MutexGroup::parse(format!("thumbnail_{asset_uuid_text}").replace('-', ""))
                .map_err(|error| AssetImportError::Parse(error.to_string()))?;
        let job = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            let Some(record) = state.catalog_by_uuid.get(&asset_uuid).cloned() else {
                return Ok(None);
            };
            let cache = state.preview_cache.as_ref().cloned().ok_or_else(|| {
                AssetImportError::Parse("preview cache is not initialized".to_string())
            })?;
            let project = state.project.as_ref().cloned().ok_or_else(|| {
                AssetImportError::Parse("editor project is not initialized".to_string())
            })?;
            let Some(admission_token) =
                state.preview_scheduler.request_refresh(asset_uuid, visible)
            else {
                return Ok(Some(record));
            };
            let asset_row = state
                .catalog_generation
                .asset_shared(&asset_uuid_text)
                .expect("mutable catalog and immutable generation must share every asset");
            PreviewRefreshJob {
                asset_uuid,
                asset_uuid_text,
                source_hash: record.source_hash.clone(),
                source_digest: record.meta.source_digest.clone(),
                meta_path: record.meta_path.clone(),
                project,
                cache,
                record,
                catalog_revision: state.catalog_generation.catalog_revision,
                asset_row,
                admission_token,
            }
        };

        let queued_record = job.record.clone();
        let state = Arc::clone(&self.state);
        let publish_gate = Arc::clone(&self.publish_gate);
        let change_stream = self.change_stream.clone();
        let spec = EditorJobSpec::new(
            format!("Generate asset preview {asset_uuid}"),
            JobCategory::Thumbnail,
        )
        .with_priority(JobPriority::Background)
        .with_mutex_group(mutex_group);
        if let Err(error) = preview_jobs.submit(
            spec,
            PreviewRefreshEditorJob {
                state,
                publish_gate,
                change_stream,
                job,
                admission_armed: true,
            },
        ) {
            return Err(AssetImportError::Parse(error.to_string()));
        }
        Ok(Some(queued_record))
    }
}

fn complete_preview_refresh_job(
    state: &Arc<RwLock<EditorAssetState>>,
    publish_gate: &Arc<Mutex<()>>,
    change_stream: &EditorAssetChangeHub,
    job: &PreviewRefreshJob,
    context: &JobContext,
) -> Result<(), JobError> {
    let mut updated_record = job.record.clone();
    let mut completion_error =
        match generate_preview_artifact(&job.project, &job.record, &job.cache) {
            Ok(path) => {
                updated_record.preview_artifact_path = path;
                updated_record.preview_state = PreviewState::Ready;
                updated_record.dirty = false;
                None
            }
            Err(error) => {
                updated_record.preview_state = PreviewState::Error;
                updated_record.dirty = false;
                Some(error)
            }
        };
    context.check_cancelled()?;

    let job_is_current = {
        let state = state.read().expect("editor asset state lock poisoned");
        preview_job_is_current(&state, job)
    };
    if !job_is_current {
        release_preview_admission(state, change_stream, job);
        return Ok(());
    }

    let latest_meta = AssetMetaDocument::load(&job.meta_path);
    context.check_cancelled()?;
    match latest_meta {
        Ok(mut latest_meta) => {
            if latest_meta.uuid != job.record.meta.uuid
                || latest_meta.url != job.record.meta.url
                || latest_meta.source_digest != job.source_digest
            {
                release_preview_admission(state, change_stream, job);
                return Ok(());
            }
            let mut comparable_meta = latest_meta.clone();
            comparable_meta.preview_state = job.record.meta.preview_state;
            if comparable_meta != job.record.meta {
                release_preview_admission(state, change_stream, job);
                return Ok(());
            }
            latest_meta.preview_state = updated_record.preview_state;
            updated_record.meta = latest_meta;
        }
        Err(error) => {
            updated_record.preview_state = PreviewState::Error;
            updated_record.dirty = false;
            updated_record.meta.preview_state = PreviewState::Error;
            if completion_error.is_none() {
                completion_error = Some(error.into());
            }
        }
    }

    context.check_cancelled()?;
    let _publish_guard = publish_gate
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    context.check_cancelled()?;
    let change = {
        let mut state = state.write().expect("editor asset state lock poisoned");
        context.check_cancelled()?;
        if !preview_job_is_current(&state, job) {
            let refill = release_preview_admission_locked(&mut state, job);
            drop(state);
            if let Some(refill) = refill {
                change_stream.publish(refill);
            }
            return Ok(());
        }

        let updated_view = record_to_view(
            &updated_record,
            &state.catalog_by_uuid,
            &state.uuid_by_locator,
        );
        let publish_epoch = state.catalog_generation.publish_epoch.saturating_add(1);
        state.catalog_generation = update_asset_in_catalog_generation(
            &state.catalog_generation,
            updated_view,
            publish_epoch,
        );
        state
            .catalog_by_uuid
            .insert(updated_record.asset_uuid, updated_record.clone());
        debug_assert!(state
            .preview_scheduler
            .complete_refresh(updated_record.asset_uuid, job.admission_token));

        EditorAssetChangeRecord {
            kind: EditorAssetChangeKind::PreviewChanged,
            catalog_revision: state.catalog_generation.catalog_revision,
            uuid: Some(updated_record.asset_uuid.to_string()),
            locator: Some(updated_record.locator.to_string()),
        }
    };
    change_stream.publish(change);
    if let Some(error) = completion_error {
        return Err(JobError::failed(error));
    }
    Ok(())
}

fn preview_job_is_current(state: &EditorAssetState, job: &PreviewRefreshJob) -> bool {
    state
        .preview_scheduler
        .owns_refresh(job.asset_uuid, job.admission_token)
        && state.catalog_generation.catalog_revision == job.catalog_revision
        && state
            .catalog_generation
            .asset_shared(&job.asset_uuid_text)
            .is_some_and(|current| Arc::ptr_eq(&current, &job.asset_row))
        && state
            .catalog_by_uuid
            .get(&job.asset_uuid)
            .is_some_and(|record| {
                record.source_hash == job.source_hash && record.meta_path == job.meta_path
            })
}

fn release_preview_admission(
    state: &Arc<RwLock<EditorAssetState>>,
    change_stream: &EditorAssetChangeHub,
    job: &PreviewRefreshJob,
) {
    let change = {
        let mut state = state.write().expect("editor asset state lock poisoned");
        release_preview_admission_locked(&mut state, job)
    };
    if let Some(change) = change {
        change_stream.publish(change);
    }
}

fn release_preview_admission_locked(
    state: &mut EditorAssetState,
    job: &PreviewRefreshJob,
) -> Option<EditorAssetChangeRecord> {
    let released = state
        .preview_scheduler
        .complete_refresh(job.asset_uuid, job.admission_token);
    released.then(|| EditorAssetChangeRecord {
        kind: EditorAssetChangeKind::PreviewAdmissionAvailable,
        catalog_revision: state.catalog_generation.catalog_revision,
        uuid: Some(job.asset_uuid_text.clone()),
        locator: Some(job.record.locator.to_string()),
    })
}
