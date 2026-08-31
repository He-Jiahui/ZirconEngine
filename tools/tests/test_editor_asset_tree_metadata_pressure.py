import json
import tempfile
import unittest
from pathlib import Path

from tools.editor_asset_tree_metadata_pressure import run, write_result


class EditorAssetTreeMetadataPressureTests(unittest.TestCase):
    def test_default_model_removes_repeated_asset_tree_scans(self) -> None:
        result = run()

        self.assertEqual(result["legacy_activity_count_node_visits"], 20_000_000)
        self.assertEqual(result["legacy_browser_count_node_visits"], 20_000_000)
        self.assertEqual(result["legacy_activity_hover_node_visits"], 10_000_000)
        self.assertEqual(result["legacy_combined_work_units"], 50_000_000)
        self.assertEqual(result["target_overlay_trie_depth"], 14)
        self.assertEqual(result["target_overlay_trie_node_visits_per_query"], 15)
        self.assertEqual(result["target_hover_live_row_probe_visits"], 15_000)
        self.assertEqual(result["target_combined_work_units"], 30_000)
        self.assertGreater(result["work_reduction_ratio"], 1_600.0)

    def test_generation_and_logarithmic_overlay_lookup_costs_are_explicit(self) -> None:
        result = run(
            template_node_count=32,
            pane_paint_count=7,
            activity_hover_paint_count=5,
            metadata_generation_count=2,
        )

        self.assertEqual(result["target_generation_node_visits"], 64)
        self.assertEqual(result["target_count_queries"], 14)
        self.assertEqual(result["target_hover_index_queries"], 5)
        self.assertEqual(result["target_overlay_trie_depth"], 5)
        self.assertEqual(result["target_overlay_trie_node_visits_per_query"], 6)
        self.assertEqual(result["target_hover_live_row_probe_visits"], 30)
        self.assertEqual(result["target_combined_work_units"], 113)

    def test_invalid_inputs_and_c_drive_output_fail_closed(self) -> None:
        with self.assertRaises(ValueError):
            run(template_node_count=0)
        with self.assertRaises(ValueError):
            run(pane_paint_count=0)
        with self.assertRaises(ValueError):
            run(activity_hover_paint_count=0)
        with self.assertRaises(ValueError):
            run(metadata_generation_count=0)
        with self.assertRaises(ValueError):
            write_result(Path("C:/asset-tree-metadata-pressure.json"), run())

    def test_output_is_stable_json_on_external_storage(self) -> None:
        result = run(template_node_count=8, pane_paint_count=3)
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "asset-tree-metadata-pressure.json"
            write_result(output, result)
            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)


if __name__ == "__main__":
    unittest.main()
