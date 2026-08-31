from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile_projection.rs"
)
SIGNATURE = "fn from_feature_ids(feature_ids: &[String]) -> Self"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class PreallocatedProfileFeatureIndexesPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_feature_id_classes_are_counted_before_allocation(self) -> None:
        self.assertIn(SIGNATURE, self.source)
        body = function_body(self.source, SIGNATURE)
        self.assertIn("let qualified_count =", body)
        self.assertIn("feature_ids.len() - qualified_count", body)

    def test_each_feature_index_uses_exact_category_capacity(self) -> None:
        self.assertIn(SIGNATURE, self.source)
        body = function_body(self.source, SIGNATURE)
        self.assertIn("HashSet::with_capacity(qualified_count)", body)
        self.assertIn("HashSet::with_capacity(short_count)", body)
        self.assertNotIn("HashSet::new()", body)

    def test_projection_and_rust_regression_use_preallocated_helper(self) -> None:
        self.assertIn("SelectedProfileFeatureIds::from_feature_ids(feature_ids)", self.source)
        self.assertIn("preallocated_profile_feature_indexes_preserve_contract", self.source)
        self.assertIn('"rendering.forward"', self.source)
        self.assertIn('"deferred"', self.source)


if __name__ == "__main__":
    unittest.main()
