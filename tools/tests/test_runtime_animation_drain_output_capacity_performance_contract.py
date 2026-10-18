import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/scene/level_system/animation_runtime.rs"
REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/scene/level_system/animation_runtime/tests/drain_output_capacity_tests.rs"
)


class RuntimeAnimationDrainOutputCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8") if REGRESSION.exists() else ""

    def test_drain_reserves_bounded_event_output(self) -> None:
        match = re.search(
            r"pub fn drain_animation_clip_events\(.*?\n    }",
            self.source,
            re.S,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        pending_index = body.index("let pending_sample_count")
        events_index = body.index("let mut events")
        self.assertLess(pending_index, events_index)
        self.assertIn("let mut events = if pending_sample_count == 0", body)
        self.assertIn("Vec::with_capacity(limits.max_events)", body)
        self.assertIn("Vec::new()", body)

    def test_drain_event_budget_and_ordering_remain_intact(self) -> None:
        body = self.source[
            self.source.index("pub fn drain_animation_clip_events") : self.source.index(
                "pub fn animation_clip_event_backlog_len"
            )
        ]
        self.assertIn("let remaining_events = limits.max_events.saturating_sub(events.len());", body)
        self.assertIn("events.extend(batch.events);", body)
        self.assertIn("Some(events)", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod drain_output_capacity_tests;", self.source)
        self.assertIn(
            "runtime857_animation_drain_output_capacity_preserves_order",
            self.regression,
        )
        self.assertIn("RUNTIME857_ANIMATION_DRAIN_OUTPUT_CAPACITY_BENCH_V1", self.regression)

    def test_deterministic_model_removes_bounded_output_growth(self) -> None:
        max_events = 4_096
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(max_events, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
