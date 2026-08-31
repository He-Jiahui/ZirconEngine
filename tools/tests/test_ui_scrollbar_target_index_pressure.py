import unittest

from tools.ui_scrollbar_target_index_pressure import run


class UiScrollbarTargetIndexPressureTests(unittest.TestCase):
    def test_default_model_includes_conservative_cold_build_cost(self) -> None:
        result = run()

        self.assertEqual(result["legacy_full_tree_node_visits"], 20_000_000)
        self.assertEqual(result["indexed_cold_build_node_visits"], 10_000)
        self.assertEqual(result["indexed_exact_candidate_checks"], 2_000)
        self.assertEqual(result["indexed_combined_work_units"], 12_000)
        self.assertGreater(result["work_reduction_ratio"], 1_600.0)

    def test_dirty_patches_and_duplicate_candidates_are_explicit(self) -> None:
        result = run(
            node_count=1_000,
            pointer_move_count=100,
            average_bucket_candidate_count=3,
            dirty_node_patch_count=25,
        )

        self.assertEqual(result["indexed_combined_work_units"], 1_325)
        self.assertEqual(result["indexed_dirty_patch_node_visits"], 25)

    def test_invalid_inputs_fail_closed(self) -> None:
        with self.assertRaises(ValueError):
            run(node_count=0)
        with self.assertRaises(ValueError):
            run(pointer_move_count=0)
        with self.assertRaises(ValueError):
            run(average_bucket_candidate_count=101, node_count=100)
        with self.assertRaises(ValueError):
            run(dirty_node_patch_count=-1)


if __name__ == "__main__":
    unittest.main()
