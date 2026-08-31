from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMMAND = ROOT / "zircon_runtime_interface/src/ui/surface/render/command.rs"
RENDER = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
PAINT_PROJECTION = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/paint_projection.rs"
)


class RuntimeUiPaintElementScratchPerformanceContractTests(unittest.TestCase):
    def test_render_command_can_fill_reusable_transient_paint_scratch(self):
        source = COMMAND.read_text(encoding="utf-8")
        self.assertIn("fill_transient_paint_elements", source)
        self.assertIn("elements: &mut Vec<UiPaintElement>", source)

    def test_ui_planner_reuses_one_scratch_buffer_per_frame(self):
        source = RENDER.read_text(encoding="utf-8")
        projection = PAINT_PROJECTION.read_text(encoding="utf-8")
        self.assertIn("let mut paint_elements = Vec::new();", source)
        self.assertIn("project_transient_paint_elements(", source)
        self.assertIn("command.fill_transient_paint_elements", projection)
        self.assertNotIn("to_transient_paint_elements(0)", source)


if __name__ == "__main__":
    unittest.main()
