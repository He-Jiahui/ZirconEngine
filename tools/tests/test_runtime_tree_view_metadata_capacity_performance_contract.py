import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "surface"
    / "surface"
    / "default_interactions"
    / "tree_view_support.rs"
)
LOWER = SOURCE.parent / "tree_view_support" / "capacity_tests.rs"


def function_body(source: str, name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(name)}(?![A-Za-z0-9_])(?:<[^>]*>)?\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {name}")


def growth_events(item_count: int, initial_capacity: int = 0) -> int:
    capacity = initial_capacity
    events = 0
    for length in range(1, item_count + 1):
        if length > capacity:
            capacity = 4 if capacity == 0 else capacity * 2
            events += 1
    return events


class RuntimeTreeViewMetadataCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_toml_collectors_reserve_direct_array_bounds_before_recursion(self) -> None:
        for name in (
            "collect_tree_node_ids",
            "collect_borrowed_string_ids",
            "collect_owned_string_ids",
        ):
            body = function_body(self.source, name)
            self.assertIn("out.reserve(values.len());", body)
            self.assertIn("seen.reserve(values.len());", body)
            self.assertLess(
                body.index("out.reserve(values.len());"),
                body.index("for value in values"),
            )

        disabled = function_body(self.source, "collect_disabled_option_ids")
        self.assertIn("out.reserve(values.len());", disabled)
        self.assertLess(
            disabled.index("out.reserve(values.len());"),
            disabled.index("for value in values"),
        )

    def test_existing_metadata_entry_points_keep_the_same_collectors(self) -> None:
        for name in (
            "tree_node_ids",
            "tree_node_ids_for_property",
            "selected_ids",
            "expanded_ids",
            "disabled_option_ids",
        ):
            self.assertIn("collect_", function_body(self.source, name))
        self.assertIn("fn range_selected_ids", self.source)
        self.assertIn("Vec::with_capacity(end - start + 1)", self.source)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("runtime842_tree_metadata_collectors_preserve_order", lower)
        self.assertIn("RUNTIME842_TREE_METADATA_COLLECTION_CAPACITY_BENCH_V1", lower)
        self.assertIn("optimized_growth_events", lower)

    def test_dense_direct_array_model_removes_geometric_growth(self) -> None:
        direct_values = 4_096
        legacy_growth_events = growth_events(direct_values)
        optimized_growth_events = growth_events(direct_values, direct_values)
        self.assertGreater(legacy_growth_events, 0)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
