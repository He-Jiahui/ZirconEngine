from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TEMPLATE_SURFACE = ROOT / "zircon_editor/src/ui/workbench/reference/template_surface.rs"
REFERENCE_TESTS = ROOT / "zircon_editor/src/tests/workbench/reference_surface.rs"


class EditorUiFrameExtractionCacheContractTests(unittest.TestCase):
    def test_refresh_selects_geometry_cache_by_layout_domain(self):
        source = TEMPLATE_SURFACE.read_text(encoding="utf-8")
        self.assertIn("refresh_frames: bool", source)
        self.assertIn(
            "self.refresh_projection(runtime, &workset, report.layout_recomputed)",
            source,
        )
        self.assertIn("ui.workbench_template.frames_extract_count", source)
        self.assertIn("ui.workbench_template.frames_extract_skip_count", source)
        self.assertIn("frames_extract_count: u64", source)
        self.assertIn("frames_extract_skip_count: u64", source)

        refresh_body = source.split("fn refresh_projection(", 1)[1]
        extraction = "EditorWorkbenchTemplateFrames::from_surface(&self.surface, &self.control_nodes)?"
        self.assertEqual(refresh_body.count(extraction), 1)
        self.assertLess(refresh_body.index("if refresh_frames"), refresh_body.index(extraction))

    def test_regression_asserts_frame_extraction_is_skipped(self):
        source = REFERENCE_TESTS.read_text(encoding="utf-8")
        self.assertIn("frames_extract_count()", source)
        self.assertIn("frames_extract_skip_count()", source)
        self.assertIn("frames_extract_skip_count + 1", source)


if __name__ == "__main__":
    unittest.main()
