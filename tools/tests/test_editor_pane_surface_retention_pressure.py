import json
import tempfile
import unittest
from pathlib import Path

from tools.editor_pane_surface_retention_pressure import run, write_result


class EditorPaneSurfaceRetentionPressureTests(unittest.TestCase):
    def test_stable_updates_do_not_rebuild_retained_pane_surfaces(self) -> None:
        result = run(
            pane_count=64,
            nodes_per_pane=2_048,
            stable_update_count=1_000,
            changed_update_count=0,
            changed_panes_per_update=0,
        )

        self.assertEqual(result["baseline_stable_surface_build_count"], 64_000)
        self.assertEqual(result["retained_stable_surface_build_count"], 0)
        self.assertEqual(result["retained_total_surface_build_count"], 64)
        self.assertEqual(result["stable_surface_build_avoidance_count"], 64_000)
        self.assertEqual(result["stable_surface_build_avoidance_percent"], 100.0)

    def test_single_pane_changes_keep_work_independent_of_other_panes(self) -> None:
        result = run(
            pane_count=64,
            nodes_per_pane=2_048,
            stable_update_count=1_000,
            changed_update_count=1_000,
            changed_panes_per_update=1,
        )

        self.assertEqual(result["baseline_total_surface_build_count"], 128_064)
        self.assertEqual(result["retained_total_surface_build_count"], 1_064)
        self.assertEqual(result["baseline_node_stage_visit_count"], 1_049_100_288)
        self.assertEqual(result["retained_node_stage_visit_count"], 8_716_288)
        self.assertGreater(result["node_stage_visit_reduction_ratio"], 120.0)
        self.assertEqual(result["retained_unchanged_pane_rebuild_count"], 0)

    def test_invalid_inputs_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            run(
                pane_count=0,
                nodes_per_pane=1,
                stable_update_count=1,
                changed_update_count=1,
                changed_panes_per_update=1,
            )
        with self.assertRaises(ValueError):
            run(
                pane_count=2,
                nodes_per_pane=1,
                stable_update_count=1,
                changed_update_count=1,
                changed_panes_per_update=3,
            )

    def test_output_is_stable_json(self) -> None:
        result = run(
            pane_count=2,
            nodes_per_pane=8,
            stable_update_count=3,
            changed_update_count=4,
            changed_panes_per_update=1,
        )
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "pressure.json"
            write_result(output, result)

            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)
            self.assertTrue(output.read_text(encoding="utf-8").endswith("\n"))


if __name__ == "__main__":
    unittest.main()
