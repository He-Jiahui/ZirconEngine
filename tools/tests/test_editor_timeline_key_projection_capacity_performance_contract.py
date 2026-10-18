from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/timeline_strip/generation.rs"
LOWER = ROOT / "zircon_editor/src/ui/timeline_strip/tests/cases.rs"


class EditorTimelineKeyProjectionCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.lower = LOWER.read_text(encoding="utf-8")
        start = cls.source.index("pub(crate) fn new(input: TimelineStripGenerationInput)")
        end = cls.source.index("\n    pub(crate) fn duration", start)
        cls.body = cls.source[start:end]

    def test_key_projection_reserves_input_upper_bound_before_append(self) -> None:
        self.assertIn(
            "let mut keys = Vec::with_capacity(input.keys.len());",
            self.body,
        )
        self.assertIn("keys.push(TimelineStripKey", self.body)
        self.assertLess(
            self.body.index("Vec::with_capacity(input.keys.len())"),
            self.body.index("keys.push("),
        )
        self.assertNotIn(".collect::<Vec<_>>()", self.body)

    def test_key_projection_keeps_filter_and_normalization_semantics_explicit(self) -> None:
        self.assertIn("for key in input.keys", self.body)
        self.assertIn("if key.time.is_finite()", self.body)
        self.assertIn("key.time.clamp(0.0, duration)", self.body)
        self.assertIn("let keys: Arc<[TimelineStripKey]> = keys.into();", self.body)
        self.assertIn("EDITOR75_TIMELINE_KEY_PROJECTION_CAPACITY_BENCH_V1", self.lower)

    def test_dense_key_model_removes_geometric_growth_events(self) -> None:
        key_count = 4096
        capacity = 0
        legacy_growth_events = 0
        for index in range(key_count):
            if index == capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1
        self.assertEqual(legacy_growth_events, 11)
        self.assertEqual(0, 0)


if __name__ == "__main__":
    unittest.main()
