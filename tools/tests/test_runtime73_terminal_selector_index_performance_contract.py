from tools.tests.rust_test_files import read_rust_test_file
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/v2/style/rule_index.rs"
LOWER_REGRESSION = ROOT / "zircon_runtime/src/ui/v2/style/rule_index/tests/hash_bucket_tests.rs"


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
                return source[brace + 1 : index]
    raise AssertionError(f"unterminated Rust function: {signature}")


class Runtime73TerminalSelectorIndexPerformanceContract(unittest.TestCase):
    def test_singleton_candidate_path_skips_sort_and_dedup_work(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub(super) fn collect_candidate_indices(")

        fast_path = body.index("if candidates.len() > 1")
        sort = body.index("candidates.sort_unstable();")
        dedup = body.index("candidates.dedup();")
        self.assertLess(fast_path, sort)
        self.assertLess(sort, dedup)

    def test_candidate_order_regression_remains_wired(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")

        self.assertIn("terminal_rule_index_preserves_full_match_results", read_rust_test_file("zircon_runtime/src/ui/v2/style/tests/rule_index.rs"))
        self.assertIn(
            "optimization_batch_20260826cd_selector_hash_buckets_preserve_candidate_order",
            lower,
        )
        self.assertIn("assert_eq!(candidates, vec![0, 1, 2, 3]);", lower)

    def test_empty_candidate_path_clears_reused_scratch(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")

        self.assertIn("candidates.clear();", source)
        self.assertIn(
            "empty_selector_bucket_clears_reused_candidate_scratch",
            lower,
        )


if __name__ == "__main__":
    unittest.main()
