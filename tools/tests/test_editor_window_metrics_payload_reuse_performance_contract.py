from pathlib import Path
import unittest

from tools.editor_window_metrics_payload_pressure import run


ROOT = Path(__file__).resolve().parents[2]
APP = ROOT / "zircon_editor/src/ui/retained_host/app"
LIFECYCLE = APP / "host_lifecycle"


class EditorWindowMetricsPayloadReusePerformanceContractTests(unittest.TestCase):
    def test_committed_shell_retains_the_payload_bundle_by_shared_identity(self) -> None:
        source = (APP / "committed_shell_state.rs").read_text(encoding="utf-8")

        self.assertIn("pub(in crate::ui::retained_host::app) struct HostLifecyclePanePayloads", source)
        self.assertIn(
            "pane_payloads: Option<Arc<HostLifecyclePanePayloads>>",
            source,
        )

    def test_window_metrics_snapshot_moves_the_committed_payload_bundle(self) -> None:
        snapshot = (LIFECYCLE / "recompute/shell/snapshot.rs").read_text(
            encoding="utf-8"
        )
        builder = (LIFECYCLE / "recompute/shell/builder.rs").read_text(
            encoding="utf-8"
        )

        self.assertRegex(
            snapshot,
            r"retained_pane_payloads:\s+Option<Arc<HostLifecyclePanePayloads>>",
        )
        self.assertIn("retained_pane_payloads: committed.pane_payloads", builder)
        self.assertIn("retained_pane_payloads: None", builder)

    def test_metrics_recompute_reuses_payloads_and_full_recompute_replaces_them(self) -> None:
        source = (LIFECYCLE / "recompute.rs").read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]

        self.assertIn("shell.retained_pane_payloads.take()", production)
        self.assertIn("Arc::new(self.collect_host_lifecycle_pane_payloads", production)
        self.assertIn('"ui.window_metrics.pane_payload_cache_hit_count"', production)
        self.assertIn('"ui.window_metrics.pane_payload_cache_miss_count"', production)
        self.assertIn("pane_payloads: Some(pane_payloads)", production)

    def test_content_patches_invalidate_the_retained_payload_bundle(self) -> None:
        source = (LIFECYCLE / "recompute/presentation.rs").read_text(
            encoding="utf-8"
        )
        shell_content = source.split(
            "pub(super) fn apply_committed_shell_content_presentation", 1
        )[1].split("pub(super) fn apply_shell_content_presentation", 1)[0]
        scoped_view = source.split(
            "pub(super) fn apply_scoped_ui_asset_presentation", 1
        )[1].split("pub(super) fn apply_recompute_presentation", 1)[0]

        self.assertIn("committed.pane_payloads = None", shell_content)
        self.assertIn("invalidate_committed_pane_payloads", scoped_view)

    def test_pressure_model_eliminates_collection_on_metrics_only_reflows(self) -> None:
        result = run(
            payload_source_count=7,
            metrics_reflow_count=1_000,
            content_patch_count=10,
        )

        self.assertEqual(result["old_payload_source_collection_count"], 7_000)
        self.assertEqual(result["new_metrics_payload_source_collection_count"], 0)
        self.assertEqual(result["new_content_refresh_source_collection_count"], 70)
        self.assertEqual(result["eliminated_payload_source_collection_count"], 6_930)
        self.assertEqual(result["payload_source_collection_reduction_ratio"], 100.0)

    def test_window_resize_profile_gate_exports_payload_cache_evidence(self) -> None:
        source = (ROOT / "tools/ui-profile-counter-evidence.ps1").read_text(
            encoding="utf-8"
        )

        self.assertIn("ui.window_metrics.pane_payload_cache_hit_count", source)
        self.assertIn("ui.window_metrics.pane_payload_cache_miss_count", source)
        self.assertIn("$panePayloadCacheMissCount -eq 0", source)


if __name__ == "__main__":
    unittest.main()
