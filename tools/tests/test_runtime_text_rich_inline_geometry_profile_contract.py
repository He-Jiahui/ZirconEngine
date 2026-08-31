import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RICH_RENDER = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs"
)
RICH_INLINE_TESTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_inline.rs"
)
RICH_INLINE_PROFILE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_inline_profile.rs"
)
PROFILE_PLAN = (
    ROOT
    / "docs/plans/zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md"
)


class RuntimeTextRichInlineGeometryProfileContractTests(unittest.TestCase):
    def test_profile_uses_fixed_low_cardinality_counter_names(self) -> None:
        source = RICH_RENDER.read_text(encoding="utf-8")
        names = [
            "rich_inline_run_count",
            "rich_inline_line_probe_count",
            "rich_inline_line_run_probe_count",
            "rich_inline_prefix_grapheme_count",
            "rich_inline_prefix_advance_count",
            "rich_inline_paint_frame_match_count",
            "rich_inline_paint_frame_mismatch_count",
        ]

        for name in names:
            self.assertEqual(source.count(f'"{name}"'), 1, name)
        self.assertNotIn("format!(", source[source.index("fn publish(&self)") :])

    def test_profile_fields_are_absent_from_ordinary_builds(self) -> None:
        source = RICH_RENDER.read_text(encoding="utf-8")
        profile_start = source.index("struct RichInlineGeometryProfile")
        profile_end = source.index("pub(super) struct RichTextRunPresentation", profile_start)
        profile = source[profile_start:profile_end]

        self.assertGreaterEqual(profile.count('#[cfg(feature = "profiling")]'), 9)
        self.assertIn('#[cfg(not(feature = "profiling"))]', profile)
        self.assertNotIn("String", profile)
        self.assertNotIn("Vec<", profile)

    def test_real_layout_regression_checks_work_and_frame_agreement(self) -> None:
        tests = RICH_INLINE_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "rich_inline_geometry_profile_reports_existing_prefix_reconstruction_work",
            tests,
        )
        self.assertIn("expected_line_run_probes", tests)
        self.assertIn("expected_prefix_graphemes", tests)
        self.assertIn('counter("rich_inline_paint_frame_match_count")', tests)
        self.assertIn('counter("rich_inline_paint_frame_mismatch_count"), 0', tests)

    def test_release_profile_harness_covers_direction_and_line_search_lanes(self) -> None:
        tests = RICH_INLINE_TESTS.read_text(encoding="utf-8")
        profile = RICH_INLINE_PROFILE.read_text(encoding="utf-8")

        self.assertIn('path = "rich_inline_profile.rs"', tests)
        self.assertIn("mod rich_inline_profile;", tests)
        self.assertIn("const SAMPLE_COUNT: usize = 31;", profile)
        self.assertIn("DenseLtr", profile)
        self.assertIn("DenseRtl", profile)
        self.assertIn("DenseVerticalRl", profile)
        self.assertIn("WrappedLines", profile)
        self.assertIn("samples_ns={samples_ns:?}", profile)
        self.assertIn("rss_delta_bytes={rss_delta_bytes:?}", profile)

    def test_release_profile_harness_reports_all_geometry_work_counters(self) -> None:
        profile = RICH_INLINE_PROFILE.read_text(encoding="utf-8")
        names = [
            "rich_inline_run_count",
            "rich_inline_line_probe_count",
            "rich_inline_line_run_probe_count",
            "rich_inline_prefix_grapheme_count",
            "rich_inline_prefix_advance_count",
            "rich_inline_paint_frame_match_count",
            "rich_inline_paint_frame_mismatch_count",
        ]

        for name in names:
            self.assertEqual(profile.count(f'"{name}"'), 1, name)
        self.assertNotIn("target/", profile)
        self.assertNotIn("File::create", profile)

    def test_plan_keeps_baseline_and_cutover_pending(self) -> None:
        plan = PROFILE_PLAN.read_text(encoding="utf-8")

        self.assertIn("baseline_profile_pending_no_optimization", plan)
        self.assertIn("Do not change geometry ownership before the baseline", plan)
        self.assertIn("31 raw samples", plan)
        self.assertIn("FSlateImageRun", plan)
        self.assertIn("FSlateWidgetRun", plan)


if __name__ == "__main__":
    unittest.main()
