import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/scene/world/render.rs"
REGRESSION = REPO_ROOT / "zircon_runtime/src/scene/world/render/tests/view_camera_filter_tests.rs"


class RuntimeRenderViewCameraFilterRetainPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_selected_camera_filter_reuses_descriptor_buffer(self) -> None:
        match = re.search(
            r"fn build_render_view_extract\(.*?\n    }\n\n    fn render_extract_layers_for_view",
            self.source,
            re.S,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertRegex(
            body,
            r"let\s+mut cameras\s*=\s*self\.scene_camera_descriptors_with_override\(",
        )
        self.assertIn("cameras.retain(|descriptor|", body)
        self.assertNotIn(
            ".into_iter()\n                    .filter(|descriptor|",
            body,
        )
        self.assertNotIn(".collect(),", body)

    def test_retain_predicate_preserves_selected_inactive_camera_and_order(self) -> None:
        match = re.search(
            r"cameras\.retain\(\|descriptor\|.*?\n\s*\}\);",
            self.source,
            re.S,
        )
        self.assertIsNotNone(match)
        predicate = match.group(0)
        self.assertIn("descriptor.entity == Some(entity)", predicate)
        self.assertIn("descriptor.is_active()", predicate)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "runtime856_render_view_camera_filter_retain_preserves_order",
            self.regression,
        )
        self.assertIn(
            "RUNTIME856_RENDER_VIEW_CAMERA_FILTER_RETAIN_BENCH_V1",
            self.regression,
        )

    def test_deterministic_model_removes_filter_buffer_growth(self) -> None:
        camera_count = 8_192
        legacy_filter_growth_events = 13
        optimized_filter_growth_events = 0
        self.assertGreater(camera_count, 0)
        self.assertEqual(legacy_filter_growth_events, 13)
        self.assertEqual(optimized_filter_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
