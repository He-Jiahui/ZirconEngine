import json
import tempfile
import unittest
from pathlib import Path

from tools.runtime_tree_logical_index_pressure import run, write_result


class RuntimeTreeLogicalIndexPressurePerformanceContractTests(unittest.TestCase):
    def test_generation_owned_index_decouples_clicks_from_logical_tree_size(self) -> None:
        result = run(
            node_count=100_000,
            single_interaction_count=1_000,
            range_interaction_count=1_000,
            range_width=10,
            legacy_full_passes_per_interaction=2,
        )

        self.assertEqual(result["old_single_logical_node_visit_count"], 200_000_000)
        self.assertEqual(result["new_single_logical_node_visit_count"], 0)
        self.assertEqual(result["old_range_logical_node_visit_count"], 200_000_000)
        self.assertEqual(result["new_range_logical_node_visit_count"], 10_000)
        self.assertEqual(result["old_temporary_id_vector_entry_count"], 400_000_000)
        self.assertEqual(result["new_temporary_id_vector_entry_count"], 0)
        self.assertEqual(result["old_temporary_dedup_entry_count"], 400_000_000)
        self.assertEqual(result["new_temporary_dedup_entry_count"], 0)
        self.assertEqual(result["logical_visit_reduction_ratio"], 40_000.0)

    def test_invalid_inputs_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            run(
                node_count=100,
                single_interaction_count=1,
                range_interaction_count=1,
                range_width=101,
                legacy_full_passes_per_interaction=2,
            )

    def test_output_is_stable_json_and_rejects_c_drive(self) -> None:
        result = run(
            node_count=10,
            single_interaction_count=2,
            range_interaction_count=1,
            range_width=3,
            legacy_full_passes_per_interaction=2,
        )
        with tempfile.TemporaryDirectory(dir=Path("E:/zircon-profiles")) as directory:
            output = Path(directory) / "tree-pressure.json"
            write_result(output, result)
            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), result)

        with self.assertRaises(ValueError):
            write_result(Path("C:/tree-pressure.json"), result)


if __name__ == "__main__":
    unittest.main()
