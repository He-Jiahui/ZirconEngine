use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::{
    ProjectAssetGenerationToken, ProjectAssetManager, ProjectGenerationCommitOutcome,
};
use zircon_runtime::asset::project::{
    ProjectCatalogInputDelta, ProjectCatalogInputGeneration, ProjectCatalogInputRecord,
    ProjectManager,
};
use zircon_runtime::core::framework::render::ShaderIdePreviewVariant;
use zircon_runtime::core::resource::{ResourceKind, ResourceState};
use zircon_runtime::graphics::write_shader_ide_env_for_project;

use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, EditorAssetSyncError, PreviewCache, PreviewScheduler,
};

use super::super::super::{EditorAssetChangeKind, EditorAssetChangeRecord};
use super::super::catalog_generation::build_catalog_generation;
use super::super::default_editor_asset_manager::{
    lock_editor_asset_gate_recovering_poison, DefaultEditorAssetManager,
};
use super::record_projection::project_catalog_record;
use crate::core::asset::EditorAssetIndex;

impl DefaultEditorAssetManager {
    pub(super) fn sync_from_project(
        &self,
        project: ProjectManager,
    ) -> Result<(), EditorAssetSyncError> {
        self.sync_from_project_with_runtime_generation(project, None)
    }

    pub(super) fn sync_from_runtime_project_generation(
        &self,
        project_asset_manager: &ProjectAssetManager,
        project: ProjectManager,
        generation: &ProjectAssetGenerationToken,
    ) -> Result<(), EditorAssetSyncError> {
        self.sync_from_project_with_runtime_generation(
            project,
            Some((project_asset_manager, generation)),
        )
    }

    fn sync_from_project_with_runtime_generation(
        &self,
        project: ProjectManager,
        runtime_generation: Option<(&ProjectAssetManager, &ProjectAssetGenerationToken)>,
    ) -> Result<(), EditorAssetSyncError> {
        zircon_runtime::profile_scope!("editor", "asset_catalog", "sync_from_project");
        // Registration and winner commit use the same source gate. A newer request
        // either cancels this epoch before shader I/O or starts after this commit.
        let source_sync_epoch = {
            let _registration_guard =
                lock_editor_asset_gate_recovering_poison(self.source_sync_gate.as_ref());
            self.advance_source_sync_epoch()
        };
        let pending_source_generation = project.catalog_input_generation();
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.runtime_catalog_input_generation_sequence",
            pending_source_generation.sequence()
        );
        let (mut expected_generation, expected_source_generation, current_asset_index) = {
            let state = self.read_state_recovering_poison();
            (
                Arc::clone(&state.catalog_generation),
                state.asset_index.as_ref().and_then(|index| {
                    index
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .catalog_input_generation()
                        .cloned()
                }),
                state.asset_index.clone(),
            )
        };
        let runtime_delta = expected_source_generation
            .as_deref()
            .map(|previous| pending_source_generation.delta_since(previous));
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.resource_delta_unchanged",
            runtime_delta
                .as_ref()
                .is_some_and(ProjectCatalogInputDelta::is_unchanged) as u8
        );
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.runtime_catalog_input_initial_sync_count",
            runtime_delta.is_none() as u8
        );
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.resource_delta_added_count",
            runtime_delta.as_ref().map_or(0, |delta| delta.added.len())
        );
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.resource_delta_modified_count",
            runtime_delta
                .as_ref()
                .map_or(0, |delta| delta.modified.len())
        );
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.resource_delta_removed_count",
            runtime_delta
                .as_ref()
                .map_or(0, |delta| delta.removed.len())
        );
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.resource_delta_renamed_count",
            runtime_delta
                .as_ref()
                .map_or(0, |delta| delta.renamed.len())
        );
        if runtime_delta
            .as_ref()
            .is_some_and(|delta| delta.is_unchanged())
        {
            zircon_runtime::profile_counter!(
                "editor",
                "asset_catalog.projection_unchanged_count",
                1_u8
            );
            return Ok(());
        }

        let preview_cache = PreviewCache::new(project.paths().cache_root())?;
        let mut candidate_index = EditorAssetIndex::from_runtime_project(&project)?;
        if let Some(current_asset_index) = current_asset_index {
            let current_asset_index = current_asset_index
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            candidate_index.inherit_transient_state_from(&current_asset_index);
        }
        debug_assert!(candidate_index
            .catalog_input_generation()
            .is_some_and(|candidate| Arc::ptr_eq(candidate, &pending_source_generation)));
        let mut catalog_by_uuid = HashMap::new();
        let mut uuid_by_locator = HashMap::new();
        project_full_catalog(
            &candidate_index,
            &preview_cache,
            &mut catalog_by_uuid,
            &mut uuid_by_locator,
        );
        zircon_runtime::profile_counter!("editor", "asset_catalog.projection_full_count", 1_u8);
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.projection_catalog_record_count",
            catalog_by_uuid.len()
        );
        let mut preview_scheduler = preview_scheduler_for(&catalog_by_uuid);

        {
            let state = self.read_state_recovering_poison();
            if std::sync::Arc::ptr_eq(&state.catalog_generation, &expected_generation) {
                merge_current_preview_results(&mut catalog_by_uuid, &state.catalog_generation);
                preview_scheduler = preview_scheduler_for(&catalog_by_uuid);
            }
        }

        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.runtime_referencer_index_entry_count",
            project.asset_registry().len()
        );
        let refresh_shader_ide = runtime_delta
            .as_ref()
            .is_none_or(catalog_delta_affects_shader);
        zircon_runtime::profile_counter!(
            "editor",
            "asset_catalog.shader_ide_refresh_request_count",
            refresh_shader_ide as u8
        );
        if refresh_shader_ide {
            refresh_shader_ide_env_after_import(&project)?;
        }
        let primary_asset_root = project.primary_project_asset_root()?.to_path_buf();
        let mut pending_project = Some(project);
        let mut pending_candidate_index = Some(candidate_index);
        let mut pending_preview_cache = Some(preview_cache);
        let mut pending_primary_asset_root = Some(primary_asset_root);

        enum CatalogCommitAttempt {
            Published(EditorAssetChangeRecord),
            Rebase(Arc<crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration>),
            Superseded,
        }

        loop {
            let (catalog_revision, publish_epoch) = expected_generation.next_catalog_identity();
            let project = pending_project
                .as_ref()
                .expect("an unpublished project sync retains its project candidate");
            let candidate_index = pending_candidate_index
                .as_ref()
                .expect("an unpublished project sync retains its index candidate");
            let primary_asset_root = pending_primary_asset_root
                .as_ref()
                .expect("an unpublished project sync retains its asset root");
            let project_root = project.paths().root().to_path_buf();
            let cache_root = project.paths().cache_root().to_path_buf();
            let project_name = project.manifest().name.clone();
            let default_scene_uri = project.manifest().default_scene.clone();
            let catalog_generation = build_catalog_generation(
                project,
                candidate_index.runtime_registry(),
                primary_asset_root,
                catalog_revision,
                publish_epoch,
                &catalog_by_uuid,
                &uuid_by_locator,
            );
            let source_commit_guard =
                lock_editor_asset_gate_recovering_poison(self.source_sync_gate.as_ref());
            if self.source_sync_epoch.load(Ordering::Acquire) != source_sync_epoch {
                zircon_runtime::profile_counter!(
                    "editor",
                    "asset_catalog.source_sync_superseded_count",
                    1_u8
                );
                return Ok(());
            }
            let commit = || {
                let _publish_guard =
                    lock_editor_asset_gate_recovering_poison(self.publish_gate.as_ref());
                let mut state = self.write_state_recovering_poison();
                if !same_catalog_input_generation(
                    state.asset_index.as_ref(),
                    &expected_source_generation,
                ) {
                    zircon_runtime::profile_counter!(
                        "editor",
                        "asset_catalog.source_generation_superseded_count",
                        1_u8
                    );
                    return CatalogCommitAttempt::Superseded;
                }
                if !std::sync::Arc::ptr_eq(&state.catalog_generation, &expected_generation) {
                    if state.catalog_generation.catalog_revision
                        != expected_generation.catalog_revision
                    {
                        zircon_runtime::profile_counter!(
                            "editor",
                            "asset_catalog.catalog_generation_superseded_count",
                            1_u8
                        );
                        return CatalogCommitAttempt::Superseded;
                    }
                    return CatalogCommitAttempt::Rebase(Arc::clone(&state.catalog_generation));
                }
                state.project_root = Some(project_root);
                state.assets_root = pending_primary_asset_root.take();
                state.cache_root = Some(cache_root);
                state.project_name = project_name;
                state.default_scene_uri = Some(default_scene_uri);
                state.catalog_generation = catalog_generation;
                state.project = pending_project.take();
                let candidate_index = pending_candidate_index
                    .take()
                    .expect("a winning project sync owns its index candidate");
                let asset_index = match state.asset_index.as_ref() {
                    Some(existing) => {
                        existing
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .replace_authoritative_projection(candidate_index);
                        Arc::clone(existing)
                    }
                    None => Arc::new(std::sync::Mutex::new(candidate_index)),
                };
                state.asset_index = Some(asset_index);
                state.preview_cache = pending_preview_cache.take();
                state.preview_scheduler = std::mem::take(&mut preview_scheduler);
                zircon_runtime::profile_counter!(
                    "editor",
                    "asset_catalog.catalog_generation_publish_count",
                    1_u8
                );

                CatalogCommitAttempt::Published(EditorAssetChangeRecord {
                    kind: EditorAssetChangeKind::CatalogChanged,
                    catalog_revision,
                    uuid: None,
                    locator: None,
                })
            };
            let attempt = match runtime_generation {
                Some((project_asset_manager, generation)) => {
                    match project_asset_manager.commit_if_project_generation(generation, commit) {
                        ProjectGenerationCommitOutcome::Committed(attempt) => attempt,
                        ProjectGenerationCommitOutcome::Superseded { .. } => {
                            zircon_runtime::profile_counter!(
                                "editor",
                                "asset_catalog.runtime_project_generation_superseded_count",
                                1_u8
                            );
                            return Ok(());
                        }
                    }
                }
                None => commit(),
            };
            drop(source_commit_guard);
            match attempt {
                CatalogCommitAttempt::Published(change) => {
                    self.broadcast(change);
                    return Ok(());
                }
                CatalogCommitAttempt::Rebase(current_generation) => {
                    zircon_runtime::profile_counter!(
                        "editor",
                        "asset_catalog.catalog_generation_rebased_count",
                        1_u8
                    );
                    merge_current_preview_results(&mut catalog_by_uuid, &current_generation);
                    preview_scheduler = preview_scheduler_for(&catalog_by_uuid);
                    expected_generation = current_generation;
                }
                CatalogCommitAttempt::Superseded => return Ok(()),
            }
        }
    }
}

fn same_catalog_input_generation(
    current: Option<&Arc<std::sync::Mutex<EditorAssetIndex>>>,
    expected: &Option<Arc<ProjectCatalogInputGeneration>>,
) -> bool {
    match (
        current.and_then(|index| {
            index
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .catalog_input_generation()
                .cloned()
        }),
        expected,
    ) {
        (Some(current), Some(expected)) => Arc::ptr_eq(&current, expected),
        (None, None) => true,
        _ => false,
    }
}

fn project_full_catalog(
    asset_index: &EditorAssetIndex,
    preview_cache: &PreviewCache,
    catalog_by_uuid: &mut HashMap<zircon_runtime::asset::AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: &mut HashMap<
        zircon_runtime::asset::AssetUri,
        zircon_runtime::asset::AssetUuid,
    >,
) {
    let catalog_input_generation = asset_index
        .catalog_input_generation()
        .expect("runtime project index always owns its catalog input generation");
    for catalog_input in catalog_input_generation.records() {
        insert_projected_record(
            preview_cache,
            catalog_input,
            asset_index
                .row_by_uuid(catalog_input.meta().uuid)
                .is_some_and(|row| row.dirty()),
            catalog_by_uuid,
            uuid_by_locator,
        );
    }
}

fn insert_projected_record(
    preview_cache: &PreviewCache,
    catalog_input: &ProjectCatalogInputRecord,
    dirty: bool,
    catalog_by_uuid: &mut HashMap<zircon_runtime::asset::AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: &mut HashMap<
        zircon_runtime::asset::AssetUri,
        zircon_runtime::asset::AssetUuid,
    >,
) {
    let Some(mut record) = project_catalog_record(preview_cache, catalog_input) else {
        return;
    };
    record.dirty |= dirty;
    uuid_by_locator.insert(record.locator.clone(), record.asset_uuid);
    catalog_by_uuid.insert(record.asset_uuid, record);
}

fn catalog_delta_affects_shader(delta: &ProjectCatalogInputDelta) -> bool {
    delta.project_metadata_changed
        || delta
            .added
            .iter()
            .chain(&delta.modified)
            .chain(&delta.removed)
            .any(|record| record.resource().kind == ResourceKind::Shader)
        || delta.renamed.iter().any(|rename| {
            rename.previous.resource().kind == ResourceKind::Shader
                || rename.current.resource().kind == ResourceKind::Shader
        })
}

fn merge_current_preview_results(
    pending: &mut HashMap<zircon_runtime::asset::AssetUuid, AssetCatalogRecord>,
    current: &crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration,
) {
    for (uuid, pending_record) in pending {
        let Some(current_record) = current.catalog_record(&uuid.to_string()) else {
            continue;
        };
        if current_record.source_hash != pending_record.source_hash
            || current_record.meta_path != pending_record.meta_path
        {
            continue;
        }
        pending_record.preview_state = current_record.preview_state;
        pending_record.preview_artifact_path = current_record.preview_artifact_path.clone();
        pending_record.dirty = current_record.dirty;
        pending_record.meta.preview_state = current_record.meta.preview_state;
    }
}

fn preview_scheduler_for(
    catalog_by_uuid: &HashMap<zircon_runtime::asset::AssetUuid, AssetCatalogRecord>,
) -> PreviewScheduler {
    let mut scheduler = PreviewScheduler::default();
    for record in catalog_by_uuid.values().filter(|record| record.dirty) {
        scheduler.mark_dirty(record.asset_uuid);
    }
    scheduler
}

fn refresh_shader_ide_env_after_import(
    project: &ProjectManager,
) -> Result<(), EditorAssetSyncError> {
    let has_ready_shader = project
        .registry()
        .values()
        .any(|record| record.kind == ResourceKind::Shader && record.state == ResourceState::Ready);
    if !has_ready_shader {
        return Ok(());
    }

    let preview_variants = [ShaderIdePreviewVariant::default_forward()];
    write_shader_ide_env_for_project(project, None, &preview_variants)
        .map(|_| ())
        .map_err(|error| {
            zircon_runtime::asset::importer::AssetImportError::ShaderValidation(format!(
                "refresh shader IDE environment: {error}"
            ))
        })
        .map_err(EditorAssetSyncError::from)
}
