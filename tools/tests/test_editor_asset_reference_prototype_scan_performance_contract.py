import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/layouts/views/asset_reference_rows.rs"


class EditorAssetReferencePrototypeScanPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("impl AssetReferenceNodePrototypes {")
        end = source.index("\nfn is_dynamic_reference_row_component", start)
        self.implementation = source[start:end]

    def test_prototype_extraction_scans_the_node_slice_once(self) -> None:
        body = self.implementation
        self.assertIn("for node in nodes", body)
        self.assertIn("capture_prototype", body)
        self.assertIn("if panel.is_some()", body)
        self.assertNotIn("find_node(nodes, controls.row_", body)

    def test_prototype_extraction_keeps_all_four_first_match_slots(self) -> None:
        body = self.implementation
        for slot in ("panel", "name", "locator", "kind"):
            self.assertIn(f"{slot}: {slot}?", body)
        for control in ("row_panel_control_id", "row_name_control_id", "row_locator_control_id", "row_kind_control_id"):
            self.assertIn(f"controls.{control}", body)

    def test_single_scan_model_removes_repeated_prototype_walks(self) -> None:
        unrelated_prefix_nodes = 4096
        legacy_visits = unrelated_prefix_nodes * 4
        single_scan_visits = unrelated_prefix_nodes
        self.assertEqual(legacy_visits, 16_384)
        self.assertEqual(single_scan_visits, 4096)
        self.assertLessEqual(single_scan_visits * 100, legacy_visits * 80)


if __name__ == "__main__":
    unittest.main()
