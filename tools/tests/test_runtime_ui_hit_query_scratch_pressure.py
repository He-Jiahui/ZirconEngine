import unittest

from tools.runtime_ui_hit_query_scratch_pressure import run


class RuntimeUiHitQueryScratchPressureTests(unittest.TestCase):
    def test_retained_index_removes_per_query_scratch_storage(self):
        result = run(
            entry_count=65_536,
            average_candidate_count=32,
            pointer_query_count=1_000_000,
        )

        self.assertEqual(result["old_scratch_initialization_count"], 1_000_000)
        self.assertEqual(result["new_scratch_initialization_count"], 1)
        self.assertEqual(result["old_storage_allocation_count"], 2_000_000)
        self.assertEqual(result["new_storage_allocation_count"], 2)
        self.assertEqual(result["avoided_storage_allocations"], 1_999_998)
        self.assertEqual(result["storage_slot_reduction_ratio"], 1_000_000.0)

    def test_model_rejects_invalid_query_shapes(self):
        with self.assertRaises(ValueError):
            run(entry_count=0, average_candidate_count=1, pointer_query_count=1)
        with self.assertRaises(ValueError):
            run(entry_count=10, average_candidate_count=11, pointer_query_count=1)
        with self.assertRaises(ValueError):
            run(entry_count=10, average_candidate_count=1, pointer_query_count=0)


if __name__ == "__main__":
    unittest.main()
