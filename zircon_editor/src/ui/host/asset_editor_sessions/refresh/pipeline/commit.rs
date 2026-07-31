use std::collections::{BTreeMap, BTreeSet};

use crate::ui::host::asset_editor_sessions::{
    ui_asset_source_hash, UiAssetExternalConflict, UiAssetStaleImportDiagnostic,
};
use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;

use super::result::{UiAssetDirectRefreshOutcome, UiAssetRefreshBatch, UiAssetRefreshCommitReport};

impl EditorUiHost {
    pub(in crate::ui::host::asset_editor_sessions) fn commit_ui_asset_refresh_batch(
        &self,
        batch: UiAssetRefreshBatch,
    ) -> Result<UiAssetRefreshCommitReport, EditorError> {
        let current_project_root = self
            .current_project_snapshot()?
            .as_ref()
            .map(|project| project.paths().root().to_path_buf());
        if current_project_root != batch.project_root {
            return Ok(UiAssetRefreshCommitReport {
                sync_instances: BTreeSet::new(),
                requeue_asset_ids: BTreeSet::new(),
                retry_asset_ids: BTreeSet::new(),
            });
        }
        // This gate makes dependency validation and every reverse-edge update
        // one commit epoch; no filesystem or parsing work runs while held.
        let mut dependency_generation = self.lock_ui_asset_dependency_generation();
        if dependency_generation.generation() != batch.dependency_generation {
            return Ok(UiAssetRefreshCommitReport {
                sync_instances: BTreeSet::new(),
                requeue_asset_ids: batch.changed_asset_ids,
                retry_asset_ids: BTreeSet::new(),
            });
        }

        let mut sync_instances = BTreeSet::new();
        let mut requeue_asset_ids = BTreeSet::new();
        let mut retry_asset_ids = BTreeSet::new();
        for result in batch.direct_results {
            let mut next_dependencies = None;
            let mut sessions = self.lock_ui_asset_sessions();
            let Some(entry) = sessions.get_mut(&result.plan.instance_id) else {
                continue;
            };
            let source_fingerprint = ui_asset_source_hash(entry.session.source_buffer().text());
            let current_asset_id = normalize_ui_asset_asset_id(&entry.session.route().asset_id);
            if source_fingerprint != result.plan.source_fingerprint
                || entry.disk_source_hash != result.plan.disk_source_hash
                || current_asset_id != result.plan.asset_id
            {
                requeue_asset_ids.extend(batch.changed_asset_ids.iter().cloned());
                continue;
            }

            match result.outcome {
                UiAssetDirectRefreshOutcome::Unchanged => {
                    entry.conflict = None;
                    entry.diff_snapshot = None;
                    entry.stale_imports.remove(&result.plan.asset_id);
                }
                UiAssetDirectRefreshOutcome::Missing => {
                    apply_external_conflict(
                        entry,
                        result.plan.asset_id.clone(),
                        result.plan.source_path,
                        String::new(),
                        "UI asset source was removed".to_string(),
                    );
                }
                UiAssetDirectRefreshOutcome::Conflict { external_source } => {
                    apply_external_conflict(
                        entry,
                        result.plan.asset_id.clone(),
                        result.plan.source_path,
                        external_source,
                        "UI asset changed on disk while local authoring state was dirty"
                            .to_string(),
                    );
                }
                UiAssetDirectRefreshOutcome::Invalid {
                    external_source,
                    message,
                } => {
                    apply_external_conflict(
                        entry,
                        result.plan.asset_id.clone(),
                        result.plan.source_path,
                        external_source,
                        message,
                    );
                }
                UiAssetDirectRefreshOutcome::Failed { message } => {
                    entry.stale_imports.insert(
                        result.plan.asset_id.clone(),
                        UiAssetStaleImportDiagnostic {
                            reference: result.plan.asset_id.clone(),
                            message,
                        },
                    );
                    retry_asset_ids.extend(batch.changed_asset_ids.iter().cloned());
                }
                UiAssetDirectRefreshOutcome::Reloaded {
                    external_source,
                    mut session,
                    imports,
                    import_errors,
                } => {
                    let dependencies = imports.dependencies;
                    let documents = imports.documents;
                    if import_errors.is_empty() {
                        match session.replace_resolved_imports(
                            documents.widgets,
                            documents.styles,
                            documents.v2_widgets,
                            documents.v2_styles,
                        ) {
                            Ok(()) => {
                                entry.session = session;
                                entry.update_disk_baseline(external_source);
                                entry.conflict = None;
                                entry.diff_snapshot = None;
                                entry.stale_imports.clear();
                                next_dependencies = Some(dependencies);
                            }
                            Err(error) => {
                                entry.stale_imports.insert(
                                    result.plan.asset_id.clone(),
                                    UiAssetStaleImportDiagnostic {
                                        reference: result.plan.asset_id.clone(),
                                        message: error.to_string(),
                                    },
                                );
                                retry_asset_ids.extend(batch.changed_asset_ids.iter().cloned());
                            }
                        }
                    } else {
                        let message = import_errors
                            .iter()
                            .map(|error| error.message.as_str())
                            .collect::<Vec<_>>()
                            .join("; ");
                        apply_external_conflict(
                            entry,
                            result.plan.asset_id.clone(),
                            result.plan.source_path,
                            external_source,
                            message,
                        );
                        entry
                            .stale_imports
                            .extend(diagnostics_by_reference(import_errors));
                        next_dependencies = Some(dependencies);
                    }
                }
            }
            let instance_id = result.plan.instance_id;
            let _ = sync_instances.insert(instance_id.clone());
            drop(sessions);
            if let Some(dependencies) = next_dependencies {
                dependency_generation.replace_dependencies(instance_id, dependencies);
            }
        }

        for result in batch.import_results {
            let mut sessions = self.lock_ui_asset_sessions();
            let Some(entry) = sessions.get_mut(&result.plan.instance_id) else {
                continue;
            };
            let source_fingerprint = ui_asset_source_hash(entry.session.source_buffer().text());
            if source_fingerprint != result.plan.source_fingerprint {
                requeue_asset_ids.extend(batch.changed_asset_ids.iter().cloned());
                continue;
            }
            let dependencies = result.imports.dependencies;
            let documents = result.imports.documents;
            if result.errors.is_empty() {
                match entry.session.replace_resolved_imports(
                    documents.widgets,
                    documents.styles,
                    documents.v2_widgets,
                    documents.v2_styles,
                ) {
                    Ok(()) => entry.stale_imports.clear(),
                    Err(error) => {
                        let message = error.to_string();
                        for reference in &batch.changed_asset_ids {
                            entry.stale_imports.insert(
                                reference.clone(),
                                UiAssetStaleImportDiagnostic {
                                    reference: reference.clone(),
                                    message: message.clone(),
                                },
                            );
                        }
                        retry_asset_ids.extend(batch.changed_asset_ids.iter().cloned());
                    }
                }
            } else {
                entry.stale_imports = diagnostics_by_reference(result.errors);
            }
            let instance_id = result.plan.instance_id;
            let _ = sync_instances.insert(instance_id.clone());
            drop(sessions);
            dependency_generation.replace_dependencies(instance_id, dependencies);
        }
        drop(dependency_generation);

        Ok(UiAssetRefreshCommitReport {
            sync_instances,
            requeue_asset_ids,
            retry_asset_ids,
        })
    }
}

fn apply_external_conflict(
    entry: &mut super::super::super::UiAssetWorkspaceEntry,
    asset_id: String,
    source_path: std::path::PathBuf,
    external_source: String,
    message: String,
) {
    let local_source = entry.session.source_buffer().text().to_string();
    entry.conflict = Some(UiAssetExternalConflict::new(
        asset_id.clone(),
        source_path,
        entry.disk_source_hash,
        local_source,
        external_source,
    ));
    entry.diff_snapshot = None;
    entry.stale_imports.insert(
        asset_id.clone(),
        UiAssetStaleImportDiagnostic {
            reference: asset_id,
            message,
        },
    );
}

fn diagnostics_by_reference(
    diagnostics: Vec<UiAssetStaleImportDiagnostic>,
) -> BTreeMap<String, UiAssetStaleImportDiagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| (diagnostic.reference.clone(), diagnostic))
        .collect()
}
