import tempfile
import unittest
from pathlib import Path

from tools.analysis.performance.editor.editor_hierarchy_sparse_publication_pressure import (
    SCHEMA,
    run,
    source_binding_report,
    write_result,
)


ROOT = Path(__file__).resolve().parents[2]


class EditorHierarchySparsePublicationPressureTests(unittest.TestCase):
    def test_default_pressure_models_atomic_sparse_publication(self) -> None:
        result = run()

        self.assertEqual(result["schema"], SCHEMA)
        self.assertEqual(
            result["retired_full_publication"]["presentation_snapshot_count"],
            65_536,
        )
        self.assertEqual(
            result["retired_full_publication"][
                "full_workbench_hit_index_rebuild_count"
            ],
            65_536,
        )
        self.assertEqual(
            result["retired_full_publication"]["modeled_workbench_node_visit_count"],
            2_147_483_648,
        )
        self.assertEqual(
            result["atomic_sparse_transaction"]["presentation_snapshot_count"], 0
        )
        self.assertEqual(
            result["atomic_sparse_transaction"][
                "exact_workbench_row_validation_count"
            ],
            65_536,
        )
        self.assertEqual(
            result["atomic_sparse_transaction"]["paint_index_handle_clone_count"],
            4_194_304,
        )
        self.assertEqual(
            result["atomic_sparse_transaction"]["floating_window_visit_count"],
            2_097_152,
        )
        self.assertGreater(result["delta"]["modeled_work_ratio"], 300.0)
        self.assertFalse(result["interpretation"]["is_product_timing"])

    def test_source_binding_covers_current_transaction_and_reference_contracts(self) -> None:
        binding = source_binding_report(ROOT)

        self.assertTrue(binding["ready"])
        self.assertEqual(len(binding["critical_sources"]), 7)
        self.assertEqual(len(binding["source_set_sha256"]), 64)

    def test_non_positive_inputs_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            run(interaction_count=0)

    def test_result_writer_rejects_c_drive(self) -> None:
        with self.assertRaises(ValueError):
            write_result(Path("C:/hierarchy-sparse-publication.json"), run())

    def test_result_writer_creates_a_non_c_artifact(self) -> None:
        with tempfile.TemporaryDirectory(dir=ROOT) as directory:
            output = Path(directory) / "hierarchy-sparse-publication.json"
            write_result(output, run())
            self.assertTrue(output.is_file())


if __name__ == "__main__":
    unittest.main()
