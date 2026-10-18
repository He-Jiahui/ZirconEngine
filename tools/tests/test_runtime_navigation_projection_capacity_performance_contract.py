import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = ROOT / "zircon_runtime/src/navigation/runtime/world_scan.rs"
LOWER = PRODUCTION.parent / "world_scan/tests/capacity_tests.rs"


def function_body(source: str, name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(name)}\s*\([^)]*\)",
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


class RuntimeNavigationProjectionCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = PRODUCTION.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, "collect_navigation_world_projection")

    def test_projection_reserves_each_component_row_bound(self) -> None:
        self.assertIn(
            "let mut agents = Vec::with_capacity(agent_component_rows);",
            self.body,
        )
        self.assertIn(
            "let mut agent_positions = Vec::with_capacity(agent_component_rows);",
            self.body,
        )
        self.assertIn(
            "let mut obstacles = Vec::with_capacity(obstacle_component_rows);",
            self.body,
        )
        self.assertNotIn("let mut agents = Vec::new();", self.body)
        self.assertNotIn("let mut agent_positions = Vec::new();", self.body)
        self.assertNotIn("let mut obstacles = Vec::new();", self.body)

    def test_reservation_happens_after_row_count_and_before_drain(self) -> None:
        first_count = self.body.index("let agent_component_rows = component_rows.len();")
        first_reserve = self.body.index(
            "Vec::with_capacity(agent_component_rows)", first_count
        )
        first_drain = self.body.index("for (entity, value) in component_rows.drain(..)")
        second_count = self.body.index("let obstacle_component_rows = component_rows.len();")
        second_reserve = self.body.index(
            "Vec::with_capacity(obstacle_component_rows)", second_count
        )
        second_drain = self.body.index("for (entity, value) in component_rows.drain(..)", first_drain + 1)
        self.assertLess(first_count, first_reserve)
        self.assertLess(first_reserve, first_drain)
        self.assertLess(second_count, second_reserve)
        self.assertLess(second_reserve, second_drain)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("runtime846_navigation_projection_reserves_component_rows", lower)
        self.assertIn("runtime846_navigation_projection_empty_rows_keep_zero_capacity", lower)
        self.assertIn("RUNTIME846_NAVIGATION_PROJECTION_CAPACITY_BENCH_V1", lower)

    def test_dense_projection_model_removes_geometric_growth(self) -> None:
        row_count = 4_096
        legacy = growth_events(row_count)
        optimized = growth_events(row_count, row_count)
        self.assertGreater(legacy, 0)
        self.assertEqual(optimized, 0)


if __name__ == "__main__":
    unittest.main()
