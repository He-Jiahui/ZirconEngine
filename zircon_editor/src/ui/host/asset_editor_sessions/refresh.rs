use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use super::super::project_access::normalize_ui_asset_asset_id;
#[cfg(test)]
use super::UiAssetDiffSnapshot;
use super::{
    preview_size_for_preset, ui_asset_source_hash, UiAssetExternalConflict,
    UiAssetStaleImportDiagnostic,
};
use crate::ui::asset_editor::{UiAssetEditorRoute, UiAssetEditorSession};
use crate::ui::workbench::view::ViewInstanceId;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetKind};

impl EditorUiHost {
    pub fn refresh_ui_asset_workspace_for_changes(
        &self,
        changed_asset_ids: impl IntoIterator<Item = String>,
    ) -> Result<(), EditorError> {
        let changed_asset_ids = normalize_ui_asset_change_set(changed_asset_ids);
        if changed_asset_ids.is_empty() {
            return Ok(());
        }

        let sync_instances = self.apply_ui_asset_workspace_changes(&changed_asset_ids)?;
        for instance_id in sync_instances {
            self.sync_ui_asset_editor_instance(&instance_id)?;
        }
        Ok(())
    }

    pub fn reload_ui_asset_editor_from_disk(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let (source_path, route) = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            (entry.source_path.clone(), entry.session.route().clone())
        };
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let session = rebuild_ui_asset_session_from_source(route, source.clone())?;
        {
            let mut sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session = session;
            entry.update_disk_baseline(source);
            entry.conflict = None;
            entry.diff_snapshot = None;
            entry.stale_imports.clear();
        }
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(true)
    }

    pub fn keep_ui_asset_editor_local_and_save(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.save_ui_asset_editor(instance_id)
    }

    #[cfg(test)]
    pub fn open_ui_asset_editor_diff_snapshot(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<UiAssetDiffSnapshot>, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        if entry.diff_snapshot.is_none() {
            entry.diff_snapshot = entry.conflict.as_ref().map(UiAssetDiffSnapshot::from);
        }
        Ok(entry.diff_snapshot.clone())
    }

    fn apply_ui_asset_workspace_changes(
        &self,
        changed_asset_ids: &BTreeSet<String>,
    ) -> Result<BTreeSet<ViewInstanceId>, EditorError> {
        let direct_sync = self.apply_direct_ui_asset_changes(changed_asset_ids)?;
        let import_sync = self.apply_import_ui_asset_changes(changed_asset_ids)?;
        Ok(direct_sync.into_iter().chain(import_sync).collect())
    }

    fn apply_direct_ui_asset_changes(
        &self,
        changed_asset_ids: &BTreeSet<String>,
    ) -> Result<BTreeSet<ViewInstanceId>, EditorError> {
        let entries = {
            let sessions = self.lock_ui_asset_sessions();
            sessions
                .iter()
                .filter_map(|(instance_id, entry)| {
                    let asset_id =
                        normalize_ui_asset_asset_id(&entry.session.route().asset_id).to_string();
                    changed_asset_ids
                        .contains(&asset_id)
                        .then(|| (instance_id.clone(), asset_id, entry.source_path.clone()))
                })
                .collect::<Vec<_>>()
        };

        let mut sync_instances = BTreeSet::new();
        for (instance_id, asset_id, source_path) in entries {
            let external_source = match fs::read_to_string(&source_path) {
                Ok(source) => source,
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    let mut sessions = self.lock_ui_asset_sessions();
                    let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                    })?;
                    let local_source = entry.session.source_buffer().text().to_string();
                    entry.conflict = Some(UiAssetExternalConflict::new(
                        asset_id,
                        source_path,
                        entry.disk_source_hash,
                        local_source,
                        String::new(),
                    ));
                    entry.diff_snapshot = None;
                    let _ = sync_instances.insert(instance_id);
                    continue;
                }
                Err(error) => return Err(EditorError::UiAsset(error.to_string())),
            };
            let external_hash = ui_asset_source_hash(&external_source);
            let (route, should_reload) = {
                let mut sessions = self.lock_ui_asset_sessions();
                let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                    EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                })?;
                if external_hash == entry.disk_source_hash {
                    if entry.conflict.is_some() || entry.diff_snapshot.is_some() {
                        entry.conflict = None;
                        entry.diff_snapshot = None;
                        let _ = sync_instances.insert(instance_id.clone());
                    }
                    continue;
                }
                let route = entry.session.route().clone();
                if entry.session.reflection_model().source_dirty {
                    let local_source = entry.session.source_buffer().text().to_string();
                    entry.conflict = Some(UiAssetExternalConflict::new(
                        asset_id,
                        source_path,
                        entry.disk_source_hash,
                        local_source,
                        external_source,
                    ));
                    entry.diff_snapshot = None;
                    let _ = sync_instances.insert(instance_id.clone());
                    continue;
                }
                (route, true)
            };

            if should_reload {
                let session = rebuild_ui_asset_session_from_source(route, external_source.clone())?;
                {
                    let mut sessions = self.lock_ui_asset_sessions();
                    let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                    })?;
                    entry.session = session;
                    entry.update_disk_baseline(external_source);
                    entry.conflict = None;
                    entry.diff_snapshot = None;
                    let route_asset_id = entry.session.route().asset_id.clone();
                    entry.stale_imports.remove(route_asset_id.as_str());
                }
                self.hydrate_ui_asset_editor_imports(&instance_id)?;
                let _ = sync_instances.insert(instance_id);
            }
        }
        Ok(sync_instances)
    }

    fn apply_import_ui_asset_changes(
        &self,
        changed_asset_ids: &BTreeSet<String>,
    ) -> Result<BTreeSet<ViewInstanceId>, EditorError> {
        let dependencies = {
            let sessions = self.lock_ui_asset_sessions();
            sessions
                .iter()
                .filter_map(|(instance_id, entry)| {
                    let (widgets, styles) = entry.session.import_references();
                    let matching = widgets
                        .into_iter()
                        .map(|reference| (reference, UiAssetKind::Widget))
                        .chain(
                            styles
                                .into_iter()
                                .map(|reference| (reference, UiAssetKind::Style)),
                        )
                        .filter(|(reference, _)| {
                            changed_asset_ids
                                .contains(&normalize_ui_asset_asset_id(reference).to_string())
                        })
                        .collect::<Vec<_>>();
                    (!matching.is_empty()).then(|| (instance_id.clone(), matching))
                })
                .collect::<Vec<_>>()
        };

        let mut sync_instances = BTreeSet::new();
        for (instance_id, matching) in dependencies {
            let (widget_refs, style_refs) = {
                let sessions = self.lock_ui_asset_sessions();
                let entry = sessions.get(&instance_id).ok_or_else(|| {
                    EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                })?;
                entry.session.import_references()
            };
            match self.collect_ui_asset_imports_lossy(&widget_refs, &style_refs) {
                Ok((widget_docs, style_docs)) => {
                    let mut sessions = self.lock_ui_asset_sessions();
                    let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                    })?;
                    entry
                        .session
                        .replace_imports(widget_docs, style_docs)
                        .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                    for (reference, _) in &matching {
                        entry
                            .stale_imports
                            .remove(normalize_ui_asset_asset_id(reference));
                    }
                }
                Err(errors) => {
                    let mut sessions = self.lock_ui_asset_sessions();
                    let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                        EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                    })?;
                    for error in errors {
                        entry.stale_imports.insert(error.reference.clone(), error);
                    }
                }
            }
            let _ = sync_instances.insert(instance_id);
        }
        Ok(sync_instances)
    }

    fn collect_ui_asset_imports_lossy(
        &self,
        widget_refs: &[String],
        style_refs: &[String],
    ) -> Result<
        (
            BTreeMap<String, UiAssetDocument>,
            BTreeMap<String, UiAssetDocument>,
        ),
        Vec<UiAssetStaleImportDiagnostic>,
    > {
        let mut widget_docs = BTreeMap::<String, UiAssetDocument>::new();
        let mut style_docs = BTreeMap::<String, UiAssetDocument>::new();
        let mut visited = BTreeSet::new();
        let mut errors = Vec::new();

        for reference in widget_refs {
            if let Err(message) = self.try_collect_ui_asset_import_document(
                reference,
                UiAssetKind::Widget,
                &mut widget_docs,
                &mut style_docs,
                &mut visited,
            ) {
                errors.push(UiAssetStaleImportDiagnostic {
                    reference: normalize_ui_asset_asset_id(reference).to_string(),
                    message,
                });
            }
        }
        for reference in style_refs {
            if let Err(message) = self.try_collect_ui_asset_import_document(
                reference,
                UiAssetKind::Style,
                &mut widget_docs,
                &mut style_docs,
                &mut visited,
            ) {
                errors.push(UiAssetStaleImportDiagnostic {
                    reference: normalize_ui_asset_asset_id(reference).to_string(),
                    message,
                });
            }
        }

        if errors.is_empty() {
            Ok((widget_docs, style_docs))
        } else {
            Err(errors)
        }
    }
}

pub(super) fn normalize_ui_asset_change_set(
    changed_asset_ids: impl IntoIterator<Item = String>,
) -> BTreeSet<String> {
    changed_asset_ids
        .into_iter()
        .map(|asset_id| normalize_ui_asset_asset_id(&asset_id).to_string())
        .collect()
}

fn rebuild_ui_asset_session_from_source(
    route: UiAssetEditorRoute,
    source: String,
) -> Result<UiAssetEditorSession, EditorError> {
    let preview_size = preview_size_for_preset(route.preview_preset);
    UiAssetEditorSession::from_source(route, source, preview_size)
        .map_err(|error| EditorError::UiAsset(error.to_string()))
}
