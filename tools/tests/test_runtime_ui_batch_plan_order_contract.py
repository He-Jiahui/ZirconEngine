from pathlib import Path
import unittest

from tools.runtime_ui_batch_plan_order_pressure import run


ROOT = Path(__file__).resolve().parents[2]
BATCH_PLAN_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/batch/plan.rs"
)
PRODUCT_PLAN_CACHE_SOURCE = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs"
)
PRODUCT_RENDER_SOURCE = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
)


class RuntimeUiBatchPlanOrderContractTests(unittest.TestCase):
    def test_batch_plan_validates_order_before_sorting(self) -> None:
        source = BATCH_PLAN_SOURCE.read_text(encoding="utf-8")
        self.assertIn("elements.windows(2).all", source)
        self.assertIn("indices.sort_by_key", source)

    def test_pressure_model_removes_sort_for_ordered_extracts(self) -> None:
        result = run(element_count=32_768, ordered_frame_count=4096)
        self.assertEqual(
            result["ordered_input_fast_path"]["sort_invocations"], 0
        )
        self.assertEqual(
            result["delta"]["avoided_sort_invocations"], 4096
        )
        self.assertEqual(
            result["legacy_unconditional_sort"]["ordered_input_visits"],
            result["ordered_input_fast_path"]["ordered_input_visits"],
        )

    def test_pressure_model_rejects_invalid_dimensions(self) -> None:
        with self.assertRaises(ValueError):
            run(element_count=0)
        with self.assertRaises(ValueError):
            run(ordered_frame_count=0)

    def test_product_renderer_keeps_batch_helper_out_of_gpu_hot_path(self) -> None:
        plan_cache = PRODUCT_PLAN_CACHE_SOURCE.read_text(encoding="utf-8")
        render = PRODUCT_RENDER_SOURCE.read_text(encoding="utf-8")

        self.assertIn("ScreenSpaceUiPlanCache", plan_cache)
        self.assertIn("&mut self.paint_elements", plan_cache)
        self.assertIn("&mut paint_elements", render)
        for source in (plan_cache, render):
            self.assertNotIn("UiBatchPlan::from_paint_elements", source)


if __name__ == "__main__":
    unittest.main()
