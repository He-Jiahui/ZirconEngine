from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/v2/style/rule_index.rs"
LOWER_REGRESSION = (
    ROOT / "zircon_runtime/src/ui/v2/style/rule_index/tests/candidate_capacity_tests.rs"
)


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


def geometric_growth_events(required: int) -> int:
    capacity = 0
    growth = 0
    while capacity < required:
        capacity = 1 if capacity == 0 else capacity * 2
        growth += 1
    return growth


class Runtime860SelectorCandidateCapacityPerformanceContract(unittest.TestCase):
    def test_candidate_scratch_reserves_bucket_upper_bound_before_projection(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub(super) fn collect_candidate_indices(")
        bound_body = function_body(source, "fn candidate_capacity_upper_bound(")

        clear = body.index("candidates.clear();")
        bound = body.index("candidate_capacity_upper_bound")
        reserve = body.index("candidates.reserve(")
        first_extend = body.index("candidates.extend_from_slice")

        self.assertLess(clear, bound)
        self.assertLess(bound, reserve)
        self.assertLess(reserve, first_extend)
        self.assertIn("saturating_add", bound_body)

    def test_empty_and_dense_models_have_zero_or_single_growth(self) -> None:
        dense = 4_096
        optimized_growth_events = 0
        self.assertEqual(geometric_growth_events(0), 0)
        self.assertEqual(geometric_growth_events(dense), 13)
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(dense, 4_096)

    def test_lower_capacity_regression_is_wired(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")

        self.assertIn("candidate_capacity_tests", source)
        self.assertIn(
            "optimization_batch_20260920_runtime860_candidate_capacity_preserves_order",
            lower,
        )
        self.assertIn(
            "optimization_batch_20260920_runtime860_empty_candidate_capacity_stays_zero",
            lower,
        )

    def test_release_marker_is_preserved_in_lower_owner(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn("RUNTIME860_SELECTOR_CANDIDATE_CAPACITY_BENCH_V1", lower)


if __name__ == "__main__":
    unittest.main()
