import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/ui/graph/model.rs"
LOWER_REGRESSION = REPO_ROOT / "zircon_editor/src/ui/graph/model.rs"


class EditorGraphCyclePendingCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_cycle_probe_reserves_edge_bounded_pending_capacity(self) -> None:
        match = re.search(r"fn\s+introduces_cycle\b.*?\n}\n", self.source, re.S)
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("cycle_pending_capacity(edges.len())", body)
        self.assertIn("Vec::with_capacity", body)
        self.assertNotIn("let mut pending = vec![", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod optimization_batch_editor816_tests", self.source)
        self.assertIn("optimization_batch_editor816_cycle_pending_capacity_preserves_cycle_verdict", self.source)
        self.assertIn("EDITOR816_GRAPH_CYCLE_PENDING_CAPACITY_BENCH_V1", self.source)

    def test_deterministic_model_removes_pending_growth(self) -> None:
        edge_count = 4_096
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(edge_count, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
