import unittest

from tools.runtime_rich_table_track_metrics_pressure import pressure_report


class RuntimeRichTableTrackMetricsPressureTest(unittest.TestCase):
    def test_default_large_span_table_is_linear_in_cell_count(self) -> None:
        result = pressure_report()
        repeated = result["repeated_span_summation"]
        prefix = result["gap_aware_prefix_metrics"]
        delta = result["delta"]
        payload = result["geometry_payload_estimate"]

        self.assertEqual(repeated["span_track_visits"], 1_040_000)
        self.assertEqual(repeated["combined_work_units"], 1_092_512)
        self.assertEqual(prefix["span_query_work_units"], 50_000)
        self.assertEqual(prefix["combined_work_units"], 101_258)
        self.assertEqual(delta["combined_work_reduction_ratio"], 10.789389)
        self.assertEqual(delta["span_work_reduction_ratio"], 20.8)
        self.assertEqual(payload["delta_bytes"], 8)
        self.assertFalse(result["is_product_timing"])

    def test_invalid_spans_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            pressure_report(column_count=4, average_column_span=5)
        with self.assertRaises(ValueError):
            pressure_report(row_count=4, average_row_span=5)


if __name__ == "__main__":
    unittest.main()
