import json
import tempfile
import unittest
from pathlib import Path

from tools.editor_asset_pointer_snapshot_pressure import run, write_result


class EditorAssetPointerSnapshotPressureTests(unittest.TestCase):
    def test_pointer_projection_drops_unrelated_snapshot_work(self) -> None:
        result = run()

        self.assertEqual(result["legacy_unrelated_row_units_per_publication"], 2_128)
        self.assertEqual(result["legacy_clone_work_units"], 12_149_000)
        self.assertEqual(result["target_projection_work_units"], 10_016_000)
        self.assertEqual(result["removed_work_units"], 2_133_000)
        self.assertGreater(result["work_reduction_ratio"], 1.2)
        self.assertFalse(result["timing_claim"])

    def test_invalid_dimensions_and_c_drive_output_fail_closed(self) -> None:
        with self.assertRaises(ValueError):
            run(folder_tree_rows=-1)
        with self.assertRaises(ValueError):
            run(stable_publications=0)
        with self.assertRaises(ValueError):
            write_result(Path("C:/asset-pointer-snapshot-pressure.json"), run())

    def test_output_is_stable_json_on_external_storage(self) -> None:
        result = run(folder_tree_rows=8, visible_folder_rows=2, stable_publications=3)
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "asset-pointer-snapshot-pressure.json"
            write_result(output, result)
            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)


if __name__ == "__main__":
    unittest.main()
