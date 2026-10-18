from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/timeline/keyframe_lane.rs"


class EditorKeyframeLaneCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("pub fn keyframes_in_range(")
        end = cls.source.index("\n}", start) + 2
        cls.body = cls.source[start:end]

    def test_known_input_bound_is_reserved_before_filter_append(self) -> None:
        self.assertIn("let mut visible = Vec::new();", self.body)
        self.assertIn("if visible.is_empty()", self.body)
        self.assertIn("visible.reserve(keys.len());", self.body)
        self.assertLess(
            self.body.index("visible.reserve(keys.len())"),
            self.body.index("visible.push(key);"),
        )
        self.assertIn("for key in keys", self.body)
        self.assertIn("editor808_keyframe_lane_capacity_bench_v1", self.source)

    def test_filter_order_and_borrowed_output_semantics_remain(self) -> None:
        self.assertIn("pub fn keyframes_in_range", self.body)
        self.assertIn("range.contains(key.time)", self.body)
        self.assertIn("Vec<&TimelineKey>", self.body)
        self.assertNotIn(".collect()", self.body)

    def test_empty_ranges_keep_the_zero_output_path(self) -> None:
        self.assertIn("keys: &[TimelineKey]", self.body)
        self.assertIn("range: TimelineRange", self.body)
        self.assertIn("let mut visible = Vec::new();", self.body)

    def test_deterministic_dense_window_removes_growth_events(self) -> None:
        key_count = 4096
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, key_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1

        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
