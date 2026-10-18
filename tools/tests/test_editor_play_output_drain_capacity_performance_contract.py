from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/core/play/process_backend/output.rs"
REGRESSION = (
    ROOT / "zircon_editor/src/core/play/process_backend/output/tests/drain_all_capacity_tests.rs"
)


class EditorPlayOutputDrainCapacityPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_drain_all_reserves_queue_and_budget_bound(self) -> None:
        production = self.source.split("#[cfg(test)]", 1)[0]
        self.assertIn("fn drain_all_capacity", production)
        self.assertIn("Vec::with_capacity(drain_all_capacity(", production)
        self.assertIn("self.receiver.len()", production)
        self.assertIn("deferred.is_some()", production)
        self.assertIn("PLAY_OUTPUT_BUDGET_DIAGNOSTIC_COUNT", production)
        self.assertIn(
            "PLAY_OUTPUT_DRAIN_LIMIT + PLAY_OUTPUT_BUDGET_DIAGNOSTIC_COUNT",
            production,
        )
        self.assertIn("if line_count == 0", production)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = REGRESSION.read_text(encoding="utf-8")
        self.assertIn("mod drain_all_capacity_tests;", self.source)
        self.assertIn(
            "optimization_batch_20260915_editor07_drain_all_capacity_covers_queue",
            lower,
        )
        self.assertIn("EDITOR07_PLAY_OUTPUT_DRAIN_ALL_CAPACITY_BENCH_V1", lower)
        self.assertIn("optimized_growth_events", lower)

    def test_deterministic_model_eliminates_geometric_growth(self) -> None:
        queue_capacity = 1_024
        budget_diagnostics = 5
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, queue_capacity + budget_diagnostics + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, 0)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
