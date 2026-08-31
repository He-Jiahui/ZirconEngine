from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ATLAS = ROOT / "zircon_runtime/src/ui/icon_atlas/atlas.rs"
SVG = ROOT / "zircon_runtime/src/ui/icon_atlas/svg.rs"


class RuntimeUiSvgDocumentCacheContractTests(unittest.TestCase):
    def test_atlas_plan_uses_bounded_cached_svg_parser(self):
        atlas_source = ATLAS.read_text(encoding="utf-8")
        svg_source = SVG.read_text(encoding="utf-8")
        self.assertIn("parse_ui_svg_icon_cached", atlas_source)
        self.assertIn("const SVG_DOCUMENT_CACHE_CAPACITY: usize = 512;", svg_source)
        self.assertIn("OnceLock<Mutex<SvgDocumentCache>>", svg_source)
        self.assertIn("HashMap<String, UiSvgIconDocument>", svg_source)
        self.assertIn("VecDeque<String>", svg_source)

    def test_cache_does_not_replace_parser_contract(self):
        svg_source = SVG.read_text(encoding="utf-8")
        self.assertIn("let document = parse_ui_svg_icon(source)?;", svg_source)
        self.assertIn(".insert(source, document.clone())", svg_source)
        self.assertNotIn("unwrap_or_default()", svg_source)


if __name__ == "__main__":
    unittest.main()
