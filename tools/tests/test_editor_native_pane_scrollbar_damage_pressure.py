from pathlib import Path
import tempfile
import unittest

from tools.editor_native_pane_scrollbar_damage_pressure import run, write_result


class EditorNativePaneScrollbarDamagePressureTests(unittest.TestCase):
    def test_one_subview_damage_reduces_prepare_work_to_intersections(self) -> None:
        result = run()

        self.assertEqual(result["legacy_metadata_lookups"], 16_000)
        self.assertEqual(result["target_metadata_lookups"], 4_000)
        self.assertEqual(result["target_damage_probes"], 16_000)
        self.assertEqual(result["target_descriptor_heap_allocations"], 0)
        self.assertEqual(result["target_descriptor_inline_capacity"], 4)
        self.assertEqual(result["legacy_geometry_evaluations"], 16_000)
        self.assertEqual(result["target_geometry_evaluations"], 4_000)
        self.assertEqual(result["geometry_evaluation_reduction_ratio"], 4.0)

    def test_full_damage_preserves_bounded_descriptor_work(self) -> None:
        result = run(intersecting_descriptors_per_paint=4)

        self.assertEqual(
            result["legacy_geometry_evaluations"],
            result["target_geometry_evaluations"],
        )
        self.assertEqual(result["target_descriptor_publications"], 8)

    def test_invalid_workloads_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            run(pane_paint_count=0)
        with self.assertRaises(ValueError):
            run(intersecting_descriptors_per_paint=5)
        with self.assertRaises(ValueError):
            run(descriptors_per_pane=5)

    def test_artifact_writer_rejects_c_drive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "result.json"
            if output.resolve().drive.upper() == "C:":
                with self.assertRaises(ValueError):
                    write_result(output, run())
            else:
                write_result(output, run())
                self.assertTrue(output.exists())


if __name__ == "__main__":
    unittest.main()
