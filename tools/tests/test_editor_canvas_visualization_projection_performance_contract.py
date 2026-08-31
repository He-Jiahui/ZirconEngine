from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HEATMAP_ROOT = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_weight_heatmap.rs"
)
HEATMAP_GEOMETRY = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_weight_heatmap/geometry.rs"
)


class EditorCanvasVisualizationProjectionPerformanceContractTests(unittest.TestCase):
    def test_collapsed_heatmap_returns_before_text_measurement_and_painter_dispatch(self) -> None:
        source = HEATMAP_ROOT.read_text(encoding="utf-8")
        function = source.split("fn push_weight_heatmap_commands", 1)[1]

        extent_guard = function.index("if !has_paintable_weight_heatmap_extent(rect)")
        early_return = function.index("return true;", extent_guard)
        generation = function.index("let generation =")
        label_measurement = function.index("legend_label_width(generation)")
        field_dispatch = function.index("push_heatmap_field(")

        self.assertLess(extent_guard, early_return)
        self.assertLess(early_return, generation)
        self.assertLess(early_return, label_measurement)
        self.assertLess(early_return, field_dispatch)

    def test_extent_gate_requires_finite_positive_frame_dimensions(self) -> None:
        source = HEATMAP_GEOMETRY.read_text(encoding="utf-8")
        function = source.split("fn has_paintable_weight_heatmap_extent", 1)[1]

        for check in (
            "frame.x.is_finite()",
            "frame.y.is_finite()",
            "frame.width.is_finite()",
            "frame.height.is_finite()",
            "frame.width > 0.0",
            "frame.height > 0.0",
        ):
            self.assertIn(check, function)


if __name__ == "__main__":
    unittest.main()
