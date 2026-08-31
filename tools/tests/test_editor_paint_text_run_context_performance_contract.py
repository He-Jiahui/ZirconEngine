from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
GLYPHS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs"
)
RASTER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs"
)


class EditorPaintTextRunContextPerformanceContract(unittest.TestCase):
    def test_glyph_run_captures_smoothing_and_optional_host_font_once(self) -> None:
        source = GLYPHS.read_text(encoding="utf-8")
        run = source.split("pub(super) fn draw_layout_glyphs", 1)[1]
        run = run.split("fn draw_layout_glyph", 1)[0]

        self.assertEqual(run.count("current_host_text_preferences()"), 1)
        self.assertEqual(source.count("host_font_snapshot_for_face(font_face)"), 1)
        self.assertIn("let host_font = OnceCell::new()", run)
        self.assertIn("&host_font", run)
        self.assertNotIn("glyphs.iter().any", run)

        fallback = source.split("fn draw_layout_glyph", 1)[1]
        self.assertIn("host_font.get_or_init", fallback)

    def test_raster_cache_core_uses_caller_supplied_smoothing(self) -> None:
        source = RASTER.read_text(encoding="utf-8")
        core = source.split("fn rasterize_cached_font_glyph", 1)[1]
        core = core.split("fn rasterize_swash_glyph", 1)[0]

        self.assertIn("text_smoothing: HostTextSmoothing", core)
        self.assertIn("text_smoothing,", core)
        self.assertNotIn("current_host_text_preferences()", core)
        self.assertIn("fn rasterize_cached_host_glyph", source)

    def test_swash_same_format_outputs_reuse_the_owned_buffer(self) -> None:
        source = RASTER.read_text(encoding="utf-8")
        bitmap = source.split("fn swash_bitmap", 1)[1]
        bitmap = bitmap.split("fn bitmap_has_visible_ink", 1)[0]

        self.assertIn("mut data: Vec<u8>", bitmap)
        self.assertGreaterEqual(bitmap.count("data.truncate("), 3)
        self.assertNotIn("data.into_iter().take", bitmap)


if __name__ == "__main__":
    unittest.main()
