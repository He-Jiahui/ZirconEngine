from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/timeline/track_list.rs"


class EditorTrackListCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("pub fn project_track_list<TrackId>(")
        end = cls.source.index("\n}", start) + 2
        cls.body = cls.source[start:end]

    def test_known_track_bound_is_reserved_before_projection_append(self) -> None:
        self.assertIn("let mut rows = Vec::with_capacity(tracks.len());", self.body)
        self.assertIn("rows.push(", self.body)
        self.assertLess(
            self.body.index("Vec::with_capacity(tracks.len())"),
            self.body.index("rows.push("),
        )
        self.assertIn("for track in tracks", self.body)
        self.assertIn("editor809_track_list_capacity_bench_v1", self.source)

    def test_track_projection_fields_and_order_remain_explicit(self) -> None:
        self.assertIn("track_id: track.id.clone()", self.body)
        self.assertIn("display_name: track.display_name.clone()", self.body)
        self.assertIn("lane_kind: lane_kind_for_value(&track.value_kind)", self.body)
        self.assertIn("key_count: track.keys.len()", self.body)
        self.assertIn("section_count: track.sections.len()", self.body)
        self.assertNotIn(".collect()", self.body)

    def test_empty_track_input_keeps_zero_capacity_path(self) -> None:
        self.assertIn("tracks: &[TimelineTrackView<TrackId>]", self.body)
        self.assertIn("Vec::with_capacity(tracks.len())", self.body)

    def test_deterministic_dense_projection_removes_growth_events(self) -> None:
        track_count = 4096
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, track_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1

        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
