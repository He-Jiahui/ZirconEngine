import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
BITMAP = REPO_ROOT / "zircon_runtime/src/text/raster/swash/bitmap.rs"
RASTERIZER = REPO_ROOT / "zircon_runtime/src/text/raster/swash/rasterizer.rs"
TESTS = REPO_ROOT / "zircon_runtime/src/text/raster/swash/tests.rs"


class RuntimeTextSwashColorAlphaContractTests(unittest.TestCase):
    def test_color_outline_is_normalized_to_the_straight_alpha_atlas_contract(self) -> None:
        bitmap = BITMAP.read_text(encoding="utf-8")
        rasterizer = RASTERIZER.read_text(encoding="utf-8")

        self.assertIn("GlyphColorBitmapAlphaMode", bitmap)
        self.assertIn("unpremultiply_rgba8_in_place", bitmap)
        self.assertIn("SwashSource::ColorOutline", rasterizer)
        self.assertIn("GlyphColorBitmapAlphaMode::Premultiplied", rasterizer)

    def test_rust_regressions_distinguish_outline_and_bitmap_sources(self) -> None:
        tests = TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "text_raster_swash_unpremultiplies_color_outline_for_straight_alpha_atlas",
            tests,
        )
        self.assertIn(
            "text_raster_swash_preserves_straight_color_bitmap_pixels",
            tests,
        )


if __name__ == "__main__":
    unittest.main()
