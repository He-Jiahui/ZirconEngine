import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GEOMETRY = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/geometry.rs"
)
RENDER = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
SHADER = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/screen_space_ui.wgsl"
)


class RuntimeUiLegacyScreenSpaceAnalyticCoverageContractTests(unittest.TestCase):
    def test_vertex_abi_carries_shape_data_for_analytic_coverage(self):
        source = GEOMETRY.read_text(encoding="utf-8")
        for field in (
            "local_position",
            "half_extent",
            "corner_radius",
            "border_width",
        ):
            self.assertIn(f"pub(super) {field}:", source)
        self.assertIn("const ATTRIBUTES: [wgpu::VertexAttribute; 6]", source)
        self.assertIn("pub(super) fn coverage_frame", source)

    def test_batch_scissor_uses_the_same_coverage_frame_as_geometry(self):
        source = RENDER.read_text(encoding="utf-8")
        self.assertIn("let scissor_frame = coverage_frame(", source)
        self.assertIn("clipped_scissor(scissor_frame", source)
        self.assertIn("push_rect_with_radius(", source)
        self.assertIn("push_border_with_radius(", source)

    def test_shader_uses_forward_smoothstep_and_derivative_width(self):
        source = SHADER.read_text(encoding="utf-8")
        self.assertIn("rounded_box_distance", source)
        self.assertIn("fn rounded_box_coverage", source)
        self.assertIn("let sample_offsets = array<vec2<f32>, 16>", source)
        self.assertIn("let subpixel_filter_scale = 0.25", source)
        self.assertIn("sample_index < 16u", source)
        self.assertIn("coverage += sample_coverage", source)
        self.assertIn("return coverage * 0.0625", source)
        self.assertIn("fwidth(outer_distance)", source)
        self.assertIn("fwidth(inner_distance)", source)
        self.assertIn("1.0 - smoothstep(", source)
        self.assertIn("input.color.a * coverage", source)


if __name__ == "__main__":
    unittest.main()
