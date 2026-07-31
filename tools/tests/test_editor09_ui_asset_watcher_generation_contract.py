from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SESSIONS = REPO_ROOT / "zircon_editor/src/ui/host/asset_editor_sessions"
DEPENDENCY_INDEX = SESSIONS / "dependency_index"
IMPORTS = SESSIONS / "imports"
REFRESH = SESSIONS / "refresh"
PIPELINE = REFRESH / "pipeline"


class Editor09UiAssetWatcherGenerationContractTests(unittest.TestCase):
    def test_reverse_dependency_generation_is_the_only_open_session_projection(self) -> None:
        root = (SESSIONS / "mod.rs").read_text(encoding="utf-8")
        generation = (DEPENDENCY_INDEX / "generation.rs").read_text(encoding="utf-8")
        impact = (DEPENDENCY_INDEX / "impact.rs").read_text(encoding="utf-8")
        tests = (DEPENDENCY_INDEX / "tests.rs").read_text(encoding="utf-8")

        self.assertIn("mod dependency_index;", root)
        self.assertIn("struct UiAssetDependencyGeneration", generation)
        self.assertIn("direct_by_asset_id", generation)
        self.assertIn("importers_by_asset_id", generation)
        self.assertIn("dependencies_by_instance", generation)
        self.assertIn("fn impact", generation)
        self.assertIn("struct UiAssetDependencyImpact", impact)
        self.assertIn("transitive_dependency_targets_only_registered_consumers", tests)
        self.assertIn("replacing_dependencies_removes_old_reverse_edges", tests)

    def test_import_parse_cache_is_shared_once_per_refresh_generation(self) -> None:
        root = (IMPORTS / "mod.rs").read_text(encoding="utf-8")
        generation = (IMPORTS / "generation.rs").read_text(encoding="utf-8")
        traversal = (IMPORTS / "traversal.rs").read_text(encoding="utf-8")
        tests = (IMPORTS / "tests.rs").read_text(encoding="utf-8")

        self.assertFalse((SESSIONS / "imports.rs").exists())
        self.assertIn("mod generation;", root)
        self.assertIn("parsed_by_physical_path", generation)
        self.assertIn("UiAssetImportResolution", traversal)
        self.assertIn("dependencies", traversal)
        self.assertIn("physical_document_is_loaded_once_across_generation_traversals", tests)

    def test_watcher_uses_background_generation_job_and_not_sync_refresh(self) -> None:
        watcher_host = (SESSIONS / "watcher/host.rs").read_text(encoding="utf-8")
        pipeline = (PIPELINE / "mod.rs").read_text(encoding="utf-8")
        service = (PIPELINE / "service.rs").read_text(encoding="utf-8")
        job = (PIPELINE / "job.rs").read_text(encoding="utf-8")
        commit = (PIPELINE / "commit.rs").read_text(encoding="utf-8")
        queue = (PIPELINE / "queue.rs").read_text(encoding="utf-8")

        self.assertFalse((SESSIONS / "refresh.rs").exists())
        self.assertNotIn("refresh_ui_asset_workspace_for_changes", watcher_host)
        self.assertIn("mod service;", pipeline)
        self.assertIn("UiAssetWorkspaceRefreshPipeline", service)
        self.assertNotIn("impl UiAssetWorkspaceRefreshPipeline", pipeline)
        self.assertIn("JobCategory::Index", service)
        self.assertIn("JobPriority::Background", service)
        self.assertGreaterEqual(job.count("context.check_cancelled()?;"), 2)
        self.assertIn("generation", commit)
        self.assertIn("source_fingerprint", commit)
        self.assertIn("superseded", queue)
        self.assertIn("defer_retry", queue)
        self.assertIn("transition_project", watcher_host)

    def test_refresh_lookup_is_delta_shaped_and_never_scans_all_open_sessions(self) -> None:
        apply = (REFRESH / "apply.rs").read_text(encoding="utf-8")
        plan = (PIPELINE / "plan.rs").read_text(encoding="utf-8")
        tests = (PIPELINE / "tests.rs").read_text(encoding="utf-8")

        self.assertIn("dependency_generation.impact", apply)
        self.assertNotIn("sessions.iter()", apply)
        self.assertIn("direct_instances", plan)
        self.assertIn("import_instances", plan)
        self.assertIn("newer_change_supersedes_active_generation", tests)
        self.assertIn("stable_generation_does_not_schedule_work", tests)

    def test_import_commit_epoch_and_transient_retry_are_explicitly_bounded(self) -> None:
        collect = (IMPORTS / "collect.rs").read_text(encoding="utf-8")
        hydration = (SESSIONS / "hydration.rs").read_text(encoding="utf-8")
        commit = (PIPELINE / "commit.rs").read_text(encoding="utf-8")
        queue = (PIPELINE / "queue.rs").read_text(encoding="utf-8")

        self.assertLess(
            collect.index("record_dependency(reference)"),
            collect.index("let source_path = resolve(reference)?"),
        )
        self.assertLess(
            hydration.index("let mut dependency_generation"),
            hydration.index("let mut sessions"),
        )
        self.assertIn("MAX_TRANSIENT_RETRY_ATTEMPTS: u8 = 6", queue)
        self.assertIn("MAX_TRANSIENT_RETRY_DELAY", queue)
        self.assertIn("entry.stale_imports.remove", commit)
        self.assertIn("retry_asset_ids", commit)
        self.assertNotIn("map_err(|error| EditorError::UiAsset(error.to_string()))?", commit)

        watcher = (SESSIONS / "watcher/host.rs").read_text(encoding="utf-8")
        diagnostics = (SESSIONS / "watcher/diagnostics.rs").read_text(encoding="utf-8")
        self.assertIn("collect_ui_asset_imports_lossy", hydration)
        self.assertLess(
            watcher.index("for asset_id in &commit.retry_asset_ids"),
            watcher.index("sync_ui_asset_refresh_instances"),
        )
        self.assertIn("refresh_deferred_retry_count", diagnostics)
        self.assertIn("refresh_exhausted_retry_count", diagnostics)


if __name__ == "__main__":
    unittest.main()
