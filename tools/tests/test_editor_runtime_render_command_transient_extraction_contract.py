from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMMAND = ROOT / "zircon_runtime_interface/src/ui/surface/render/command.rs"
HOST_CONVERSION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "render_command_conversion/commands/command.rs"
)
RUNTIME_RENDERER = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
)


class EditorRuntimeRenderCommandTransientExtractionContractTests(unittest.TestCase):
    def test_transient_extraction_omits_cache_and_debug_metadata(self) -> None:
        source = COMMAND.read_text(encoding="utf-8")
        function = source.split("pub fn to_transient_paint_elements_with_metrics", 1)[1]
        function = function.split("fn build_paint_elements_with_metrics", 1)[0]

        self.assertIn("PaintElementMetadata::Transient", function)
        transient_metadata = source.split("PaintElementMetadata::Transient =>", 1)[1]
        transient_metadata = transient_metadata.split("}", 1)[0]
        self.assertIn("(None, None)", transient_metadata)

    def test_editor_host_uses_transient_extraction(self) -> None:
        source = HOST_CONVERSION.read_text(encoding="utf-8")
        function = source.split("fn push_runtime_command", 1)[1]

        self.assertIn("command.to_transient_paint_elements(0)", function)
        self.assertNotIn("command.to_paint_elements(0)", function)

    def test_runtime_gpu_planner_uses_transient_extraction(self) -> None:
        source = RUNTIME_RENDERER.read_text(encoding="utf-8")
        function = source.split(
            "fn plan_screen_space_ui_batches_with_framebuffer_background", 1
        )[1]
        function = function.split("fn framebuffer_background_color", 1)[0]

        self.assertIn("command.to_transient_paint_elements(0)", function)
        self.assertNotIn("command.to_paint_elements(0)", function)


if __name__ == "__main__":
    unittest.main()
