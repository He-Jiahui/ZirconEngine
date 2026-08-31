import json
import tempfile
import unittest
from pathlib import Path

from tools.editor_build_export_base_cache_pressure import run, write_result


class EditorBuildExportBaseCachePressureTests(unittest.TestCase):
    def test_shared_generation_replaces_payload_clones_with_arc_clones(self) -> None:
        result = run(
            target_count=1_000,
            diagnostic_count=128,
            preset_path_count=1_000,
            stable_cache_hit_count=1_000,
            source_change_count=1,
            wizard_view_model_payload_item_count=2_048,
            stable_wizard_projection_count=1_000,
            wizard_projection_node_count=512,
            wizard_owned_payload_slots_per_node=13,
        )

        self.assertEqual(result["old_payload_item_clone_count"], 2_128_000)
        self.assertEqual(result["new_payload_item_clone_count"], 0)
        self.assertEqual(result["new_arc_clone_count"], 1_000)
        self.assertEqual(result["payload_item_clone_avoidance_count"], 2_128_000)
        self.assertEqual(result["ownership_operation_reduction_ratio"], 2_128.0)
        self.assertEqual(result["metadata_probes_per_identity_check"], 1_002)
        self.assertEqual(result["old_stable_hit_metadata_probe_count"], 1_002_000)
        self.assertEqual(result["new_stable_hit_metadata_probe_count"], 0)
        self.assertEqual(result["stable_hit_metadata_probe_avoidance_count"], 1_002_000)
        self.assertEqual(result["watcher_epoch_load_count"], 1_000)
        self.assertEqual(result["watcher_setup_count"], 1)
        self.assertEqual(result["source_epoch_refresh_count"], 1)
        self.assertEqual(result["watcher_setup_filesystem_probe_count"], 3)
        self.assertEqual(
            result["old_wizard_view_model_item_clone_count"], 2_048_000
        )
        self.assertEqual(result["new_wizard_view_model_item_clone_count"], 0)
        self.assertEqual(result["new_wizard_view_model_borrow_count"], 1_000)
        self.assertEqual(
            result["wizard_view_model_item_clone_avoidance_count"], 2_048_000
        )
        self.assertEqual(
            result["old_wizard_node_payload_slot_clone_count"], 6_656_000
        )
        self.assertEqual(result["new_wizard_node_payload_slot_clone_count"], 0)
        self.assertEqual(
            result["new_wizard_node_payload_slot_move_count"], 6_656_000
        )

    def test_invalid_inputs_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            run(
                target_count=-1,
                diagnostic_count=0,
                preset_path_count=0,
                stable_cache_hit_count=1,
            )

    def test_output_is_stable_json(self) -> None:
        result = run(
            target_count=2,
            diagnostic_count=1,
            preset_path_count=2,
            stable_cache_hit_count=3,
        )
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "pressure.json"
            write_result(output, result)

            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)
            self.assertTrue(output.read_text(encoding="utf-8").endswith("\n"))


if __name__ == "__main__":
    unittest.main()
