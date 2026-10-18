import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/core/asset/dirty/save_batch.rs"
REGRESSION = REPO_ROOT / "zircon_editor/src/core/asset/dirty/save_batch/tests/capacity_tests.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class EditorSaveBatchFailureCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8") if REGRESSION.exists() else ""

    def test_save_preflight_reserves_candidate_failure_bound(self) -> None:
        body = function_body(self.source, "pub fn prepare(")
        compact = "".join(body.split())
        self.assertIn("let mut candidates = candidates.into_iter().collect::<Vec<_>>();", body)
        self.assertIn("Vec::with_capacity(candidates.len())", compact)
        self.assertNotIn("let mut failures = Vec::new();", body)
        self.assertIn("failures.push(", body)

    def test_lower_owner_is_wired_into_the_editor_test_tree(self) -> None:
        self.assertIn("#[path = \"save_batch/tests/capacity_tests.rs\"]", self.source)
        self.assertIn("mod capacity_tests;", self.source)

    def test_lower_capacity_regression_and_release_marker_exist(self) -> None:
        self.assertTrue(self.regression)
        self.assertIn(
            'const BENCHMARK_MARKER: &str = "EDITOR858_SAVE_BATCH_FAILURE_CAPACITY_BENCH_V1";',
            self.regression,
        )
        self.assertIn("editor858_save_batch_failure_capacity_preserves_order", self.regression)
        self.assertIn("editor858_save_batch_failure_capacity_keeps_empty_batch_zero_capacity", self.regression)
        self.assertIn("editor858_save_batch_failure_capacity_bench", self.regression)

    def test_failure_capacity_is_scoped_to_the_collected_candidate_batch(self) -> None:
        body = function_body(self.source, "pub fn prepare(")
        self.assertEqual(body.count("Vec::with_capacity(candidates.len())"), 1)
        self.assertRegex(body, r"candidates\.sort_by_key\(")
        self.assertRegex(body, r"for candidate in &candidates")


if __name__ == "__main__":
    unittest.main()
