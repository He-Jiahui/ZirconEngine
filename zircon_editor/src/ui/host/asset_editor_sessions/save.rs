use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::super::editor_error::{EditorError, UiAssetSaveStage};
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;
use zircon_runtime::core::resource::io::atomic_write_new;

use super::super::project_access::normalize_ui_asset_asset_id;
use super::{ui_asset_source_digest, UiAssetExternalConflict};
use crate::core::extension::{
    DocumentAutosavePayload, DocumentSourceWritePublication, DocumentSourceWriteReceipt, SaveCtx,
    SaveReason, ToolkitSaveFailure,
};

pub(super) fn save_ui_asset_document(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
    context: &mut SaveCtx,
) -> Result<(), ToolkitSaveFailure> {
    let (written_bytes, receipt) = host.save_ui_asset_editor_canonical(instance_id)?;
    context.record_serialized_project_source_write(written_bytes, receipt)?;
    Ok(())
}

pub(super) fn validate_ui_asset_document_references(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
) -> Result<(), ToolkitSaveFailure> {
    let sessions = host.lock_ui_asset_sessions();
    let entry = sessions.get(instance_id).ok_or_else(|| {
        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
    })?;
    if !entry.stale_imports.is_empty() {
        let references = entry
            .stale_imports
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(Box::new(EditorError::UiAsset(format!(
            "UI asset {} has unresolved imports: {references}",
            entry.session.route().asset_id
        ))));
    }
    entry
        .session
        .canonical_source()
        .map(|_| ())
        .map_err(|error| Box::new(EditorError::UiAsset(error.to_string())) as ToolkitSaveFailure)
}

pub(super) fn capture_ui_asset_document_autosave(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
    let source_path = ui_asset_document_autosave_source_path(host, instance_id)?;
    let sessions = host.lock_ui_asset_sessions();
    let entry = sessions.get(instance_id).ok_or_else(|| {
        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
    })?;
    let source = entry
        .session
        .canonical_source()
        .map_err(|error| EditorError::UiAsset(error.to_string()))?;
    Ok(DocumentAutosavePayload::new(
        source_path,
        source.into_bytes(),
    ))
}

pub(super) fn ui_asset_document_autosave_source_path(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
) -> Result<PathBuf, ToolkitSaveFailure> {
    let sessions = host.lock_ui_asset_sessions();
    let entry = sessions.get(instance_id).ok_or_else(|| {
        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
    })?;
    Ok(entry.source_path.clone())
}

impl EditorUiHost {
    pub fn save_ui_asset_editor(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        self.save_document_toolkit(instance_id, SaveReason::Explicit)?;
        let sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        Ok(entry.disk_source.clone())
    }

    fn save_ui_asset_editor_canonical(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(u64, DocumentSourceWriteReceipt), EditorError> {
        let (saved, asset_id, source_path, expected_source, expected_digest, source_revision) = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let saved = entry
                .session
                .canonical_source()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            (
                saved,
                entry.session.route().asset_id.clone(),
                entry.source_path.clone(),
                entry.disk_source.clone(),
                entry.disk_source_digest,
                entry.session.source_revision(),
            )
        };
        let project = self.current_project_snapshot()?.ok_or_else(|| {
            EditorError::UiAsset(
                "cannot save a canonical UI asset without an active project".to_string(),
            )
        })?;
        let publication = self
            .document_toolkits
            .with_source_write(project.paths().root(), &source_path, |source_write| {
                let write_outcome =
                    source_write.commit_if_matches(expected_source.as_bytes(), saved.as_bytes());
                if write_outcome.source_changed() {
                    return Err(self.record_ui_asset_save_conflict(
                        instance_id,
                        asset_id.clone(),
                        source_path.clone(),
                        expected_digest,
                    ));
                }
                let publication = write_outcome.into_publication().map_err(|source| {
                    EditorError::UiAssetSaveIo {
                        stage: UiAssetSaveStage::AtomicCommit,
                        source_path: source_path.clone(),
                        source,
                    }
                })?;
                {
                    let mut sessions = self.lock_ui_asset_sessions();
                    let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                    })?;
                    entry.update_disk_baseline(saved.clone());
                    entry.conflict = None;
                    entry.diff_snapshot = None;
                    if matches!(&publication, DocumentSourceWritePublication::Durable(_)) {
                        let _ = entry
                            .session
                            .mark_canonical_source_persisted(source_revision, saved.clone());
                    }
                }
                Ok(publication)
            })
            .map_err(|source| EditorError::UiAssetSaveIo {
                stage: UiAssetSaveStage::AtomicCommit,
                source_path: source_path.clone(),
                source,
            })??;
        let receipt = match publication {
            DocumentSourceWritePublication::Durable(receipt) => receipt,
            DocumentSourceWritePublication::PublishedNotDurable(source) => {
                return Err(EditorError::UiAssetSaveIo {
                    stage: UiAssetSaveStage::DurabilityBarrier,
                    source_path,
                    source,
                });
            }
        };
        if asset_id.starts_with("res://") {
            let normalized = normalize_ui_asset_asset_id(&asset_id).to_string();
            let _ = self.asset_manager()?.import_asset(&normalized);
            self.refresh_ui_asset_workspace_for_changes(vec![normalized])?;
        }
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        let written_bytes = u64::try_from(saved.len())
            .map_err(|_| EditorError::UiAsset("saved ui asset size exceeds u64".to_string()))?;
        Ok((written_bytes, receipt))
    }

    pub fn save_ui_asset_editor_local_copy(
        &self,
        instance_id: &ViewInstanceId,
        copy_path: impl AsRef<Path>,
    ) -> Result<String, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let saved = self.canonical_ui_asset_editor_source(instance_id)?;
        atomic_write_new(copy_path.as_ref(), saved.as_bytes()).map_err(|source| {
            EditorError::UiAssetSaveIo {
                stage: UiAssetSaveStage::LocalCopyPublish,
                source_path: copy_path.as_ref().to_path_buf(),
                source,
            }
        })?;
        Ok(saved)
    }

    pub fn save_ui_asset_editor_local_copy_next_to_source(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<PathBuf, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let (source_path, saved) = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let saved = entry
                .session
                .canonical_source()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            (entry.source_path.clone(), saved)
        };
        for index in 0..LOCAL_COPY_CANDIDATE_LIMIT {
            let copy_path = local_copy_path(&source_path, index);
            match atomic_write_new(&copy_path, saved.as_bytes()) {
                Ok(()) => return Ok(copy_path),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(source) => {
                    return Err(EditorError::UiAssetSaveIo {
                        stage: UiAssetSaveStage::LocalCopyPublish,
                        source_path: copy_path,
                        source,
                    });
                }
            }
        }
        Err(EditorError::UiAsset(format!(
            "could not allocate a local copy path for {}",
            source_path.display()
        )))
    }

    fn canonical_ui_asset_editor_source(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        let sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .canonical_source()
            .map_err(|error| EditorError::UiAsset(error.to_string()))
    }

    fn record_ui_asset_save_conflict(
        &self,
        instance_id: &ViewInstanceId,
        asset_id: String,
        source_path: PathBuf,
        expected_digest: blake3::Hash,
    ) -> EditorError {
        let external_source = match fs::read_to_string(&source_path) {
            Ok(source) => source,
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            Err(source) => {
                return EditorError::UiAssetSaveIo {
                    stage: UiAssetSaveStage::SourceRead,
                    source_path,
                    source,
                };
            }
        };
        let actual_digest = ui_asset_source_digest(&external_source);
        let mut sessions = self.lock_ui_asset_sessions();
        if let Some(entry) = sessions.get_mut(instance_id) {
            let local_source = entry.session.source_buffer().text().to_string();
            entry.conflict = Some(UiAssetExternalConflict::new(
                asset_id.clone(),
                source_path.clone(),
                expected_digest,
                local_source,
                external_source,
            ));
            entry.diff_snapshot = None;
        }
        EditorError::UiAssetSourceConflict {
            asset_id,
            source_path,
            expected_digest,
            actual_digest,
        }
    }
}

const LOCAL_COPY_CANDIDATE_LIMIT: u32 = 1000;

fn local_copy_path(source_path: &Path, index: u32) -> PathBuf {
    let parent = source_path.parent().unwrap_or_else(|| Path::new(""));
    let stem = source_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("ui_asset");
    let extension = source_path.extension().and_then(|value| value.to_str());
    let suffix = if index == 0 {
        String::new()
    } else {
        format!("-{index}")
    };
    let file_name = match extension {
        Some(extension) if !extension.is_empty() => {
            format!("{stem}.local-copy{suffix}.{extension}")
        }
        _ => format!("{stem}.local-copy{suffix}"),
    };
    parent.join(file_name)
}
