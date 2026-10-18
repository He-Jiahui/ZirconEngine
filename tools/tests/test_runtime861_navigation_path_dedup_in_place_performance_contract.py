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


class Runtime861NavigationPathDedupInPlacePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, "deduplicate_path_points")

    def test_dedup_reuses_the_input_vector(self) -> None:
        self.assertIn("mut points: Vec<NavPathPoint>", self.source)
        self.assertIn("points.dedup_by", self.body)
        self.assertNotIn("let mut result = Vec::new()", self.body)
        self.assertNotIn("Vec::with_capacity", self.body)

    def test_distance_threshold_and_order_semantics_remain_intact(self) -> None:
        self.assertIn("distance_xz", self.body)
        self.assertIn("< 0.05", self.body)
        self.assertIn("optimization_batch_20260920_runtime861_navigation_path_deduplicates_in_place", self.source)
        self.assertIn("optimization_batch_20260920_runtime861_navigation_path_preserves_duplicate_semantics", self.source)
        self.assertIn("transitive_points", self.source)

    def test_release_marker_is_wired(self) -> None:
        self.assertIn("RUNTIME861_NAVIGATION_PATH_DEDUP_IN_PLACE_BENCH_V1", self.source)

    def test_deterministic_model_removes_output_growth(self) -> None:
        point_count = 4_096
        self.assertEqual(growth_events(point_count), 11)
        self.assertEqual(growth_events(point_count, point_count), 0)


if __name__ == "__main__":
    unittest.main()
