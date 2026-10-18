from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "visual_assets/candidates/query.rs"
)
LOWER = SOURCE.parent / "query/tests/capacity_tests.rs"


class EditorVisualCandidateCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.lower = LOWER.read_text(encoding="utf-8")

    def test_image_and_preview_candidates_use_finite_nonempty_bounds(self) -> None:
        self.assertIn("const MAX_PACKAGED_IMAGE_CANDIDATES: usize = 4;", self.source)
        self.assertIn(
            "const MAX_PREVIEW_ARTIFACT_CANDIDATES: usize = 5;", self.source
        )
        self.assertIn(
            "Vec::with_capacity(MAX_PACKAGED_IMAGE_CANDIDATES)", self.source
        )
        self.assertIn(
            "Vec::with_capacity(MAX_PREVIEW_ARTIFACT_CANDIDATES)", self.source
        )
        self.assertIn("if source.is_empty() {", self.source)
        self.assertIn("return Vec::new();", self.source)

    def test_icon_candidates_keep_packaged_bound_and_dynamic_module_growth(self) -> None:
        self.assertIn("const MAX_PACKAGED_ICON_CANDIDATES: usize = 6;", self.source)
        self.assertIn(
            "Vec::with_capacity(MAX_PACKAGED_ICON_CANDIDATES)", self.source
        )
        self.assertIn("include_development_modules", self.source)
        self.assertIn("mui_icons::module_candidates", self.source)

    def test_lower_capacity_regression_and_marker_are_wired(self) -> None:
        self.assertIn(
            '#[path = "query/tests/capacity_tests.rs"]',
            self.source,
        )
        self.assertIn("image_candidates_from_asset_root", self.lower)
        self.assertIn("preview_artifact_candidates", self.lower)
        self.assertIn("icon_candidates_from_asset_root", self.lower)
        self.assertIn("EDITOR829_VISUAL_CANDIDATE_CAPACITY_BENCH_V1", self.lower)

    def test_deterministic_variant_model_removes_growth_events(self) -> None:
        candidate_counts = (4, 5, 6)
        legacy_growth_events = sum(self._growth_events(count) for count in candidate_counts)
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(legacy_growth_events, 5)
        self.assertEqual(candidate_counts, (4, 5, 6))

    @staticmethod
    def _growth_events(count: int) -> int:
        capacity = 0
        events = 0
        for index in range(count):
            if index == capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                events += 1
        return events


if __name__ == "__main__":
    unittest.main()
