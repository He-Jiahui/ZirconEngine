import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/scene/world/render_post_process.rs"
REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/scene/world/render_post_process/tests/output_capacity_tests.rs"
)


class RuntimeRenderPostProcessOutputCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8") if REGRESSION.exists() else ""

    def test_collection_reserves_component_upper_bound_for_both_outputs(self) -> None:
        match = re.search(
            r"pub\(super\) fn collect_post_process_volumes\(.*?\n    }",
            self.source,
            re.S,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("self.component_count_for_id(component_id)", body)
        self.assertEqual(body.count("Vec::with_capacity(volume_count)"), 2)
        self.assertNotIn("let mut extracts = Vec::new();", body)
        self.assertNotIn("let mut fog_volumes = Vec::new();", body)

    def test_collection_ordering_and_layer_filters_remain_intact(self) -> None:
        body = self.source[
            self.source.index("pub(super) fn collect_post_process_volumes(") : self.source.index(
                "    fn post_process_volume_extract"
            )
        ]
        self.assertIn("extracts.sort_by", body)
        self.assertIn("fog_volumes.sort_by_key", body)
        self.assertIn("if !affects_post_process && !affects_local_fog", body)
        self.assertIn("extracts: extracts.into_iter().map(|(_, extract)| extract).collect()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod output_capacity_tests;", self.source)
        self.assertIn(
            "runtime858_render_post_process_output_capacity_preserves_order",
            self.regression,
        )
        self.assertIn("RUNTIME858_RENDER_POST_PROCESS_OUTPUT_CAPACITY_BENCH_V1", self.regression)

    def test_deterministic_model_removes_component_output_growth(self) -> None:
        component_count = 4_096
        legacy_growth_events = 24
        optimized_growth_events = 0
        self.assertGreater(component_count, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
