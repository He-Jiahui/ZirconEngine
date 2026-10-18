import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/asset/registry/dependency_extractors/mod.rs"
LOWER_TESTS = (
    REPO_ROOT / "zircon_runtime/src/asset/registry/dependency_extractors/tests/dedup_index_tests.rs"
)


class Runtime864HandwrittenDependencyMoveAppendPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.lower_tests = LOWER_TESTS.read_text(encoding="utf-8")
        cls.append = (
            cls.source.split("fn append_unique_dependencies", 1)[1]
            .split("pub(crate) fn handwritten_dependencies", 1)[0]
        )

    def test_append_classifies_candidates_with_a_compact_acceptance_mask(self) -> None:
        self.assertIn("Vec::with_capacity(candidates.len())", self.append)
        self.assertIn("accepted.push(is_new)", self.append)
        self.assertIn("accepted_count", self.append)

    def test_accepted_asset_uris_are_moved_without_a_second_uri_clone(self) -> None:
        self.assertIn("candidates", self.append)
        self.assertIn(".into_iter()", self.append)
        self.assertIn(".zip(accepted)", self.append)
        self.assertIn("dependencies.extend", self.append)
        self.assertNotIn("additions.push(dependency.clone())", self.append)

    def test_destination_reserves_the_exact_accepted_count(self) -> None:
        self.assertIn("dependencies.reserve(accepted_count)", self.append)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "optimization_batch_20260921_runtime864_handwritten_dependency_append_moves_candidates",
            self.lower_tests,
        )
        self.assertIn(
            "RUNTIME864_HANDWRITTEN_DEPENDENCY_MOVE_APPEND_BENCH_V1",
            self.lower_tests,
        )
        self.assertIn("const MOVE_SAMPLE_PAIRS: usize = 101;", self.lower_tests)
        self.assertIn("baseline_p99_ns", self.lower_tests)
        self.assertIn("optimized_p99_ns", self.lower_tests)


if __name__ == "__main__":
    unittest.main()
