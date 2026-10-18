import unittest
from pathlib import Path

from tools.analysis.performance.editor.editor_runtime_diagnostics_scope_pressure import (
    pressure_report,
    run,
    validate_source_contract,
)


ROOT = Path(__file__).resolve().parents[2]


class EditorRuntimeDiagnosticsScopePressureTests(unittest.TestCase):
    def test_document_or_floating_refresh_is_the_remaining_global_boundary(self) -> None:
        result = pressure_report(refresh_count=200, unrelated_pane_count=8)
        current = result["current_document_or_floating_fallback"]
        target = result["target_pane_scoped_publication"]
        delta = result["delta"]

        self.assertEqual(current["full_presentation_refresh_count"], 200)
        self.assertEqual(current["unrelated_pane_visit_count"], 1_600)
        self.assertEqual(target["diagnostic_pane_payload_build_count"], 200)
        self.assertEqual(target["unrelated_pane_visit_count"], 0)
        self.assertEqual(target["damage_region_count_per_refresh"], 1)
        self.assertEqual(delta["avoided_full_presentation_refresh_count"], 200)
        self.assertEqual(delta["avoided_unrelated_pane_visits"], 1_600)
        self.assertFalse(result["interpretation"]["timing_claim"])

    def test_source_contract_keeps_drawer_fast_path_and_typed_fallbacks(self) -> None:
        validate_source_contract(ROOT)
        result = run(ROOT)
        self.assertEqual(len(result["source_binding"]["files"]), 7)
        self.assertEqual(
            result["schema"],
            "zircon.editor.runtime_diagnostics_scope_pressure.v2",
        )

    def test_negative_counts_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            pressure_report(refresh_count=-1)
        with self.assertRaises(ValueError):
            pressure_report(unrelated_pane_count=-1)


if __name__ == "__main__":
    unittest.main()
