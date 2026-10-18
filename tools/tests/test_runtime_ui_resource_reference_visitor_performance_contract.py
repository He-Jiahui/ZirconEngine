import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/asset/assets/ui/resource_references.rs"
CALLER = REPO_ROOT / "zircon_runtime/src/asset/assets/ui.rs"
REGRESSION = REPO_ROOT / "zircon_runtime/src/asset/tests/assets/ui.rs"


class RuntimeUiResourceReferenceVisitorPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.caller = CALLER.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_visitor_streams_borrowed_uris_without_intermediate_vector(self) -> None:
        production = self.source.split("#[cfg(test)]", 1)[0]
        self.assertIn("pub(super) fn visit_resource_uris", production)
        self.assertIn("FnMut(&'a str)", production)
        self.assertNotIn("Vec<&str>", production)
        self.assertNotIn("uris.push(uri.to_string())", production)

    def test_asset_reference_projection_uses_the_visitor(self) -> None:
        self.assertIn("visit_resource_uris(document", self.caller)
        self.assertIn("push_reference(uri, &mut references, &mut seen)", self.caller)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "optimization_batch_20260826b_runtime74_ui_resource_reference_visitor_borrows_document_uris",
            self.regression,
        )
        self.assertIn(
            "RUNTIME74_UI_RESOURCE_REFERENCE_VISITOR_BENCH_V1",
            self.source,
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", self.source)
        self.assertIn("legacy_first_pairs={}", self.source)
        self.assertIn("optimized_first_pairs={}", self.source)


if __name__ == "__main__":
    unittest.main()
