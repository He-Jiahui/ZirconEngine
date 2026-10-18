import pathlib
import re
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
MEASURE = ROOT / "zircon_runtime/src/text/layout/measure.rs"
CURSOR = ROOT / "zircon_runtime/src/text/layout/measure/grapheme_projection.rs"


class RuntimeTextGraphemeProjectionCursorContractTests(unittest.TestCase):
    def test_measurement_uses_a_monotonic_cursor_for_cluster_bounds(self):
        source = MEASURE.read_text(encoding="utf-8")
        cursor = CURSOR.read_text(encoding="utf-8")

        self.assertIn("GraphemeProjectionCursor", source)
        self.assertIn("projection_cursor.overlap_bounds(", source)
        self.assertNotIn("graphemes.partition_point", source)
        self.assertIn("self.previous", cursor)
        self.assertIn("while self.first < graphemes.len()", cursor)
        self.assertIn("while self.after_last < graphemes.len()", cursor)

    def test_reordered_clusters_keep_the_binary_search_fallback(self):
        cursor = CURSOR.read_text(encoding="utf-8")

        self.assertIn("if !monotonic", cursor)
        self.assertIn(
            "graphemes.partition_point(|&(_, end)| end <= cluster_start)",
            cursor,
        )
        self.assertIn(
            "graphemes.partition_point(|&(start, _)| start < cluster_end)",
            cursor,
        )
        self.assertIn(
            "cursor_advances_monotonic_ranges_and_falls_back_for_reordered_ranges",
            cursor,
        )

    def test_measurement_reuses_the_grapheme_pass_for_tab_presence(self):
        source = MEASURE.read_text(encoding="utf-8")

        self.assertIn(
            "project_grapheme_geometry_with_tab_presence_from_shaped", source
        )
        self.assertRegex(
            source,
            re.compile(
                r"project_grapheme_geometry_with_tab_presence_from_shaped::<\s*TRACK_TAB,?\s*>"
            ),
        )
        self.assertIn("contains_tab |= grapheme == \"\\t\";", source)
        self.assertIn("measure_line_with_tab_presence_with_provider", source)
        self.assertRegex(
            source,
            re.compile(
                r"measure_line_with_tab_presence_with_provider::<\s*false(?:\s*,\s*P)?\s*>"
            ),
        )
        self.assertRegex(
            source,
            re.compile(
                r"measure_line_with_tab_presence_with_provider::<\s*true(?:\s*,\s*P)?\s*>"
            ),
        )
        self.assertIn("if !contains_tab {", source)
        self.assertNotIn("text.contains('\\t')", source)

    def test_plain_projection_reuses_aligned_grapheme_boundaries(self):
        source = MEASURE.read_text(encoding="utf-8")
        cursor = CURSOR.read_text(encoding="utf-8")

        self.assertIn("aligned_grapheme_span(", source)
        self.assertIn("if let Some(cluster_span) = aligned_grapheme_span", source)
        self.assertIn("fn aligned_grapheme_span", cursor)
        self.assertIn("aligned_grapheme_span_rejects_partial_boundaries", cursor)

    def test_measure_owner_stays_within_the_existing_file_budget(self):
        self.assertLessEqual(len(MEASURE.read_text(encoding="utf-8").splitlines()), 800)


if __name__ == "__main__":
    unittest.main()
