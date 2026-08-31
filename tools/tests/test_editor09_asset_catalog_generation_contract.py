from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANAGER = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/manager"
DEFAULT_MANAGER = MANAGER / "default_editor_asset_manager"
CATALOG_GENERATION = MANAGER / "catalog_generation"
API = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/api.rs"
RECORDS = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/records.rs"
GENERATION = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/generation.rs"
PREVIEW = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/preview.rs"
WORKSPACE = REPO_ROOT / "zircon_editor/src/ui/workbench/project/asset_workspace_state.rs"
RETAINED_SNAPSHOTS = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs"
BACKEND_REFRESH = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/backend_refresh.rs"
CHANGE_STREAM = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/change_stream.rs"
HOST_EVENT_DRAIN = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/events/runtime.rs"
HOST_STARTUP_DRAIN = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/events/startup.rs"
PROJECT_SYNC = MANAGER / "project_sync"
WATCH_PROJECTION = DEFAULT_MANAGER / "watch_projection.rs"
RETAINED_ASSET_REFRESH = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh.rs"
)


class Editor09AssetCatalogGenerationContractTests(unittest.TestCase):
    def test_catalog_generation_is_folder_backed_and_old_projection_owners_are_deleted(self) -> None:
        manager_root = (MANAGER / "mod.rs").read_text(encoding="utf-8")
        default_root = (DEFAULT_MANAGER / "mod.rs").read_text(encoding="utf-8")
        self.assertIn("mod catalog_generation;", manager_root)
        self.assertNotIn("mod folder_projection;", manager_root)
        self.assertNotIn("mod record_to_view;", default_root)
        self.assertNotIn("mod record_access;", default_root)
        for name in (
            "build.rs",
            "details.rs",
            "folders.rs",
            "record.rs",
            "update.rs",
            "tests.rs",
        ):
            self.assertTrue((CATALOG_GENERATION / name).is_file(), name)
        self.assertFalse((MANAGER / "folder_projection.rs").exists())
        self.assertFalse((DEFAULT_MANAGER / "record_to_view.rs").exists())
        self.assertFalse((DEFAULT_MANAGER / "record_access.rs").exists())

    def test_manager_hard_cuts_catalog_query_to_shared_generation(self) -> None:
        api_source = API.read_text(encoding="utf-8")
        snapshot_source = (DEFAULT_MANAGER / "catalog_snapshot.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "fn catalog_snapshot(&self) -> Arc<EditorAssetCatalogGeneration>",
            api_source,
        )
        self.assertIn(
            "Option<Arc<EditorAssetDetailsGeneration>>",
            api_source,
        )
        self.assertIn("Arc::clone(&state.catalog_generation)", snapshot_source)
        for token in ("catalog_by_uuid.values()", ".sort_by(", "build_folder_records"):
            self.assertNotIn(token, snapshot_source)

    def test_project_publish_builds_outside_live_state_lock_and_preview_updates_one_generation_row(self) -> None:
        sync_source = (
            MANAGER / "project_sync/sync_from_project.rs"
        ).read_text(encoding="utf-8")
        preview_source = (
            MANAGER / "preview_refresh/request_preview_refresh.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("build_catalog_generation(", sync_source)
        self.assertLess(
            sync_source.index("build_catalog_generation("),
            sync_source.index("let mut state = self.write_state_recovering_poison()"),
        )
        self.assertIn("update_catalog_record_in_catalog_generation", preview_source)
        self.assertIn("catalog_generation", preview_source)

    def test_generation_owns_shared_rows_indexes_and_cached_details(self) -> None:
        source = GENERATION.read_text(encoding="utf-8")
        self.assertIn("Arc<[Arc<EditorAssetCatalogRecord>]>", source)
        self.assertIn("asset_index_by_uuid: Arc<HashMap<String, usize>>", source)
        self.assertIn("asset_index_by_locator: Arc<HashMap<String, usize>>", source)
        self.assertIn("folder_index_by_id: Arc<HashMap<String, usize>>", source)
        self.assertIn("details_by_asset_index", source)
        update_source = (CATALOG_GENERATION / "update.rs").read_text(encoding="utf-8")
        self.assertNotIn("current.as_ref().clone()", update_source)
        generation_tests = (CATALOG_GENERATION / "tests.rs").read_text(encoding="utf-8")
        self.assertIn(
            "ten_thousand_asset_preview_update_preserves_9999_row_allocations_and_indexes",
            generation_tests,
        )

    def test_arc_generation_reaches_retained_workspace_without_owned_snapshot_clone(self) -> None:
        retained = RETAINED_SNAPSHOTS.read_text(encoding="utf-8")
        workspace = WORKSPACE.read_text(encoding="utf-8")
        self.assertIn("let catalog = editor_asset_manager.catalog_snapshot()", retained)
        self.assertIn("self.runtime.sync_asset_catalog_data(catalog)", retained)
        self.assertIn("self.runtime\n                    .sync_asset_catalog_changes(catalog", retained)
        self.assertNotIn("catalog_snapshot().as_ref().clone()", retained)
        self.assertIn("catalog: Option<Arc<EditorAssetCatalogGeneration>>", workspace)
        self.assertIn("selected_details: Option<Arc<EditorAssetDetailsGeneration>>", workspace)

    def test_preview_io_is_outside_live_state_lock_and_publication_is_generation_safe(self) -> None:
        preview = (MANAGER / "preview_refresh/request_preview_refresh.rs").read_text(
            encoding="utf-8"
        )
        self.assertLess(
            preview.index("generate_preview_artifact("),
            preview.index(
                "let _publish_guard = lock_editor_asset_gate_recovering_poison("
            ),
        )
        self.assertIn("preview_job_is_current", preview)
        self.assertIn("Arc::ptr_eq(&current, &job.asset_row)", preview)
        self.assertIn("catalog_revision == job.catalog_revision", preview)
        self.assertIn("AssetMetaDocument::load(&job.meta_path)", preview)
        self.assertIn("latest_meta.preview_state = updated_record.preview_state", preview)
        self.assertNotIn("latest_meta.save", preview)
        self.assertIn("preview_jobs.submit(", preview)
        self.assertIn("JobCategory::Thumbnail", preview)
        self.assertIn("JobPriority::Background", preview)
        self.assertIn("with_mutex_group(mutex_group)", preview)
        self.assertIn("impl Drop for PreviewRefreshEditorJob", preview)
        self.assertIn("job.admission_token", preview)
        self.assertIn("PreviewAdmissionAvailable", preview)
        self.assertGreaterEqual(preview.count("context.check_cancelled()?;"), 4)
        self.assertNotIn("retry_refresh", preview)
        worker = PREVIEW.read_text(encoding="utf-8")
        self.assertIn("const MAX_PREVIEW_IN_FLIGHT: usize = 64", worker)
        self.assertIn("self.in_flight.len() >= MAX_PREVIEW_IN_FLIGHT", worker)
        self.assertIn(
            "preview_scheduler_bounds_in_flight_assets_without_implicitly_retrying_completion",
            worker,
        )
        self.assertNotIn("fn retry_refresh", worker)
        self.assertIn("stale_job_token_cannot_release_new_generation_admission", worker)
        backend_refresh = BACKEND_REFRESH.read_text(encoding="utf-8")
        preview_branch = backend_refresh.split(
            "EditorAssetChangeKind::PreviewChanged =>", 1
        )[1].split("}", 1)[0]
        self.assertIn("plan.refresh_visible_asset_previews = true", preview_branch)
        self.assertIn("EditorAssetChangeKind::PreviewAdmissionAvailable", backend_refresh)
        sync = (MANAGER / "project_sync/sync_from_project.rs").read_text(encoding="utf-8")
        self.assertIn("Arc::ptr_eq(&state.catalog_generation, &expected_generation)", sync)
        self.assertIn("merge_current_preview_results", sync)

    def test_asset_details_queries_only_the_immutable_generation(self) -> None:
        details = (DEFAULT_MANAGER / "asset_details.rs").read_text(encoding="utf-8")
        self.assertIn("state.catalog_generation.details(uuid)", details)
        for token in ("reference_graph", "sort_by", "record_to_view", "catalog_by_uuid"):
            self.assertNotIn(token, details)

    def test_project_sync_hard_cuts_the_private_input_generation_for_runtime_authority(self) -> None:
        self.assertFalse((PROJECT_SYNC / "source_generation.rs").exists())
        sync = (MANAGER / "project_sync/sync_from_project.rs").read_text(
            encoding="utf-8"
        )
        projection = (PROJECT_SYNC / "record_projection.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("project.catalog_input_generation()", sync)
        self.assertIn("ProjectCatalogInputGeneration", sync)
        self.assertIn("ProjectCatalogInputDelta", sync)
        self.assertIn("delta.is_unchanged()", sync)
        self.assertIn("source_sync_epoch", sync)
        self.assertIn("source_sync_gate", sync)
        self.assertNotIn("EditorAssetProjectSourceGeneration", sync)
        self.assertNotIn("EditorAssetProjectSourceDelta", sync)
        self.assertNotIn("AssetMetaDocument::load", projection)
        self.assertNotIn("load_artifact_by_id", projection)
        self.assertNotIn("AssetImportError", projection)
        self.assertNotIn("Result<", projection)

    def test_manager_owns_one_runtime_index_and_no_mutable_catalog_lookup_maps(self) -> None:
        state = (DEFAULT_MANAGER / "editor_asset_state.rs").read_text(
            encoding="utf-8"
        )
        sync = (PROJECT_SYNC / "sync_from_project.rs").read_text(encoding="utf-8")
        generation = GENERATION.read_text(encoding="utf-8")
        preview = (MANAGER / "preview_refresh/request_preview_refresh.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("asset_index:", state)
        self.assertIn("Option<Arc<Mutex<EditorAssetIndex>>>", state)
        self.assertNotIn("catalog_by_uuid:", state)
        self.assertNotIn("uuid_by_locator:", state)
        self.assertNotIn("source_generation:", state)
        self.assertIn("EditorAssetIndex::from_runtime_project(&project)", sync)
        self.assertIn("state.asset_index", sync)
        self.assertIn("replace_authoritative_projection(candidate_index.clone())", sync)
        self.assertNotIn("= candidate_index.clone()", sync)
        index = (REPO_ROOT / "zircon_editor/src/core/asset/index.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("fn replace_authoritative_projection", index)
        self.assertIn("std::mem::take(&mut self.importing_uuids)", index)
        self.assertIn("retain_transient_state_for_current_registry", index)
        self.assertIn("catalog_records", generation)
        self.assertIn("catalog_record", preview)
        self.assertNotIn("state.catalog_by_uuid", preview)
        self.assertNotIn("state.uuid_by_locator", preview)

    def test_runtime_watch_changes_flow_through_one_batched_index_projection(self) -> None:
        generation = GENERATION.read_text(encoding="utf-8")
        records = RECORDS.read_text(encoding="utf-8")
        api = API.read_text(encoding="utf-8")
        projection = WATCH_PROJECTION.read_text(encoding="utf-8")
        retained_refresh = RETAINED_ASSET_REFRESH.read_text(encoding="utf-8")

        self.assertIn("AssetStateChanged", records)
        self.assertIn("fn project_runtime_asset_changes", api)
        self.assertIn("updated_catalog_records", generation)
        self.assertIn("AssetWatchEvent", projection)
        self.assertIn("apply_watch_events", projection)
        self.assertIn("update_catalog_records_in_catalog_generation", projection)
        self.assertIn("EditorAssetChangeKind::AssetStateChanged", projection)
        self.assertIn("asset_catalog.watch_index_dirty_uuid_count", projection)
        self.assertIn("asset_catalog.watch_index_pending_path_count", projection)
        self.assertIn("project_runtime_asset_changes(&events.asset_changes)", retained_refresh)

        sync = (PROJECT_SYNC / "sync_from_project.rs").read_text(encoding="utf-8")
        self.assertIn("inherit_transient_state_from(&current_asset_index)", sync)
        self.assertLess(
            sync.index("inherit_transient_state_from(&current_asset_index)"),
            sync.index("project_full_catalog("),
        )
        self.assertIn(".row_by_uuid(catalog_input.meta().uuid)", sync)
        self.assertIn("record.dirty |= dirty", sync)
        index = (REPO_ROOT / "zircon_editor/src/core/asset/index.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("pub(crate) fn dirty_count", index)

    def test_runtime_index_projection_failure_remains_a_typed_editor_error(self) -> None:
        sync = (PROJECT_SYNC / "sync_from_project.rs").read_text(encoding="utf-8")
        error = (MANAGER.parent / "error.rs").read_text(encoding="utf-8")

        self.assertIn("Result<(), EditorAssetSyncError>", sync)
        self.assertNotIn("AssetImportError::Parse", sync)
        self.assertIn("Index(#[from] EditorAssetIndexError)", error)
        self.assertIn("Runtime(#[from] AssetImportError)", error)

    def test_reverse_references_use_the_runtime_registry_without_an_editor_graph(self) -> None:
        state = (DEFAULT_MANAGER / "editor_asset_state.rs").read_text(
            encoding="utf-8"
        )
        asset_module = (
            REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/mod.rs"
        ).read_text(encoding="utf-8")
        build = (CATALOG_GENERATION / "build.rs").read_text(encoding="utf-8")
        details = (CATALOG_GENERATION / "details.rs").read_text(encoding="utf-8")

        self.assertNotIn("reference_graph:", state)
        self.assertNotIn("ReferenceGraph", asset_module)
        self.assertFalse(
            (REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/reference_graph.rs").exists()
        )
        self.assertNotIn("ReferenceGraph", build)
        self.assertIn("runtime_registry: &AssetRegistryIndex", build)
        self.assertNotIn("project.asset_registry()", build)
        self.assertIn("AssetRegistryIndex", details)
        self.assertIn("get_referencers_by_uuid", details)
        self.assertNotIn("ReferenceGraph", details)

    def test_project_sync_emits_one_owner_scope_and_structural_work_counters(self) -> None:
        sync = (MANAGER / "project_sync/sync_from_project.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            'zircon_runtime::profile_scope!("editor", "asset_catalog", "sync_from_project")',
            sync,
        )
        for counter in (
            "asset_catalog.runtime_catalog_input_generation_sequence",
            "asset_catalog.projection_catalog_record_count",
            "asset_catalog.runtime_referencer_index_entry_count",
            "asset_catalog.source_sync_superseded_count",
            "asset_catalog.source_generation_superseded_count",
            "asset_catalog.catalog_generation_rebased_count",
            "asset_catalog.shader_ide_refresh_request_count",
            "asset_catalog.catalog_generation_publish_count",
        ):
            self.assertIn(counter, sync)

    def test_change_stream_and_host_drain_are_bounded_coalesced_and_observable(self) -> None:
        stream = CHANGE_STREAM.read_text(encoding="utf-8")
        self.assertIn("const MAX_PENDING_EDITOR_ASSET_CHANGES: usize = 512", stream)
        self.assertIn("HashMap<EditorAssetChangeKey, PendingEditorAssetChange>", stream)
        self.assertIn("VecDeque<EditorAssetChangeKey>", stream)
        self.assertIn("overflow_collapses_to_latest_catalog_generation", stream)
        self.assertIn("fanout_shares_one_immutable_change_payload", stream)
        self.assertIn("silent_subscribe_drop_churn_prunes_dead_owners", stream)
        self.assertIn("coalesced_key_moves_to_tail_without_revision_regression", stream)
        self.assertIn(
            "concurrent_publishers_converge_all_subscribers_to_same_latest_payload",
            stream,
        )
        self.assertIn("Arc::ptr_eq", stream)
        subscription = (DEFAULT_MANAGER / "subscribe_editor_asset_changes.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("unbounded()", subscription)
        broadcast = (DEFAULT_MANAGER / "broadcast.rs").read_text(encoding="utf-8")
        self.assertIn("self.change_stream.publish(change)", broadcast)
        drain = HOST_EVENT_DRAIN.read_text(encoding="utf-8")
        self.assertIn("MAX_ASSET_REFRESH_EVENTS_PER_STREAM", drain)
        self.assertIn("ASSET_REFRESH_DRAIN_TIME_BUDGET", drain)
        self.assertIn("queue_age", drain)
        self.assertIn("pending_len", drain)
        self.assertIn("every_stream_uses_an_independent_time_slice", drain)
        self.assertNotIn("while let Ok(change)", drain)
        startup_drain = HOST_STARTUP_DRAIN.read_text(encoding="utf-8")
        self.assertIn("bootstrap_asset_count", startup_drain)
        self.assertIn("bootstrap_resource_count", startup_drain)
        self.assertNotIn("while self.asset_change_events.try_recv()", startup_drain)
        self.assertNotIn("while self.resource_change_events.try_recv()", startup_drain)


if __name__ == "__main__":
    unittest.main()
