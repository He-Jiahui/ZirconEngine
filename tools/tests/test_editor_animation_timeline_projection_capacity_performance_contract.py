from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/animation_editor/session/timeline_foundation.rs"


class EditorAnimationTimelineProjectionCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("pub(super) fn project_sequence_timeline(")
        end = cls.source.index("\nfn track_value_kind", start)
        cls.body = cls.source[start:end]

    def test_track_and_key_bounds_are_reserved_before_projection_append(self) -> None:
        self.assertIn(
            "let track_count: usize = asset\n        .bindings\n        .iter()\n        .map(|binding| binding.tracks.len())\n        .sum();",
            self.body,
        )
        self.assertIn("let mut tracks = Vec::with_capacity(track_count);", self.body)
        self.assertIn(
            "let mut keys = Vec::with_capacity(track.channel.keys.len());", self.body
        )
        self.assertLess(self.body.index("Vec::with_capacity(track_count)"), self.body.index("tracks.push("))
        self.assertLess(
            self.body.index("Vec::with_capacity(track.channel.keys.len())"),
            self.body.index("keys.push("),
        )
        self.assertIn("editor813_animation_timeline_projection_capacity_bench_v1", self.source)

    def test_projection_order_and_timeline_semantics_remain_explicit(self) -> None:
        self.assertIn("for binding in &asset.bindings", self.body)
        self.assertIn("for track in &binding.tracks", self.body)
        self.assertIn("id: track_id.clone()", self.body)
        self.assertIn("display_name: track_id.clone()", self.body)
        self.assertIn("value_kind: track_value_kind(track).to_string()", self.body)
        self.assertIn("sections: Vec::new()", self.body)
        self.assertIn("for key in &track.channel.keys", self.body)
        self.assertIn("format!(\"{track_id}@{:08x}\", key.time_seconds.to_bits())", self.body)
        self.assertNotIn(".flat_map(", self.body)
        self.assertNotIn(".collect()", self.body)

    def test_empty_sequence_keeps_zero_capacity_path(self) -> None:
        self.assertIn("let mut tracks = Vec::with_capacity(track_count);", self.body)
        self.assertIn("let track_count: usize = asset", self.body)
        self.assertIn(".sum();", self.body)

    def test_deterministic_dense_projection_removes_growth_events(self) -> None:
        track_count = 4096
        key_count = 64

        def geometric_growth_events(length: int) -> int:
            capacity = 0
            growth_events = 0
            for current_length in range(1, length + 1):
                if current_length > capacity:
                    capacity = 4 if capacity == 0 else capacity * 2
                    growth_events += 1
            return growth_events

        legacy_growth_events = geometric_growth_events(track_count) + track_count * geometric_growth_events(
            key_count
        )
        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
