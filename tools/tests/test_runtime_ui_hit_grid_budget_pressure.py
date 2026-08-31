import unittest

from tools.runtime_ui_hit_grid_budget_pressure import MAX_CELL_COUNT, run


class RuntimeUiHitGridBudgetPressureTests(unittest.TestCase):
    def test_extreme_authored_extent_is_bounded(self):
        result = run(10_000, 1_000_000.0, 1_000_000.0)
        self.assertEqual(result["unbounded_cell_count"], 244_140_625)
        self.assertEqual(result["bounded_cell_count"], 1)
        self.assertTrue(result["wide_entry_fallback"])
        self.assertLessEqual(result["bounded_cell_count"], MAX_CELL_COUNT)

    def test_ordinary_extent_keeps_spatial_partition(self):
        result = run(10, 256.0, 128.0)
        self.assertEqual(result["bounded_columns"], 4)
        self.assertEqual(result["bounded_rows"], 2)
        self.assertFalse(result["wide_entry_fallback"])

    def test_model_rejects_invalid_inputs(self):
        with self.assertRaises(ValueError):
            run(0, 100.0, 100.0)
        with self.assertRaises(ValueError):
            run(1, 0.0, 100.0)
        with self.assertRaises(ValueError):
            run(1, 100.0, 100.0, 0.0)


if __name__ == "__main__":
    unittest.main()
