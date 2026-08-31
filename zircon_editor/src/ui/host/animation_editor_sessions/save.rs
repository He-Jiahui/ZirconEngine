use super::super::editor_error::{EditorError, UiAssetSaveStage};
use super::super::editor_ui_host::EditorUiHost;
use crate::core::extension::{
    DocumentAutosavePayload, DocumentSourceWritePublication, DocumentSourceWriteReceipt, SaveCtx,
    SaveReason, ToolkitSaveFailure,
};
use crate::ui::workbench::view::ViewInstanceId;

pub(super) fn save_animation_document(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
    context: &mut SaveCtx,
) -> Result<(), ToolkitSaveFailure> {
    let (written_bytes, receipt) = host.save_animation_editor_canonical(instance_id)?;
    context.record_serialized_project_source_write(written_bytes, receipt)?;
    Ok(())
}

pub(super) fn validate_animation_document_references(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
) -> Result<(), ToolkitSaveFailure> {
    let sessions = host.lock_animation_editor_sessions();
    let entry = sessions.get(instance_id).ok_or_else(|| {
        EditorError::UiAsset(format!(
            "missing animation editor session {}",
            instance_id.0
        ))
    })?;
    entry
        .session
        .document_bytes()
        .map(|_| ())
        .map_err(|error| Box::new(error) as ToolkitSaveFailure)
}

pub(super) fn capture_animation_document_autosave(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
    let source_path = animation_document_autosave_source_path(host, instance_id)?;
    let sessions = host.lock_animation_editor_sessions();
    let entry = sessions.get(instance_id).ok_or_else(|| {
        EditorError::UiAsset(format!(
            "missing animation editor session {}",
            instance_id.0
        ))
    })?;
    let bytes = entry.session.document_bytes()?;
    Ok(DocumentAutosavePayload::new(source_path, bytes))
}

pub(super) fn animation_document_autosave_source_path(
    host: &EditorUiHost,
    instance_id: &ViewInstanceId,
) -> Result<std::path::PathBuf, ToolkitSaveFailure> {
    let document_handle = {
        let sessions = host.lock_animation_editor_sessions();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!(
                "missing animation editor session {}",
                instance_id.0
            ))
        })?;
        entry.session.document().clone()
    };
    let asset_locator = document_handle.read().asset_locator().clone();
    host.resolve_asset_locator_path(&asset_locator)
        .map_err(|error| Box::new(error) as ToolkitSaveFailure)
}

impl EditorUiHost {
    pub fn save_animation_editor(&self, instance_id: &ViewInstanceId) -> Result<(), EditorError> {
        self.ensure_animation_editor_session(instance_id)?;
        self.save_document_toolkit(instance_id, SaveReason::Explicit)?;
        Ok(())
    }

    fn save_animation_editor_canonical(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(u64, DocumentSourceWriteReceipt), EditorError> {
        let (document_handle, expected_source) = {
            let sessions = self.lock_animation_editor_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "missing animation editor session {}",
                    instance_id.0
                ))
            })?;
            (entry.session.document().clone(), entry.disk_source.clone())
        };
        let document = document_handle.read();
        let asset_locator = document.asset_locator().clone();
        let bytes = document
            .document_bytes()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(document);
        let source_path = self.resolve_asset_locator_path(&asset_locator)?;
        let project = self.current_project_snapshot()?.ok_or_else(|| {
            EditorError::UiAsset(
                "cannot save a canonical animation asset without an active project".to_string(),
            )
        })?;
        let publication =
            self.document_toolkits
                .with_source_write(project.paths().root(), &source_path, |source_write| {
                    let outcome = source_write.commit_if_matches(&expected_source, &bytes);
                    if outcome.source_changed() {
                        return Err(EditorError::DocumentSourceChanged {
                            source_path: source_path.clone(),
                        });
                    }
                    let publication = outcome.into_publication().map_err(|source| {
                        EditorError::UiAssetSaveIo {
                            stage: UiAssetSaveStage::AtomicCommit,
                            source_path: source_path.clone(),
                            source,
                        }
                    })?;
                    let mut sessions = self.lock_animation_editor_sessions();
                    let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                        EditorError::UiAsset(format!(
                            "missing animation editor session {}",
                            instance_id.0
                        ))
                    })?;
                    entry.disk_source = bytes.clone();
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
        self.asset_manager()?
            .import_asset(&asset_locator.to_string())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let written_bytes =
            u64::try_from(bytes.len()).map_err(|error| EditorError::UiAsset(error.to_string()))?;
        Ok((written_bytes, receipt))
    }
}
