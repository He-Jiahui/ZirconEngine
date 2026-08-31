import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TEXT_SHAPE = ROOT / "zircon_runtime_interface/src/ui/surface/render/text_shape.rs"
TEXT_SHAPE_TESTS = (
    ROOT
    / "zircon_runtime_interface/src/ui/surface/render/text_shape/resolved_layout_tests.rs"
)
RESOLVED_LAYOUT = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout/rich_artifact_routes.rs"
)
RICH_RENDER = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs"
)
TEXT_BATCHES = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_batches.rs"
)
SCREEN_RENDER = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
RICH_ROUTE_TESTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_artifact_routes.rs"
)
RICH_PROJECTION_TESTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_projection_admission.rs"
)


class RuntimeTextPaintRunFailClosedContractTests(unittest.TestCase):
    def test_interface_projection_is_all_or_empty_for_nonempty_runs(self) -> None:
        source = TEXT_SHAPE.read_text(encoding="utf-8")
        tests = TEXT_SHAPE_TESTS.read_text(encoding="utf-8")

        self.assertIn('path = "text_shape/resolved_layout_tests.rs"', source)
        self.assertIn("mod resolved_layout_tests;", source)
        self.assertIn("return Vec::new();", source)
        self.assertIn("text.is_char_boundary(visual_range.start)", source)
        self.assertIn(
            "resolved_paint_projection_rejects_the_complete_batch_when_one_run_has_invalid_advances",
            tests,
        )
        self.assertIn(
            "resolved_paint_projection_rejects_invalid_visual_utf8_ranges",
            tests,
        )
        self.assertIn(
            "resolved_paint_projection_preserves_scalar_aligned_style_boundaries_inside_a_grapheme",
            tests,
        )
        self.assertIn(
            "resolved_paint_projection_rejects_run_text_that_disagrees_with_visual_slice",
            tests,
        )
        self.assertIn(
            "resolved_paint_projection_rejects_non_contiguous_visual_runs", tests
        )
        self.assertIn("run.visual_range.start != expected_visual_start", source)
        self.assertIn(
            "line.text.get(run.visual_range.start..run.visual_range.end)", source
        )
        self.assertIn("assert!(paint_runs.is_empty())", tests)

    def test_renderer_distinguishes_layout_mismatch_from_missing_artifact(self) -> None:
        source = RESOLVED_LAYOUT.read_text(encoding="utf-8")
        render = RICH_RENDER.read_text(encoding="utf-8")
        text_batches = TEXT_BATCHES.read_text(encoding="utf-8")
        screen_render = SCREEN_RENDER.read_text(encoding="utf-8")

        self.assertIn("enum RichTextGlyphArtifactRouteBatch", source)
        self.assertIn("PaintLayoutMismatch", source)
        self.assertIn("if !paint_runs_match_layout(layout, paint_runs)", source)
        self.assertIn("enum TextPlanOutcome", text_batches)
        self.assertIn("TextPlanOutcome::Rejected", render)
        self.assertIn("RichTextGlyphArtifactRouteBatch::PaintLayoutMismatch", render)
        self.assertIn("ResolvedGlyphArtifactRouteReceipt::Rejected(", render)
        self.assertIn("ResolvedGlyphArtifactRejection::Incomplete", render)
        self.assertIn("plan.vertices.truncate(text_decoration_vertex_start)", screen_render)
        self.assertIn("if !text_projection_rejected", screen_render)

    def test_mismatch_is_handled_without_whole_line_fallback(self) -> None:
        tests = RICH_ROUTE_TESTS.read_text(encoding="utf-8")
        render = RICH_RENDER.read_text(encoding="utf-8")

        self.assertIn(
            "rich_paint_run_projection_mismatch_fails_closed_without_line_fallback",
            tests,
        )
        self.assertIn(
            "rich_paint_run_projection_mismatch_fails_closed_when_glyph_artifact_is_missing",
            tests,
        )
        self.assertIn("assert!(plan.native_texts.is_empty())", tests)
        self.assertIn("assert!(plan.images.is_empty())", tests)
        self.assertIn("assert_eq!(plan.vertices.len(), 6)", tests)
        self.assertIn(
            "rich_paint_run_non_finite_geometry_rejects_before_partial_materialization",
            tests,
        )
        self.assertIn(
            "rich_paint_run_non_positive_metrics_reject_before_partial_materialization",
            tests,
        )
        self.assertIn("rich_text_paint_run_geometry_is_valid", render)
        self.assertIn("frame.x.is_finite()", render)
        self.assertIn("run.font_size > 0.0", render)
        self.assertIn("run.line_height > 0.0", render)
        self.assertIn("assert!(plan.post_text_draws.is_empty())", tests)
        self.assertIn("incomplete_artifact_count", tests)

    def test_unrecoverable_rich_run_rejects_before_any_command_materialization(self) -> None:
        tests = RICH_ROUTE_TESTS.read_text(encoding="utf-8")
        render = RICH_RENDER.read_text(encoding="utf-8")

        self.assertIn(
            "rich_unrecoverable_run_rejects_before_source_fallback_partial_materialization",
            tests,
        )
        self.assertIn("preflight_rich_text_run_admissions", render)
        preflight = render.index("preflight_rich_text_run_admissions")
        inline_materialization = render.index("plan_inline_run(", preflight)
        text_materialization = render.index("push_text_batch(", preflight)
        self.assertLess(preflight, inline_materialization)
        self.assertLess(preflight, text_materialization)
        self.assertIn("ResolvedGlyphArtifactRouteReceipt::Rejected(rejection)", render)

    def test_rich_presentation_reuses_layout_style_admission(self) -> None:
        tests = RICH_PROJECTION_TESTS.read_text(encoding="utf-8")
        render = RICH_RENDER.read_text(encoding="utf-8")

        self.assertIn(
            "rich_presentation_reuses_layout_style_admission_for_invalid_overrides",
            tests,
        )
        start = render.index("pub(super) fn prepare_text_run")
        end = render.index("pub(super) fn decorations_for_rich_run", start)
        presentation = render[start:end]
        self.assertIn(
            ".filter(|size| size.is_finite() && *size > 0.0)", presentation
        )
        self.assertIn(".filter(|family| !family.is_empty())", presentation)

    def test_empty_failed_rich_projection_does_not_fall_through_to_plain_batches(
        self,
    ) -> None:
        tests = RICH_PROJECTION_TESTS.read_text(encoding="utf-8")
        render = RICH_RENDER.read_text(encoding="utf-8")

        self.assertIn(
            "empty_failed_rich_paint_projection_cannot_fall_through_to_plain_layout_batches",
            tests,
        )
        self.assertIn(
            "text_paint.runs.is_empty() && command.text_layout.is_none()", render
        )


if __name__ == "__main__":
    unittest.main()
