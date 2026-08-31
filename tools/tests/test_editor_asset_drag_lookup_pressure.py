import unittest

from tools.editor_asset_drag_lookup_pressure import pressure_report


class EditorAssetDragLookupPressureTests(unittest.TestCase):
    def test_large_visible_generation_has_constant_indexed_lookup_work(self):
        report = pressure_report(100_000, 1_000)

        self.assertEqual(report["legacy"]["expected_item_visits"], 50_000_500)
        self.assertEqual(report["indexed"]["logical_lookup_operations"], 2_000)
        self.assertEqual(report["indexed"]["additional_index_allocations"], 0)
        self.assertEqual(report["expected_work_reduction_ratio"], 25_000.25)
        self.assertFalse(report["is_product_timing"])

    def test_rejects_empty_pressure_inputs(self):
        with self.assertRaises(ValueError):
            pressure_report(0, 1)
        with self.assertRaises(ValueError):
            pressure_report(1, 0)


if __name__ == "__main__":
    unittest.main()
