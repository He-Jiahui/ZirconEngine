import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips/identity.rs"
)
REGRESSION = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips/identity/tests/cached_control_id_tests.rs"
)


class EditorCachedControlIdPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_production_caches_control_id_before_branching(self) -> None:
        production = self.source.split("#[cfg(test)]", 1)[0]
        self.assertIn("let control_id = node.control_id.as_str();", production)
        self.assertEqual(production.count("node.control_id.as_str()"), 1)
        self.assertNotIn("node.control_id.starts_with", production)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"identity/tests/cached_control_id_tests.rs\"]",
            self.source,
        )
        self.assertIn("mod cached_control_id_tests;", self.source)
        self.assertIn(
            "optimization_batch_20260830bd_editor301_cached_control_id_preserves_results",
            self.regression,
        )
        self.assertIn(
            "EDITOR301_CACHED_CONTROL_ID_BENCH_V1",
            self.regression,
        )

    def test_lower_regression_checks_single_source_view(self) -> None:
        self.assertIn("node.control_id.as_str();", self.regression)
        self.assertIn('production.matches("node.control_id.as_str()").count(), 1', self.regression)


if __name__ == "__main__":
    unittest.main()
