import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RENDER_PLAN = REPO_ROOT / "zircon_runtime/src/text/atlas/render_plan.rs"
RENDER_PLAN_TESTS = REPO_ROOT / "zircon_runtime/src/text/atlas/render_plan/tests.rs"


class RuntimeTextAtlasUvAdmissionContractTests(unittest.TestCase):
    def test_content_uv_projection_checks_page_bounds_without_clamping(self) -> None:
        source = RENDER_PLAN.read_text(encoding="utf-8")

        self.assertIn("atlas_content_rect", source)
        self.assertIn("checked_add", source)
        self.assertIn("slot_right > atlas_size.x", source)
        self.assertIn("slot_bottom > atlas_size.y", source)
        self.assertIn("content_right > atlas_size.x", source)
        self.assertIn("content_bottom > atlas_size.y", source)
        self.assertNotIn("content_size.x.min(atlas_rect.width)", source)
        self.assertNotIn("content_size.y.min(atlas_rect.height)", source)

    def test_screen_rect_projection_rejects_non_finite_derived_edges(self) -> None:
        source = RENDER_PLAN.read_text(encoding="utf-8")

        self.assertIn("self.right().is_finite()", source)
        self.assertIn("self.bottom().is_finite()", source)

    def test_gpu_color_projection_normalizes_every_channel(self) -> None:
        source = RENDER_PLAN.read_text(encoding="utf-8")
        tests = RENDER_PLAN_TESTS.read_text(encoding="utf-8")

        self.assertIn("fn normalized_gpu_color", source)
        self.assertIn("normalized_gpu_color(glyph.foreground_color)", source)
        self.assertIn(
            "render_text_atlas_draw_plan_normalizes_all_gpu_color_channels",
            tests,
        )

    def test_rust_regressions_cover_slot_and_page_disagreement(self) -> None:
        source = RENDER_PLAN_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "render_text_atlas_draw_plan_rejects_content_larger_than_slot",
            source,
        )
        self.assertIn(
            "render_text_atlas_draw_plan_rejects_content_outside_page",
            source,
        )
        self.assertIn(
            "render_text_atlas_draw_plan_rejects_slot_outside_page",
            source,
        )
        self.assertIn(
            "render_text_atlas_draw_plan_rejects_overflowed_screen_extents",
            source,
        )


if __name__ == "__main__":
    unittest.main()
