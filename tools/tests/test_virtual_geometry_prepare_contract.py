import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
PREPARE = (
    ROOT
    / "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/prepare_frame/prepare_visible_clusters.rs"
)
OUTPUTS = (
    ROOT
    / "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_plugin_renderer_outputs.rs"
)


class VirtualGeometryPrepareContractTests(unittest.TestCase):
    def test_compacted_segments_have_an_explicit_element_type(self):
        source = PREPARE.read_text(encoding="utf-8")

        self.assertIn(
            "let mut cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment> =",
            source,
        )

    def test_renderer_output_uses_the_public_gpu_recording_context(self):
        source = OUTPUTS.read_text(encoding="utf-8")

        self.assertIn("let gpu = context.gpu_recording_context();", source)
        self.assertIn("gpu.device\n            .create_buffer_init", source)
        self.assertIn("contents: bytemuck::cast_slice(&page_request_ids),", source)
        self.assertNotIn("context\n        .device\n        .create_buffer_init", source)


if __name__ == "__main__":
    unittest.main()
