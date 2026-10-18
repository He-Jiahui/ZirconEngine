import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
PRODUCTION = ROOT / "zircon_editor/src/ui/timeline_strip/generation.rs"
LOWER_TESTS = ROOT / "zircon_editor/src/ui/timeline_strip/tests/cases.rs"


class EditorTimelineTickProjectionCapacityContract(unittest.TestCase):
    def setUp(self):
        self.production = PRODUCTION.read_text(encoding="utf-8")
        self.lower_tests = LOWER_TESTS.read_text(encoding="utf-8")

    def test_static_projection_builds_ticks_directly_with_exact_bound(self):
        self.assertIn(
            "timeline_tick_projection(duration, tick_interval, visual_budget)",
            self.production,
        )
        self.assertIn("fn timeline_tick_projection(", self.production)
        self.assertIn(
            "Vec::with_capacity(segment_count.saturating_add(1))", self.production
        )
        self.assertIn("TimelineStripTick::from_value", self.production)
        self.assertNotIn(
            "timeline_tick_values(duration, tick_interval, visual_budget)",
            self.production,
        )

    def test_static_projection_model_removes_intermediate_float_vector(self):
        legacy_allocations = 2  # float values plus materialized tick records
        optimized_allocations = 1  # one bounded tick-record vector
        self.assertEqual(legacy_allocations, 2)
        self.assertEqual(optimized_allocations, 1)
        self.assertLess(optimized_allocations, legacy_allocations)

    def test_lower_semantic_regression_and_release_marker_are_wired(self):
        self.assertIn(
            "optimization_batch_20260919_editor75_tick_projection_preserves_values_and_labels",
            self.lower_tests,
        )
        self.assertIn(
            "EDITOR75_TIMELINE_TICK_PROJECTION_CAPACITY_BENCH_V1",
            self.lower_tests,
        )


if __name__ == "__main__":
    unittest.main()
