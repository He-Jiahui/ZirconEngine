import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT / "zircon_editor/src/scene/viewport/render_packet/tests/reused_overlay_storage_tests.rs"
)
PLAN = (
    REPO_ROOT
    / "docs/plans/optimize/zircon_editor/641/2026-09-01-preallocated-scene-gizmos.md"
)


class Editor641SceneGizmoCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.plan = PLAN.read_text(encoding="utf-8")

    def test_release_probe_uses_101_pairs_and_reports_raw_series(self) -> None:
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", self.source)
        self.assertIn("sample_pairs={SAMPLE_PAIRS}", self.source)
        self.assertIn("legacy_samples={}", self.source)
        self.assertIn("optimized_samples={}", self.source)

    def test_scene_gizmo_capacity_bound_remains_direct_and_bounded(self) -> None:
        self.assertIn("let gizmo_capacity = scene", self.source)
        self.assertIn("Vec::with_capacity(gizmo_capacity)", self.source)
        self.assertIn("101 alternating sample pairs", self.plan)


if __name__ == "__main__":
    unittest.main()
