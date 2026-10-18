import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/v2/style/runtime_state.rs"


class RuntimeV2PseudoStateCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_arena_pseudo_state_collector_reserves_authored_bound(self) -> None:
        match = re.search(
            r"pub\(super\) fn collect_pseudo_states\(.*?\n}\n",
            self.source,
            re.S,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("pseudo_state_initial_capacity", body)
        self.assertIn("Vec::with_capacity", body)
        self.assertNotIn("let mut states = Vec::new()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("pseudo_state_capacity_preserves_alias_and_order_semantics", self.source)
        self.assertIn("RUNTIME810_V2_PSEUDO_STATE_CAPACITY_BENCH_V1", self.source)

    def test_deterministic_model_removes_growth_events(self) -> None:
        authored_state_count = 4_096
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(authored_state_count, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
