import unittest
from pathlib import Path

from tools.analysis.performance.editor.editor_runtime_diagnostics_surface_pressure import (
    pressure_report,
    run,
    validate_source_contract,
)


ROOT = Path(__file__).resolve().parents[2]


class EditorRuntimeDiagnosticsSurfacePressureTests(unittest.TestCase):
    def test_published_frame_eliminates_duplicate_surface_work(self) -> None:
        result = pressure_report(recompute_count=200, row_count=256)
        legacy = result["legacy_duplicate_surface"]
        current = result["current_published_frame_read"]
        delta = result["delta"]

        self.assertEqual(legacy["surface_build_count"], 200)
        self.assertEqual(current["surface_build_count"], 0)
        self.assertEqual(legacy["tree_node_materialization_count"], 51_400)
        self.assertEqual(current["tree_node_materialization_count"], 0)
        self.assertEqual(delta["avoided_layout_rebuild_count"], 200)
        projection = result["live_reflector_projection"]
        self.assertEqual(projection["legacy_intermediate_vec_count"], 400)
        self.assertEqual(projection["current_intermediate_vec_count"], 0)
        self.assertEqual(
            projection["legacy_intermediate_owned_string_product_count"],
            62_600,
        )
        self.assertEqual(projection["current_intermediate_owned_string_product_count"], 0)
        self.assertFalse(result["interpretation"]["timing_claim"])
        self.assertEqual(
            result["schema"],
            "zircon.editor.runtime_diagnostics_surface_pressure.v2",
        )

    def test_current_source_binding_and_guards(self) -> None:
        validate_source_contract(ROOT)
        result = run(ROOT)
        self.assertEqual(len(result["source_binding"]["files"]), 6)

    def test_negative_counts_are_rejected_by_pressure_contract(self) -> None:
        with self.assertRaises(ValueError):
            pressure_report(recompute_count=-1)


if __name__ == "__main__":
    unittest.main()
