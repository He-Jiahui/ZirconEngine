from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
GLYPHS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs"
)
RASTER = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs"
)
RUNTIME_REQUEST = ROOT / (
    "zircon_runtime/src/core/framework/text/glyph_raster/request.rs"
)
RUNTIME_SERVICE = ROOT / (
    "zircon_runtime/src/text/raster/service/glyph_raster_service.rs"
)
ROW = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs"
)


class EditorPaintTextRunContextPerformanceContract(unittest.TestCase):
    def test_glyph_run_captures_smoothing_once_and_uses_exact_artifact_faces(self) -> None:
        source = GLYPHS.read_text(encoding="utf-8")
        run = source.split("pub(super) fn draw_layout_glyphs", 1)[1]
        run = run.split("fn draw_layout_glyph", 1)[0]

        self.assertEqual(run.count("current_host_text_preferences()"), 1)
        self.assertIn("artifact_raster_faces.get(glyph.raster_face_index)", run)
        self.assertNotIn("OnceCell", source)
        self.assertNotIn("host_font", source)
        self.assertNotIn("glyphs.iter().any", run)

        draw = source.split("fn draw_layout_glyph", 1)[1]
        self.assertIn("face.rasterize_glyph(request)", draw)

    def test_editor_has_no_private_raster_backend_or_cache(self) -> None:
        source = GLYPHS.read_text(encoding="utf-8")

        self.assertFalse(RASTER.exists())
        self.assertNotIn("HashMap", source)
        self.assertNotIn("swash", source.lower())
        self.assertNotIn("fontdue", source.lower())
        self.assertIn("TextGlyphRasterRequest::new", source)
        self.assertIn("with_subpixel_position", source)

    def test_runtime_owns_phase_backend_and_shared_receipt_bitmap(self) -> None:
        request = RUNTIME_REQUEST.read_text(encoding="utf-8")
        service = RUNTIME_SERVICE.read_text(encoding="utf-8")
        row = ROW.read_text(encoding="utf-8")

        self.assertIn("HORIZONTAL_PHASE_COUNT: u8 = 3", request)
        self.assertIn("VERTICAL_PHASE_COUNT: u8 = 4", request)
        self.assertIn("bitmap: Arc::from(bitmap.data)", service)
        self.assertIn("TextGlyphBitmapFormat::AlphaMask", row)
        self.assertIn("TextGlyphBitmapFormat::SubpixelMask", row)
        self.assertIn("TextGlyphBitmapFormat::ColorRgba", row)


if __name__ == "__main__":
    unittest.main()
