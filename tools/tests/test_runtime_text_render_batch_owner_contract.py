import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RENDER_ROOT = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
TEXT_BATCHES = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_batches.rs"
)
RESOLVED_LAYOUT = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs"
)
RICH_TEXT = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs"
)
GLYPH_ARTIFACT_TESTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/glyph_artifacts.rs"
)


class RuntimeTextRenderBatchOwnerContractTests(unittest.TestCase):
    def test_text_batch_planning_has_one_child_owner(self) -> None:
        root = RENDER_ROOT.read_text(encoding="utf-8")
        batches = TEXT_BATCHES.read_text(encoding="utf-8")
        resolved = RESOLVED_LAYOUT.read_text(encoding="utf-8")
        rich = RICH_TEXT.read_text(encoding="utf-8")

        self.assertIn("mod text_batches;", root)
        self.assertRegex(
            root,
            r"use text_batches::\{(?:TextPlanOutcome, push_text_batches|push_text_batches, TextPlanOutcome)\};",
        )
        self.assertNotIn("fn push_text_batches(", root)
        self.assertNotIn("fn push_text_batch(", root)
        self.assertNotIn("struct ScreenSpaceUiTextBatch {", root)
        self.assertIn("pub(super) use text_batches::{ScreenSpaceUiTextBatch", root)
        self.assertIn("pub(super) fn push_text_batches(", batches)
        self.assertIn("pub(super) fn push_text_batch(", batches)
        self.assertIn("pub(super) struct ScreenSpaceUiTextBatch {", batches)
        self.assertIn("pub(super) enum TextPlanOutcome {", batches)
        self.assertIn("impl ScreenSpaceUiTextBatch", batches)
        self.assertIn(
            "resolved_layout::ResolvedGlyphArtifactRouteReceipt::Rejected(_)",
            batches,
        )
        self.assertIn(
            "TextPlanOutcome::Rejected",
            batches,
        )
        self.assertNotIn("RichTextPlanOutcome", batches)
        self.assertNotIn("RichTextPlanOutcome", rich)
        self.assertIn("super::text_batches::push_text_batch", resolved)
        self.assertRegex(
            rich,
            r"super::text_batches::\{(?:TextPlanOutcome, push_text_batch|push_text_batch, TextPlanOutcome)\}",
        )

    def test_render_batch_owners_stay_below_the_repository_warning_line(self) -> None:
        self.assertLess(len(RENDER_ROOT.read_text(encoding="utf-8").splitlines()), 800)
        self.assertLess(len(TEXT_BATCHES.read_text(encoding="utf-8").splitlines()), 800)

    def test_plain_rejection_regression_suppresses_text_owned_decorations(self) -> None:
        tests = GLYPH_ARTIFACT_TESTS.read_text(encoding="utf-8")
        resolved = RESOLVED_LAYOUT.read_text(encoding="utf-8")
        batches = TEXT_BATCHES.read_text(encoding="utf-8")

        self.assertIn(
            "screen_space_ui_plan_rejects_visual_bidi_without_artifact_and_suppresses_decorations",
            tests,
        )
        self.assertIn("editable: Some(UiEditableTextState", tests)
        self.assertIn("selection: Some(UiTextSelection", tests)
        self.assertIn("assert!(plan.vertices.is_empty())", tests)
        self.assertIn("assert!(plan.post_text_draws.is_empty())", tests)
        self.assertIn(
            "screen_space_ui_plan_rejects_non_finite_plain_layout_geometry_before_source_fallback",
            tests,
        )
        self.assertIn("f32::NAN", tests)
        self.assertIn(
            "screen_space_ui_plan_rejects_non_finite_command_frame_before_fallback",
            tests,
        )
        self.assertIn("text_batch_frame_is_valid", batches)
        self.assertIn("frame.x.is_finite()", batches)
        self.assertIn("resolved_text_layout_batch_geometry_is_valid", resolved)
        self.assertIn("advance.is_finite() && *advance >= 0.0", resolved)
        self.assertIn("if layout.lines.is_empty()", batches)
        self.assertIn(
            "screen_space_ui_plan_does_not_re_shape_nonempty_text_with_safe_empty_layout",
            tests,
        )


if __name__ == "__main__":
    unittest.main()
