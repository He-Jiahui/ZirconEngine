from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WATCHER_ROOT = (
    REPO_ROOT / "zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs"
)
WATCHER_DIR = WATCHER_ROOT.with_suffix("")
MANAGER_API = REPO_ROOT / "zircon_editor/src/ui/host/editor_manager_asset_workspace.rs"
CRATE_ROOT = REPO_ROOT / "zircon_editor/src/lib.rs"
RECONCILE = (
    REPO_ROOT
    / "zircon_editor/src/ui/host/asset_editor_sessions/refresh/reconcile.rs"
)
SESSION_IMPORT_ACCESS = (
    REPO_ROOT / "zircon_editor/src/ui/asset_editor/session/import_reference_access.rs"
)
SESSION_ROOT = REPO_ROOT / "zircon_editor/src/ui/asset_editor/session/mod.rs"


class Editor09UiAssetWatcherBoundedRefreshContractTests(unittest.TestCase):
    def test_watcher_is_folder_backed_and_has_no_unbounded_drain_all_transport(self) -> None:
        root_source = WATCHER_ROOT.read_text(encoding="utf-8")
        for module in (
            "budget",
            "diagnostics",
            "host",
            "ingress",
            "path_identity",
            "service",
        ):
            self.assertIn(f"mod {module};", root_source)
            self.assertTrue((WATCHER_DIR / f"{module}.rs").is_file())
        watcher_source = root_source + "".join(
            path.read_text(encoding="utf-8")
            for path in sorted(WATCHER_DIR.glob("*.rs"))
        )
        self.assertNotIn("unbounded", watcher_source)
        self.assertNotIn("while let Ok(path) = self.receiver.try_recv()", watcher_source)

    def test_poll_surface_is_a_typed_observable_report_without_vec_compatibility(self) -> None:
        manager_source = MANAGER_API.read_text(encoding="utf-8")
        crate_source = CRATE_ROOT.read_text(encoding="utf-8")
        signature = (
            "pub fn poll_ui_asset_workspace_watcher(\n"
            "        &self,\n"
            "    ) -> Result<UiAssetWorkspaceWatchPollReport, EditorError>"
        )
        self.assertIn(signature, manager_source)
        self.assertNotIn(
            "poll_ui_asset_workspace_watcher(&self) -> Result<Vec<String>, EditorError>",
            manager_source,
        )
        self.assertIn("UiAssetWorkspaceWatchPollReport", crate_source)

    def test_real_contract_covers_scale_budget_overflow_and_age(self) -> None:
        budget_source = (WATCHER_DIR / "budget.rs").read_text(encoding="utf-8")
        ingress_source = (WATCHER_DIR / "ingress.rs").read_text(encoding="utf-8")
        service_source = (WATCHER_DIR / "service.rs").read_text(encoding="utf-8")
        tests_source = (WATCHER_DIR / "tests.rs").read_text(encoding="utf-8")
        reconcile_source = RECONCILE.read_text(encoding="utf-8")
        import_access_source = SESSION_IMPORT_ACCESS.read_text(encoding="utf-8")
        session_root_source = SESSION_ROOT.read_text(encoding="utf-8")
        combined = (
            budget_source
            + ingress_source
            + service_source
            + tests_source
            + reconcile_source
        )
        for token in (
            "10_000",
            "max_pending_paths",
            "max_paths_per_poll",
            "max_poll_time",
            "coalesced_path_count",
            "overflow_count",
            "oldest_pending_age",
            "budget_exhausted",
            "UiAssetWatchReconcileCursor",
            "reconcile_cursor_active",
            "restoring_a_polled_suffix_after_callback_refill_preserves_capacity",
            "second_overflow_is_visible_in_the_report_after_reconcile_work",
        ):
            self.assertIn(token, combined)
        self.assertIn("import_reference_count", import_access_source)
        self.assertIn("import_reference_at", import_access_source)
        self.assertIn("mod import_reference_access;", session_root_source)
        self.assertNotIn(".import_references()", reconcile_source)


if __name__ == "__main__":
    unittest.main()
