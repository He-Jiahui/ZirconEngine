import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = ROOT / "zircon_runtime/src/scene/world/render_particles.rs"
LOWER = PRODUCTION.parent / "render_particles/tests/capacity_tests.rs"


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


class RuntimeParticleExtractOutputCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = PRODUCTION.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, "collect_render_particles")

    def test_emitter_and_bounds_outputs_reserve_owner_bound(self) -> None:
        self.assertIn(
            "let entity_count = dynamic_component_entities.len();",
            self.body,
        )
        self.assertIn(
            "let mut emitters = Vec::with_capacity(entity_count);",
            self.body,
        )
        self.assertIn(
            "let mut bounds = Vec::with_capacity(entity_count);",
            self.body,
        )
        self.assertNotIn("let mut emitters = Vec::new();", self.body)
        self.assertNotIn("let mut bounds = Vec::new();", self.body)

    def test_reservation_follows_sorted_owner_collection(self) -> None:
        sort_index = self.body.index("dynamic_component_entities.sort_unstable();")
        count_index = self.body.index(
            "let entity_count = dynamic_component_entities.len();", sort_index
        )
        reserve_index = self.body.index(
            "let mut emitters = Vec::with_capacity(entity_count);", count_index
        )
        loop_index = self.body.index("for entity in dynamic_component_entities", reserve_index)
        self.assertLess(sort_index, count_index)
        self.assertLess(count_index, reserve_index)
        self.assertLess(reserve_index, loop_index)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("runtime847_particle_extract_output_capacity_is_bounded", lower)
        self.assertIn("runtime847_particle_extract_empty_outputs_keep_zero_capacity", lower)
        self.assertIn("RUNTIME847_PARTICLE_EXTRACT_OUTPUT_CAPACITY_BENCH_V1", lower)

    def test_dense_owner_model_removes_geometric_growth(self) -> None:
        owner_count = 4_096
        legacy = growth_events(owner_count)
        optimized = growth_events(owner_count, owner_count)
        self.assertGreater(legacy, 0)
        self.assertEqual(optimized, 0)


if __name__ == "__main__":
    unittest.main()
