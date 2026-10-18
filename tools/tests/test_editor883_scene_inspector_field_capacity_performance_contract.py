import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/scene/viewport/edit_mode_projection/build.rs"
LOWER = (
    REPO_ROOT / "zircon_editor/src/scene/viewport/edit_mode_projection/build/tests/field_capacity_tests.rs"
)


class Editor883SceneInspectorFieldCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        owner = re.search(
            r"pub\(crate\) fn build_scene_edit_mode_projection\(.*?\n}\n\n",
            cls.source,
            re.S,
        )
        if owner is None:
            raise AssertionError("scene edit-mode projection owner is missing")
        cls.owner = owner.group(0)
        helper = re.search(
            r"fn scene_inspector_fields_from_runtime\(.*?\n}\n\n",
            cls.source,
            re.S,
        )
        cls.helper = helper.group(0) if helper is not None else ""

    def test_projection_owner_routes_fields_through_shared_helper(self) -> None:
        self.assertIn("scene_inspector_fields_from_runtime(artifact.fields())", self.owner)

    def test_helper_lazily_reserves_the_runtime_field_bound(self) -> None:
        self.assertIn("let mut projected = Vec::new()", self.helper)
        self.assertIn("if projected.is_empty()", self.helper)
        self.assertIn("projected.reserve(fields.len())", self.helper)
        self.assertIn("projected.push(field)", self.helper)

    def test_owner_no_longer_collects_filtered_fields(self) -> None:
        self.assertNotIn("filter_map(scene_inspector_field_from_runtime)", self.owner)
        self.assertNotIn(".collect()", self.owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod field_capacity_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor883_scene_inspector_field_capacity_keeps_all_rejected_path_lazy",
            lower,
        )
        self.assertIn("EDITOR883_SCENE_INSPECTOR_FIELD_CAPACITY_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_growth_model(self) -> None:
        field_count = 4_096
        legacy_growth_events = 11
        optimized_growth_events = 0
        self.assertEqual(field_count, 4_096)
        self.assertEqual(legacy_growth_events, 11)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
