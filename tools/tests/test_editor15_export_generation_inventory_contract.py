from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    path = ROOT / relative
    return path.read_text(encoding="utf-8") if path.exists() else ""


class Editor15ExportGenerationInventoryContractTests(unittest.TestCase):
    def test_inventory_is_the_only_recursive_digest_owner(self) -> None:
        inventory = source(
            "zircon_editor/src/core/export/inventory.rs"
        )
        executor = source("zircon_editor/src/core/export/stages/executor.rs")
        core_export = source("zircon_editor/src/core/export/mod.rs")

        for required in [
            "ExportGenerationInventory",
            "digests_by_canonical_path",
            "digest_path",
            "invalidate_subtree",
            "artifact_with_current_digest",
            "artifact_matches_disk",
        ]:
            self.assertIn(required, inventory)
        self.assertIn("mod inventory;", core_export)
        self.assertIn("inventory: ExportGenerationInventory", executor)
        self.assertNotIn("fn digest_path(", executor)

    def test_overlap_and_rebuild_invalidation_are_locked_by_behavior_tests(self) -> None:
        inventory = source(
            "zircon_editor/src/core/export/inventory.rs"
        )
        executor = source("zircon_editor/src/core/export/stages/executor.rs")

        for test_name in [
            "overlapping_root_and_child_digests_read_each_file_once",
            "invalidating_a_rebuilt_subtree_refreshes_its_digest",
        ]:
            self.assertIn(f"fn {test_name}", inventory)
        self.assertIn(
            "invalidate_subtree(&self.compile_host.staged_engine_root())", executor
        )

    def test_wizard_streams_full_logs_but_retains_only_bounded_tails(self) -> None:
        wizard = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/mod.rs"
        )
        plan = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/plan.rs"
        )
        execution = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs"
        )
        capture = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/output_capture.rs"
        )
        job_state = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/job.rs"
        )
        view_model = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs"
        )
        tail = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/output_tail.rs"
        )
        job = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/job.rs"
        )

        self.assertIn("mod output_tail;", wizard)
        self.assertIn("mod output_capture;", execution)
        for key in ["stdout_log", "stderr_log", "output_log_manifest"]:
            self.assertIn(f'"{key}"', plan)
        for required in [
            "ExportWizardOutputCapture",
            "blake3::Hasher",
            "byte_count",
            "digest",
            "sync_all",
            "full_output_is_written_while_only_tail_is_retained",
        ]:
            self.assertIn(required, capture)
        for required in [
            "MAX_OUTPUT_TAIL_LINES",
            "push_bounded_output_line",
            "tail_never_exceeds_limit",
            "truncation_marker_is_retained",
            "terminal_vec_results_are_bounded_at_the_output_boundary",
            "VecDeque",
            "pop_front()",
            "push_front",
        ]:
            self.assertIn(required, tail)
        self.assertNotIn("lines.remove(", tail)
        self.assertNotIn("lines.drain(", tail)
        self.assertNotIn("lines.insert(", tail)
        self.assertIn("tail_lines: VecDeque<String>", capture)
        self.assertIn("self.tail_lines.into_iter().collect()", capture)
        self.assertIn("stdout_lines: VecDeque<String>", job_state)
        self.assertIn("stderr_lines: VecDeque<String>", job_state)
        self.assertIn("output.stdout_lines.iter().cloned().collect()", view_model)
        self.assertIn("output.stderr_lines.iter().cloned().collect()", view_model)
        self.assertIn("ExportWizardOutputCapture::open", execution)
        self.assertNotIn("let mut stdout_lines = Vec::new();", execution)
        self.assertIn("push_bounded_output_line", job)

    def test_incremental_output_line_scan_resumes_at_the_pending_suffix(self) -> None:
        capture = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/output_capture.rs"
        )

        self.assertIn("scan_from", capture)
        self.assertIn("self.pending.len().saturating_sub(1)", capture)
        self.assertNotIn("for index in 0..self.pending.len()", capture)

    def test_wizard_stage_output_events_are_delta_based_and_budgeted(self) -> None:
        run = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs"
        )
        job = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/job.rs"
        )
        controller = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs"
        )
        controller_job = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/job.rs"
        )
        view_model = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs"
        )
        streaming_tests = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/streaming_output_tests.rs"
        )

        for required in [
            "ExportWizardStageOutputDelta",
            "output_delta: Option<ExportWizardStageOutputDelta>",
            "emit_stage_output_event",
        ]:
            self.assertIn(required, run)
        self.assertIn("event_header", job)
        self.assertIn("apply_stage_output", job)
        self.assertIn("sync_channel", controller)
        self.assertIn("EXPORT_WIZARD_EVENT_CHANNEL_CAPACITY", controller)
        for required in [
            "SyncSender",
            "TrySendError",
            "MAX_BUFFERED_STAGE_OUTPUT_EVENTS_PER_STAGE",
            "coalesced_output_events",
            "output_backpressure_preserves_terminal_event_and_reports_coalesced_count",
        ]:
            self.assertIn(required, controller_job)
        for required in [
            "MAX_EVENTS_PER_DRAIN",
            "MAX_EVENT_DRAIN_TIME",
            "Instant::now",
            "apply_stage_output",
        ]:
            self.assertIn(required, view_model)
        for test_name in [
            "stage_output_event_carries_delta_without_accumulated_snapshot",
            "view_model_drain_is_budgeted",
        ]:
            self.assertIn(f"fn {test_name}", streaming_tests)

    def test_system_build_runner_streams_logs_and_retains_only_byte_tails(self) -> None:
        compile_host = source(
            "zircon_editor/src/core/export/stages/compile_host.rs"
        )
        executor = source(
            "zircon_editor/src/core/export/stages/executor.rs"
        )

        for required in [
            "MAX_COMMAND_OUTPUT_TAIL_BYTES",
            ".spawn()",
            "capture_output_stream",
            "write_output_manifest",
            "byte_count",
            "digest",
            "sync_all",
            "system_runner_streams_full_logs_and_bounds_memory_tails",
        ]:
            self.assertIn(required, compile_host)
        self.assertNotIn(".output()", compile_host)
        self.assertIn("compile_host_output_artifacts", executor)

    def test_inventory_persists_file_identity_and_generation_tool_probes(self) -> None:
        inventory = source(
            "zircon_editor/src/core/export/inventory.rs"
        )
        executor = source(
            "zircon_editor/src/core/export/stages/executor.rs"
        )

        for required in [
            "PersistentInventoryCache",
            "FileMetadataIdentity",
            "change_marker",
            "file_identity",
            "with_persistent_cache",
            "persist_cache",
            "tool_identity",
            "persistent_cache_reuses_unchanged_file_without_reading_content",
            "same_size_rewrite_invalidates_persistent_digest",
            "tool_identity_is_probed_once_per_generation",
        ]:
            self.assertIn(required, inventory)
        self.assertIn("FILE_BASIC_INFO", inventory)
        self.assertIn("FILE_ID_INFO", inventory)
        self.assertIn("parameter_digest: Option<ExportDigest>", executor)
        self.assertIn("inventory.tool_identity", executor)
        self.assertNotIn("fn tool_identity(", executor)

    def test_native_staging_uses_shared_inventory_and_persistent_deltas(self) -> None:
        core_export = source("zircon_editor/src/core/export/mod.rs")
        stages = source("zircon_editor/src/core/export/stages/mod.rs")
        staging = source(
            "zircon_editor/src/ui/host/native_dynamic_export_preparation/staging.rs"
        )
        preparation = source(
            "zircon_editor/src/ui/host/native_dynamic_export_preparation/prepare.rs"
        )
        preparation_module = source(
            "zircon_editor/src/ui/host/native_dynamic_export_preparation/mod.rs"
        )
        manager = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs"
        )

        self.assertIn("mod inventory;", core_export)
        self.assertIn("ExportGenerationInventory", core_export)
        self.assertNotIn("mod inventory;", stages)
        self.assertFalse(
            source("zircon_editor/src/core/export/stages/inventory.rs")
        )
        for required in [
            "NativeStagingManifest",
            "NativeStagingStats",
            "sync_native_package",
            "copied_files",
            "copied_bytes",
            "removed_files",
            "unchanged_warm_staging_copies_zero_files_and_bytes",
            "changed_deleted_and_renamed_sources_update_the_staging_tree",
        ]:
            self.assertIn(required, staging)
        self.assertIn(".zircon/cache/export/native-dynamic", preparation)
        self.assertIn("ExportGenerationInventory", preparation)
        self.assertNotIn("cleanup_native_dynamic_roots", preparation)
        self.assertNotIn("mod cleanup;", preparation_module)
        self.assertFalse(
            source("zircon_editor/src/ui/host/native_dynamic_export_preparation/cleanup.rs")
        )
        self.assertNotIn("cleanup_native_dynamic_preparation", manager)

    def test_build_export_pane_projection_is_source_and_overlay_cached(self) -> None:
        projection = source(
            "zircon_editor/src/ui/retained_host/app/build_export_projection.rs"
        )
        cache = source(
            "zircon_editor/src/ui/retained_host/app/build_export_projection/cache.rs"
        )
        targets = source(
            "zircon_editor/src/ui/retained_host/app/build_export_projection/targets.rs"
        )
        sessions = source(
            "zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state.rs"
        )
        actions = source(
            "zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions.rs"
        )
        job_polling = source(
            "zircon_editor/src/ui/retained_host/app/build_export_actions/host_actions/jobs/polling.rs"
        )

        for required in [
            "BuildExportProjectionCache",
            "cached_base",
            "cached_rendered",
            "FileMetadataIdentity",
            "unchanged_source_identity_reuses_cached_projection_without_read_dir",
            "changed_preset_invalidates_cached_projection_once",
        ]:
            self.assertIn(required, cache)
        self.assertNotIn("std::fs::read_dir", cache)
        self.assertIn("rebuild_export_targets", targets)
        self.assertIn("std::fs::read_dir", targets)
        for required in [
            "projection_cache",
            "projection_overlay_generation",
            "invalidate_projection_source",
            "invalidate_projection_overlay",
        ]:
            self.assertIn(required, sessions)
        self.assertIn("cached_rendered", projection)
        self.assertIn("projection_overlay_generation", projection)
        self.assertIn("invalidate_projection_source", actions)
        self.assertIn("invalidate_projection_overlay", job_polling)

    def test_report_projection_parses_structured_stdout_once(self) -> None:
        projection = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection.rs"
        )
        tests = source(
            "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_report_body_tests.rs"
        )

        self.assertIn("parsed_report_from_stdout", projection)
        self.assertEqual(projection.count("serde_json::from_str"), 1)
        self.assertIn(
            "panel_report_json_is_parsed_once_for_all_summaries",
            tests,
        )


if __name__ == "__main__":
    unittest.main()
