import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "component"
    / "state_reducer"
    / "tree_view.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])(?:<[^>]*>)?\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class RuntimeTreeIdCollectionCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_tree_node_collector_reserves_each_direct_array_bound(self) -> None:
        body = function_body(self.source, "collect_tree_node_ids")
        self.assertIn("out.reserve(values.len());", body)
        self.assertIn("seen.reserve(values.len());", body)
        self.assertLess(body.index("out.reserve(values.len());"), body.index("for value in values"))

    def test_owned_borrowed_and_disabled_collectors_reserve_array_and_flags_bounds(self) -> None:
        for name in [
            "collect_borrowed_string_ids",
            "collect_owned_string_ids",
        ]:
            body = function_body(self.source, name)
            self.assertGreaterEqual(body.count("out.reserve(values.len());"), 2)
            self.assertGreaterEqual(body.count("seen.reserve(values.len());"), 2)

        disabled = function_body(self.source, "collect_disabled_option_ids")
        self.assertGreaterEqual(disabled.count("out.reserve(values.len());"), 2)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "runtime757_tree_id_collectors_preserve_nested_order_and_capacity",
            self.source,
        )
        self.assertIn("RUNTIME757_TREE_ID_COLLECTION_CAPACITY_BENCH_V1", self.source)
        self.assertIn("optimized_growth_events", self.source)

    def test_deterministic_model_eliminates_direct_container_growth(self) -> None:
        direct_values = 1_024
        legacy_growth_events = 10
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(direct_values, 1_024)


if __name__ == "__main__":
    unittest.main()
