import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "template"
    / "asset"
    / "surface_index"
    / "node_resource_registration.rs"
)
LOWER = SOURCE.parent / "capacity_tests.rs"


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


class RuntimeUiNodeResourceRegistrationCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_report_reserves_node_output_bound_before_iteration(self) -> None:
        body = function_body(self.source, "record_tree_node_resources")
        self.assertIn("report.nodes_without_resources.reserve(tree.nodes.len())", body)
        self.assertLess(
            body.index("report.nodes_without_resources.reserve(tree.nodes.len())"),
            body.index("report.nodes_without_resources.push(*node_id)"),
        )

    def test_metadata_collector_uses_bounded_uri_capacity(self) -> None:
        body = function_body(self.source, "record_tree_node_resources")
        self.assertIn("NodeResourceCollector::with_capacity", body)
        self.assertRegex(body, r"metadata\s*\.attributes\s*\.len\(\)")
        self.assertRegex(body, r"metadata\s*\.slot_attributes\s*\.len\(\)")
        self.assertRegex(body, r"metadata\s*\.style_overrides\s*\.len\(\)")
        collector = function_body(self.source, "with_capacity")
        self.assertIn("uris: Vec::with_capacity(capacity)", collector)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("runtime843_node_resource_collector_reserves_capacity", lower)
        self.assertIn("RUNTIME843_NODE_RESOURCE_REGISTRATION_CAPACITY_BENCH_V1", lower)
        self.assertIn("optimized_growth_events", lower)

    def test_dense_node_report_model_removes_geometric_growth(self) -> None:
        node_count = 4_096
        legacy_growth_events = growth_events(node_count)
        optimized_growth_events = growth_events(node_count, node_count)
        self.assertGreater(legacy_growth_events, 0)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
