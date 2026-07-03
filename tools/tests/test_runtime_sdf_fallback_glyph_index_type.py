import unittest
from pathlib import Path


class RuntimeSdfFallbackGlyphIndexTypeTests(unittest.TestCase):
    def test_sdf_fallback_glyph_index_has_explicit_usize_type(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        source = (
            repo_root
            / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("let mut glyph_index: usize = 0;", source)
        self.assertNotIn("let mut glyph_index = 0;", source)


if __name__ == "__main__":
    unittest.main()
