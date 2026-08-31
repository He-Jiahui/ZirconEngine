from pathlib import Path
import unittest

from tools.runtime_ui_transient_paint_metadata_pressure import run


ROOT = Path(__file__).resolve().parents[2]
COMMAND_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/command.rs"
PRODUCT_RENDER_SOURCE = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
)
PAINT_PROJECTION_SOURCE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/paint_projection.rs"
)
TEXT_BATCHES_SOURCE = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_batches.rs"
)


class RuntimeUiTransientPaintMetadataContractTests(unittest.TestCase):
    def test_product_path_uses_transient_elements_and_defers_generation_to_text(self) -> None:
        command = COMMAND_SOURCE.read_text(encoding="utf-8")
        render = PRODUCT_RENDER_SOURCE.read_text(encoding="utf-8")
        projection = PAINT_PROJECTION_SOURCE.read_text(encoding="utf-8")
        text_batches = TEXT_BATCHES_SOURCE.read_text(encoding="utf-8")

        self.assertIn("pub fn fill_transient_paint_elements", command)
        self.assertIn("project_transient_paint_elements(", render)
        self.assertIn("command.fill_transient_paint_elements", projection)
        self.assertNotIn("command.fill_paint_elements(0, metrics, paint_elements)", render)
        self.assertIn("command_generation: command.cache_generation()", text_batches)

    def test_pressure_model_counts_only_text_generation_after_transient_switch(self) -> None:
        result = run(command_count=32_768, text_command_count=8_192)

        self.assertEqual(
            result["legacy_cached_paint_path"]["stable_json_generation_calls"],
            32_768,
        )
        self.assertEqual(
            result["transient_product_path"]["stable_json_generation_calls"],
            8_192,
        )
        self.assertEqual(
            result["delta"]["avoided_stable_json_generation_calls"],
            24_576,
        )
        self.assertEqual(
            result["delta"]["avoided_debug_label_format_calls"],
            32_768,
        )

    def test_pressure_model_rejects_invalid_dimensions(self) -> None:
        with self.assertRaises(ValueError):
            run(command_count=0)
        with self.assertRaises(ValueError):
            run(text_command_count=-1)
        with self.assertRaises(ValueError):
            run(command_count=1, text_command_count=2)


if __name__ == "__main__":
    unittest.main()
