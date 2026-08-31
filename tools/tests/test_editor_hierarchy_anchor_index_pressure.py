import json
import tempfile
import unittest
from pathlib import Path

from tools.editor_hierarchy_anchor_index_pressure import run, write_result


class EditorHierarchyAnchorIndexPressureTests(unittest.TestCase):
    def test_default_model_replaces_full_paint_scans_with_bounded_candidates(self) -> None:
        result = run()

        self.assertEqual(result["legacy_scanned_nodes_per_paint"], 10_000)
        self.assertEqual(result["legacy_template_node_visits"], 20_000_000)
        self.assertEqual(result["target_generation_node_visits"], 10_000)
        self.assertEqual(result["target_candidate_index_visits"], 4_000)
        self.assertEqual(result["target_overlay_trie_depth"], 14)
        self.assertEqual(result["target_live_row_probe_visits"], 60_000)
        self.assertEqual(result["target_combined_work_units"], 76_000)
        self.assertGreater(result["work_reduction_ratio"], 260.0)

    def test_candidate_and_overlay_costs_are_explicit(self) -> None:
        result = run(
            template_node_count=32,
            pane_paint_count=7,
            metadata_generation_count=2,
            anchor_candidate_count=3,
        )

        self.assertEqual(result["target_generation_node_visits"], 64)
        self.assertEqual(result["target_metadata_queries"], 7)
        self.assertEqual(result["target_candidate_index_visits"], 21)
        self.assertEqual(result["target_overlay_trie_node_visits_per_candidate"], 6)
        self.assertEqual(result["target_live_row_probe_visits"], 126)
        self.assertEqual(result["target_combined_work_units"], 218)

    def test_invalid_inputs_and_c_drive_output_fail_closed(self) -> None:
        with self.assertRaises(ValueError):
            run(template_node_count=0)
        with self.assertRaises(ValueError):
            run(pane_paint_count=0)
        with self.assertRaises(ValueError):
            run(metadata_generation_count=0)
        with self.assertRaises(ValueError):
            run(anchor_candidate_count=0)
        with self.assertRaises(ValueError):
            write_result(Path("C:/hierarchy-anchor-index-pressure.json"), run())

    def test_output_is_stable_json_on_external_storage(self) -> None:
        result = run(template_node_count=8, pane_paint_count=3)
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "hierarchy-anchor-index-pressure.json"
            write_result(output, result)
            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)


if __name__ == "__main__":
    unittest.main()
