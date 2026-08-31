import unittest
from pathlib import Path

from tools.runtime_ui_layout_report_aggregation_pressure import run, validate_output_path


class RuntimeUiLayoutReportAggregationPressureTests(unittest.TestCase):
    def test_default_model_shifts_reason_entry_allocations_to_retained_capacity(self):
        result = run()

        self.assertEqual(
            result["historical_temporary_btree_map"]["reason_entry_allocations"],
            8_000,
        )
        self.assertEqual(
            result["retained_sorted_reason_vector"]["reason_entry_allocations"],
            8,
        )
        self.assertEqual(result["delta"]["avoided_reason_entry_allocations"], 7_992)
        self.assertTrue(result["delta"]["aggregation_operation_count_unchanged"])
        self.assertFalse(result["interpretation"]["timing_claim"])

    def test_default_model_bounds_layout_report_publication_to_one_leaf_segment(self):
        result = run()

        self.assertEqual(
            result["historical_flat_selection_vector"]["selection_clone_work"],
            10_000_000,
        )
        self.assertEqual(
            result["persistent_segmented_selection_sequence"]["selection_clone_work"],
            64_000,
        )
        self.assertEqual(
            result["persistent_segmented_selection_sequence"][
                "publication_handle_clone_count"
            ],
            1_000,
        )
        self.assertEqual(
            result["persistent_segmented_selection_sequence"][
                "directory_node_clone_work"
            ],
            2_000,
        )
        self.assertEqual(
            result["persistent_segmented_selection_sequence"][
                "residual_reason_entry_clone_work"
            ],
            8_000,
        )
        self.assertEqual(
            result["persistent_segmented_selection_sequence"][
                "residual_reason_vector_allocation_count"
            ],
            1_000,
        )
        self.assertEqual(
            result["delta"]["selection_clone_work_reduction_ratio"],
            156.25,
        )

    def test_model_rejects_invalid_cardinality(self):
        for kwargs in (
            {"selection_count": 0},
            {"non_native_selection_count": 11, "selection_count": 10},
            {"distinct_reason_count": 11, "non_native_selection_count": 10},
            {"recompute_count": 0},
            {"changed_selection_count": 0},
            {"changed_selection_count": 11, "selection_count": 10},
            {"selection_segment_size": 0},
            {"directory_fanout": 1},
            {"selection_payload_bytes": 0},
        ):
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(ValueError):
                    run(**kwargs)

    def test_output_artifacts_are_restricted_to_d_e_or_f(self):
        for path in (
            Path("D:/profiles/layout.json"),
            Path("E:/profiles/layout.json"),
            Path("F:/profiles/layout.json"),
        ):
            self.assertEqual(validate_output_path(path), path)
        for path in (Path("C:/profiles/layout.json"), Path("layout.json")):
            with self.assertRaises(ValueError):
                validate_output_path(path)


if __name__ == "__main__":
    unittest.main()
