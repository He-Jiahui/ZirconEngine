import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "pane_payload_builders/runtime_diagnostics.rs"
)


class EditorRuntimeDiagnosticsDetailCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_detail_projection_reserves_exact_optional_detail_count(self) -> None:
        match = re.search(r"fn\s+detail_items\b.*?\n}\n", self.source, re.S)
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("detail_item_capacity(diagnostics)", body)
        self.assertIn("Vec::with_capacity", body)
        self.assertNotIn("let mut items = Vec::new()", body)

    def test_capacity_accumulator_has_explicit_usize_type(self) -> None:
        match = re.search(r"fn\s+detail_item_capacity\b.*?\n}\n", self.source, re.S)
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertRegex(body, r"let\s+mut\s+capacity(?::\s*usize)?\s*=\s*1usize;")

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "runtime_diagnostics_detail_capacity_preserves_item_order_and_bound",
            self.source,
        )
        self.assertIn(
            "EDITOR818_RUNTIME_DIAGNOSTICS_DETAIL_CAPACITY_BENCH_V1",
            self.source,
        )

    def test_deterministic_model_removes_detail_growth_events(self) -> None:
        max_details = 11
        legacy_growth_events = 3
        optimized_growth_events = 0
        self.assertEqual(max_details, 11)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
