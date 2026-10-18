import unittest
from pathlib import Path

from tools.analysis.performance.editor.editor_performance_timeline_dead_projection_pressure import (
    pressure_report,
    run,
    validate_source_contract,
)


ROOT = Path(__file__).resolve().parents[2]


class EditorPerformanceTimelineDeadProjectionPressureTests(unittest.TestCase):
    def test_nodes_only_contract_eliminates_unused_row_projection(self) -> None:
        result = pressure_report(refresh_count=200, logical_rows=256)

        self.assertEqual(result["delta"]["avoided_dead_row_mapping_count"], 153_600)
        self.assertEqual(result["delta"]["avoided_dead_control_mapping_count"], 800)
        self.assertEqual(result["delta"]["avoided_dead_summary_clone_count"], 600)
        self.assertEqual(
            result["current_nodes_only_projection"]["frame_row_mapping_count"], 0
        )
        self.assertTrue(
            result["current_nodes_only_projection"][
                "visible_template_node_projection_preserved"
            ]
        )
        self.assertFalse(result["interpretation"]["timing_claim"])

    def test_current_source_guards_and_binding(self) -> None:
        validate_source_contract(ROOT)
        self.assertEqual(len(run(ROOT)["source_binding"]["files"]), 5)

    def test_negative_counts_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            pressure_report(logical_rows=-1)


if __name__ == "__main__":
    unittest.main()
