import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class EditorAssetBrowserProfileScaleContract(unittest.TestCase):
    def test_retained_asset_nodes_are_bounded_and_scroll_rebinds_slots(self) -> None:
        browser = (
            REPO_ROOT / "zircon_editor/src/ui/layouts/views/asset_browser.rs"
        ).read_text(encoding="utf-8")
        table = (
            REPO_ROOT
            / "zircon_editor/src/ui/layouts/views/asset_browser/table_nodes.rs"
        ).read_text(encoding="utf-8")
        thumbnails = (
            REPO_ROOT
            / "zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs"
        ).read_text(encoding="utf-8")
        projector = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("asset_browser_materialized_item_budget", browser)
        self.assertIn("materialized_item_count", table)
        self.assertNotIn("snapshot.visible_assets.len()", table)
        self.assertIn(".take(materialized_item_count)", thumbnails)
        self.assertIn("trim_asset_browser_thumbnail_slots", browser)
        self.assertIn("grid.frame.height", thumbnails)
        self.assertIn("browser_slot_binding", projector)
        self.assertIn("node.options = item.cells.clone()", projector)
        self.assertNotIn("model_rc(item.cells", projector)
        self.assertNotIn("visible_assets", projector)
        self.assertNotIn("for row in 0..nodes.row_count()", projector)

    def test_scroll_path_publishes_dispatch_and_logical_catalog_cardinality(self) -> None:
        dispatch = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch/pane/asset/content.rs"
        ).read_text(encoding="utf-8")
        motion = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/motion.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("AssetBrowserScrollDispatchCount", dispatch)
        self.assertIn("AssetBrowserLogicalItemCount", motion)
        self.assertIn("target.snapshot.visible_assets.len()", motion)

    def test_paint_path_reports_cached_materialization_and_visible_range_counts(self) -> None:
        metadata = (
            REPO_ROOT
            / "zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs"
        ).read_text(encoding="utf-8")
        projector = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("browser_materialized_item_count", metadata)
        self.assertIn("browser_materialized_node_count", metadata)
        self.assertIn("visible_browser_item_count", metadata)
        self.assertIn("AssetBrowserMaterializedItemCount", projector)
        self.assertIn("AssetBrowserMaterializedNodeCount", projector)
        self.assertIn("AssetBrowserVisibleItemCount", projector)
        self.assertIn("AssetBrowserVisibleNodeCount", projector)
        self.assertIn("record_current_ui_perf_counter_batch", projector)
        self.assertNotIn("record_current_ui_perf_counter(", projector)
        self.assertNotIn("self.metadata.scroll_groups.iter().count()", projector)

    def test_capture_gate_binds_counters_to_the_source_manifest_scale(self) -> None:
        evidence = (REPO_ROOT / "tools/ui-profile-counter-evidence.ps1").read_text(
            encoding="utf-8"
        )
        capture = (REPO_ROOT / "tools/ui-profile-capture.ps1").read_text(
            encoding="utf-8"
        )

        self.assertIn("function Test-ZirconAssetBrowserScrollCounterGate", evidence)
        self.assertIn("source_manifest.json", evidence)
        self.assertIn('fixtureKind -ne "asset_catalog_json"', evidence)
        self.assertIn("asset_catalog_item_count", evidence)
        self.assertIn("ui.idle_hover.asset_browser_scroll_dispatch_count", evidence)
        self.assertIn("ui.idle_hover.asset_browser_logical_item_count", evidence)
        self.assertIn("ui.idle_hover.asset_browser_materialized_item_count", evidence)
        self.assertIn("ui.idle_hover.asset_browser_materialized_node_count", evidence)
        self.assertIn("ui.idle_hover.asset_browser_visible_item_count", evidence)
        self.assertIn("ui.idle_hover.asset_browser_visible_node_count", evidence)
        self.assertIn("ui.idle_hover.asset_browser_projection_build_count", evidence)
        self.assertIn("function Test-AssetBrowserScrollCounterGate", capture)
        self.assertIn("$assetBrowserScrollOk = Test-AssetBrowserScrollCounterGate", capture)


if __name__ == "__main__":
    unittest.main()
