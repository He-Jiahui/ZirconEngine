import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from tools.editor_inspector_projection_pressure import (
    REFERENCE_SOURCE_PATHS,
    SOURCE_PATHS,
    build_source_binding,
    run,
    validate_source_contract,
    write_result,
)


ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "tools/editor_inspector_projection_pressure.py"


class EditorInspectorProjectionPressureTests(unittest.TestCase):
    def test_stable_and_resize_work_do_not_rematerialize_payload_rows(self) -> None:
        result = run(
            plugin_property_count=10_000,
            authored_node_count=256,
            physical_slot_count=64,
            stable_recompute_count=1_000,
            resize_step_count=200,
            delta_update_count=0,
            changed_fields_per_delta=0,
            cache_entry_limit=8,
        )

        self.assertEqual(result["baseline_stable_property_materialization_count"], 10_000_000)
        self.assertEqual(result["baseline_resize_property_materialization_count"], 0)
        self.assertEqual(result["current_metrics_fast_path_count"], 200)
        self.assertEqual(result["current_metrics_fallback_count"], 0)
        self.assertEqual(
            result["rejected_resize_full_rebuild_property_materialization_count"],
            2_000_000,
        )
        self.assertEqual(result["retained_stable_property_materialization_count"], 0)
        self.assertEqual(result["retained_resize_property_materialization_count"], 0)
        self.assertEqual(result["retained_stable_surface_build_count"], 0)
        self.assertEqual(result["retained_resize_surface_build_count"], 0)

    def test_sparse_updates_scale_with_changed_fields_not_total_payload(self) -> None:
        result = run(
            plugin_property_count=10_000,
            authored_node_count=256,
            physical_slot_count=64,
            stable_recompute_count=1_000,
            resize_step_count=200,
            delta_update_count=1_000,
            changed_fields_per_delta=1,
            cache_entry_limit=8,
        )

        self.assertEqual(result["baseline_delta_property_materialization_count"], 10_000_000)
        self.assertEqual(result["retained_delta_property_materialization_count"], 1_000)
        self.assertEqual(result["baseline_total_property_materialization_count"], 20_010_000)
        self.assertEqual(result["current_pane_payload_property_copy_count"], 20_010_000)
        self.assertEqual(result["current_two_stage_property_record_work"], 40_020_000)
        self.assertEqual(result["retained_total_property_materialization_count"], 11_000)
        self.assertGreater(result["property_materialization_reduction_ratio"], 1_800.0)
        self.assertGreater(result["two_stage_property_record_work_ratio"], 3_600.0)

    def test_metrics_fallback_cost_is_explicit_and_bounded_by_resize_steps(self) -> None:
        result = run(
            plugin_property_count=10_000,
            authored_node_count=256,
            physical_slot_count=64,
            stable_recompute_count=0,
            resize_step_count=200,
            delta_update_count=0,
            changed_fields_per_delta=0,
            cache_entry_limit=8,
            resize_fallback_count=2,
        )

        self.assertEqual(result["current_metrics_fast_path_count"], 198)
        self.assertEqual(result["current_metrics_fallback_count"], 2)
        self.assertEqual(result["baseline_resize_property_materialization_count"], 20_000)
        self.assertEqual(result["baseline_resize_surface_build_count"], 2)

        with self.assertRaises(ValueError):
            run(
                plugin_property_count=10_000,
                authored_node_count=256,
                physical_slot_count=64,
                stable_recompute_count=0,
                resize_step_count=1,
                delta_update_count=0,
                changed_fields_per_delta=0,
                cache_entry_limit=8,
                resize_fallback_count=2,
            )

    def test_cache_memory_is_bounded_by_physical_slots_and_shares_logical_source(self) -> None:
        result = run(
            plugin_property_count=10_000,
            authored_node_count=256,
            physical_slot_count=64,
            stable_recompute_count=0,
            resize_step_count=0,
            delta_update_count=0,
            changed_fields_per_delta=0,
            cache_entry_limit=8,
        )

        self.assertEqual(result["retained_logical_property_copy_count"], 10_000)
        self.assertEqual(result["retained_surface_node_capacity"], 2_560)
        self.assertEqual(result["retained_cached_payload_property_capacity"], 10_000)

    def test_invalid_inputs_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            run(
                plugin_property_count=0,
                authored_node_count=1,
                physical_slot_count=1,
                stable_recompute_count=0,
                resize_step_count=0,
                delta_update_count=0,
                changed_fields_per_delta=0,
                cache_entry_limit=1,
            )
        with self.assertRaises(ValueError):
            run(
                plugin_property_count=10,
                authored_node_count=1,
                physical_slot_count=11,
                stable_recompute_count=0,
                resize_step_count=0,
                delta_update_count=0,
                changed_fields_per_delta=0,
                cache_entry_limit=1,
            )
        with self.assertRaises(ValueError):
            run(
                plugin_property_count=10,
                authored_node_count=1,
                physical_slot_count=1,
                stable_recompute_count=0,
                resize_step_count=0,
                delta_update_count=1,
                changed_fields_per_delta=11,
                cache_entry_limit=1,
            )

    def test_output_is_stable_json_and_rejects_c_drive(self) -> None:
        result = run(
            plugin_property_count=10,
            authored_node_count=4,
            physical_slot_count=2,
            stable_recompute_count=3,
            resize_step_count=2,
            delta_update_count=4,
            changed_fields_per_delta=1,
            cache_entry_limit=2,
        )
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "pressure.json"
            write_result(output, result)
            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)
            self.assertTrue(output.read_text(encoding="utf-8").endswith("\n"))

        with self.assertRaises(ValueError):
            write_result(Path("C:/temp/inspector-pressure.json"), result)
        with self.assertRaises(ValueError):
            write_result(Path("relative/inspector-pressure.json"), result)

    def test_current_source_binding_is_ready_and_content_hashed(self) -> None:
        binding = build_source_binding(ROOT)

        self.assertTrue(binding["ready"], binding["blockers"])
        self.assertEqual(len(binding["git_revision"]), 40)
        self.assertEqual(
            [entry["relative_path"] for entry in binding["critical_source_files"]],
            list(SOURCE_PATHS),
        )
        self.assertTrue(
            all(len(entry["sha256"]) == 64 for entry in binding["critical_source_files"])
        )
        self.assertEqual(
            [entry["relative_path"] for entry in binding["reference_source_files"]],
            list(REFERENCE_SOURCE_PATHS),
        )
        self.assertEqual(len(binding["source_set_sha256"]), 64)

    def test_source_guard_fails_closed_when_full_projection_shape_changes(self) -> None:
        sources = {
            relative_path: (ROOT / relative_path).read_text(encoding="utf-8")
            for relative_path in SOURCE_PATHS
        }
        current = validate_source_contract(sources)
        self.assertTrue(current["ready"], current["blockers"])

        projection_path = (
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "inspector_projection.rs"
        )
        sources[projection_path] = sources[projection_path].replace(
            "build_shared_surface", "build_shared_surface_removed", 1
        )
        changed = validate_source_contract(sources)

        self.assertFalse(changed["ready"])
        self.assertIn(
            projection_path,
            {blocker.get("relative_path") for blocker in changed["blockers"]},
        )

    def test_cli_returns_nonzero_when_source_binding_is_not_ready(self) -> None:
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            root = Path(directory)
            output = root / "not-ready.json"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "--repo-root",
                    str(root),
                    "--output",
                    str(output),
                ],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
            )

            self.assertEqual(completed.returncode, 2)
            self.assertFalse(json.loads(output.read_text(encoding="utf-8"))["ready"])


if __name__ == "__main__":
    unittest.main()
