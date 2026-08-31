import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COPY = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/"
    "softbuffer/surface_io/copy.rs"
)
PLAN = REPO_ROOT / (
    "docs/plans/zircon_editor/editor_ui/12/"
    "2026-08-15-m6-device-pixel-aa-and-local-supersampling.md"
)


class EditorUiPhysicalSamplingContractTests(unittest.TestCase):
    def test_native_resize_does_not_use_nearest_neighbor_source_selection(self):
        source = COPY.read_text(encoding="utf-8")

        self.assertNotIn("EndpointNearestAxis", source)
        self.assertNotIn("Vec<AxisSample>", source)
        self.assertIn("axis_sample", source)
        self.assertIn("bilinear_channel", source)
        self.assertIn("srgb_byte_to_linear", source)
        self.assertIn("linear_to_srgb_byte", source)
        self.assertNotIn("powf", source)
        self.assertIn("copy_rgba_to_softbuffer(source, buffer, None, target_size)", source)

    def test_m6_plan_records_bilinear_resize_sampling(self):
        plan = PLAN.read_text(encoding="utf-8")

        self.assertIn("physical-pixel-center bilinear sampling", plan)
        self.assertIn("linear-light color interpolation", plan)
        self.assertNotIn("software scaled images use nearest-neighbor", plan)


if __name__ == "__main__":
    unittest.main()
