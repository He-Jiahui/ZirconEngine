import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs"
)
LOWER_TESTS = (
    REPO_ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution/tests/optimization_batch_jd_runtime643_tests.rs"
)


class Runtime865HandwrittenMetaDependencyBorrowedIndexPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        cls.merge = (
            source.split("pub(super) fn merge_handwritten_dependencies_into_meta", 1)[1]
            .split("#[cfg(test)]", 1)[0]
        )
        cls.lower_tests = LOWER_TESTS.read_text(encoding="utf-8")

    def test_meta_and_root_indexes_borrow_existing_asset_uris(self) -> None:
        self.assertIn("HashSet<&AssetUri>", self.merge)
        self.assertIn("meta_dependencies.iter()", self.merge)
        self.assertIn("root.dependencies.iter()", self.merge)
        self.assertNotIn(".iter().cloned()", self.merge)

    def test_candidate_classification_uses_one_compact_dual_target_mask(self) -> None:
        self.assertIn("admission_flags", self.merge)
        self.assertIn("meta_addition_count", self.merge)
        self.assertIn("root_addition_count", self.merge)
        self.assertIn("dependencies.into_iter().zip(admission_flags)", self.merge)

    def test_final_projection_clones_only_for_dual_destination_ownership(self) -> None:
        self.assertEqual(self.merge.count("dependency.clone()"), 1)
        self.assertIn("(true, true)", self.merge)
        self.assertIn("(true, false)", self.merge)
        self.assertIn("(false, true)", self.merge)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "optimization_batch_20260921_runtime865_borrowed_dependency_indexes_preserve_dual_destination_order",
            self.lower_tests,
        )
        self.assertIn(
            "RUNTIME865_HANDWRITTEN_META_DEPENDENCY_BORROWED_INDEX_BENCH_V1",
            self.lower_tests,
        )
        self.assertIn("const BORROWED_SAMPLE_PAIRS: usize = 101;", self.lower_tests)
        self.assertIn("owned_index_p99_ns", self.lower_tests)
        self.assertIn("borrowed_index_p99_ns", self.lower_tests)


if __name__ == "__main__":
    unittest.main()
