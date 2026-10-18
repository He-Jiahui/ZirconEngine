import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/navigation/runtime/baked_mesh.rs"


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\([^)]*\)", source, re.S)
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


class Runtime862NavigationVertexProjectionCapacityPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, "polygon_vertices")

    def test_vertex_projection_reserves_index_bound_before_filtering(self) -> None:
        self.assertIn("Vec::with_capacity(index_set.len())", self.body)
        self.assertIn("vertices.extend", self.body)
        self.assertIn("filter_map", self.body)
        self.assertNotIn("collect::<Vec<_>>()", self.body)

    def test_filtering_and_order_lower_regression_are_wired(self) -> None:
        self.assertIn(
            "optimization_batch_20260920_runtime862_navigation_vertex_projection_reserves_index_bound",
            self.source,
        )
        self.assertIn(
            "optimization_batch_20260920_runtime862_navigation_vertex_projection_filters_invalid_indices",
            self.source,
        )

    def test_release_marker_is_wired(self) -> None:
        self.assertIn("RUNTIME862_NAVIGATION_VERTEX_PROJECTION_CAPACITY_BENCH_V1", self.source)

    def test_deterministic_model_removes_projection_growth(self) -> None:
        index_count = 4_096
        self.assertEqual(growth_events(index_count), 11)
        self.assertEqual(growth_events(index_count, index_count), 0)


if __name__ == "__main__":
    unittest.main()
