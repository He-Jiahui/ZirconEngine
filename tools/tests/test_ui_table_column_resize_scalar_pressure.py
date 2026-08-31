import json
import tempfile
import unittest
from pathlib import Path

from tools.ui_table_column_resize_scalar_pressure import run, write_result


class UiTableColumnResizeScalarPressureTests(unittest.TestCase):
    def test_default_model_removes_per_move_aggregate_round_trips(self) -> None:
        result = run()

        self.assertEqual(result["legacy_width_map_entry_visits"], 1_536_000)
        self.assertEqual(result["legacy_column_array_entry_visits"], 13_824_000)
        self.assertEqual(result["legacy_column_match_checks"], 256_000)
        self.assertEqual(result["legacy_property_transactions"], 4_000)
        self.assertEqual(result["legacy_combined_work_units"], 15_620_000)
        self.assertEqual(result["target_combined_work_units"], 12_864)
        self.assertGreater(result["work_reduction_ratio"], 1_200.0)

    def test_cadence_flush_cost_is_explicit_and_bounded(self) -> None:
        result = run(
            column_count=16,
            pointer_move_count=100,
            column_metadata_entry_count=4,
            compatibility_flush_count=4,
            scalar_operations_per_move=2,
        )

        self.assertEqual(result["target_schema_build_entry_visits"], 80)
        self.assertEqual(result["target_scalar_work_units"], 200)
        self.assertEqual(result["target_property_transactions"], 100)
        self.assertEqual(result["target_compatibility_projection_entry_visits"], 384)
        self.assertEqual(result["target_combined_work_units"], 764)

    def test_invalid_inputs_and_c_drive_output_fail_closed(self) -> None:
        with self.assertRaises(ValueError):
            run(column_count=0)
        with self.assertRaises(ValueError):
            run(pointer_move_count=0)
        with self.assertRaises(ValueError):
            run(column_metadata_entry_count=0)
        with self.assertRaises(ValueError):
            run(compatibility_flush_count=-1)
        with self.assertRaises(ValueError):
            run(pointer_move_count=4, compatibility_flush_count=5)
        with self.assertRaises(ValueError):
            run(scalar_operations_per_move=0)
        with self.assertRaises(ValueError):
            write_result(Path("C:/table-column-resize-pressure.json"), run())

    def test_output_is_stable_json_on_external_storage(self) -> None:
        result = run(column_count=8, pointer_move_count=12)
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "table-column-resize-pressure.json"
            write_result(output, result)
            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)


if __name__ == "__main__":
    unittest.main()
