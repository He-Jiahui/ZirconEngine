from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANAGER = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/manager"
DEFAULT_MANAGER = MANAGER / "default_editor_asset_manager"
CATALOG_GENERATION = MANAGER / "catalog_generation"
API = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/api.rs"
GENERATION = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/generation.rs"
PREVIEW = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/preview.rs"
WORKSPACE = REPO_ROOT / "zircon_editor/src/ui/workbench/project/asset_workspace_state.rs"
RETAINED_SNAPSHOTS = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/snapshots.rs"
BACKEND_REFRESH = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/backend_refresh.rs"
CHANGE_STREAM = REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/change_stream.rs"
HOST_EVENT_DRAIN = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/events/runtime.rs"
HOST_STARTUP_DRAIN = REPO_ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/events/startup.rs"
SOURCE_GENERATION = MANAGER / "project_sync/source_generation.rs"


class Editor09AssetCatalogGenerationContractTests(unittest.TestCase):
    def test_catalog_generation_is_folder_backed_and_old_projection_owners_are_deleted(self) -> None:
        manager_root = (MANAGER / "mod.rs").read_text(encoding="utf-8")
        default_root = (DEFAULT_MANAGER / "mod.rs").read_text(encoding="utf-8")
        self.assertIn("mod catalog_generation;", manager_root)
        self.assertNotIn("mod folder_projection;", manager_root)
        self.assertNotIn("mod record_to_view;", default_root)
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
            sync_source.index(".state\n                .write()"),
        )
        self.assertIn("update_asset_in_catalog_generation", preview_source)
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
        self.assertIn("sync_asset_catalog(editor_asset_manager.catalog_snapshot())", retained)
        self.assertNotIn("catalog_snapshot().as_ref().clone()", retained)
        self.assertIn("catalog: Option<Arc<EditorAssetCatalogGeneration>>", workspace)
        self.assertIn("selected_details: Option<Arc<EditorAssetDetailsGeneration>>", workspace)

    def test_preview_io_is_outside_live_state_lock_and_publication_is_generation_safe(self) -> None:
        preview = (MANAGER / "preview_refresh/request_preview_refresh.rs").read_text(
            encoding="utf-8"
        )
        self.assertLess(
            preview.index("generate_preview_artifact("),
            preview.index("let _publish_guard = publish_gate"),
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

    def test_project_resource_delta_is_typed_but_does_not_claim_incomplete_catalog_fast_path(self) -> None:
        delta = SOURCE_GENERATION.read_text(encoding="utf-8")
        self.assertIn("struct EditorAssetProjectSourceGeneration", delta)
        self.assertIn("struct EditorAssetProjectSourceDelta", delta)
        self.assertIn("added", delta)
        self.assertIn("modified", delta)
        self.assertIn("removed", delta)
        self.assertIn("renamed", delta)
        self.assertIn(
            "source_generation_classifies_added_modified_removed_and_renamed",
            delta,
        )
        sync = (MANAGER / "project_sync/sync_from_project.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("EditorAssetProjectSourceGeneration::capture(&project)", sync)
        self.assertIn("resource_delta.is_unchanged()", sync)
        self.assertIn("source_sync_epoch", sync)
        self.assertIn("source_sync_gate", sync)
        self.assertNotIn("if resource_delta.is_unchanged()", sync)
        self.assertNotIn("unchanged_project_generation_reuses_catalog_arc_without_rebuild", sync)

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
