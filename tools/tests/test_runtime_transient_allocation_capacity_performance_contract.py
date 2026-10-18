from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/render_graph/graph/transient_allocation.rs"
LOWER_TEST = (
    ROOT / "zircon_runtime/src/render_graph/graph/transient_allocation/tests/capacity_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust function: {signature}")


class RuntimeTransientAllocationCapacityPerformanceContractTests(unittest.TestCase):
    def test_allocation_projection_reserves_filtered_lifetime_bound(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "fn allocate_transient_lifetimes<'a>(")

        self.assertIn("let mut allocations = Vec::with_capacity(lifetimes.len());", body)
        self.assertNotIn("let mut allocations = Vec::new();", body)
        self.assertIn("allocations.push(CompiledRenderGraphTransientAllocation", body)

    def test_lower_regression_and_deterministic_capacity_model_are_wired(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "#[path = \"transient_allocation/tests/capacity_tests.rs\"]", source
        )
        self.assertIn("mod capacity_tests;", source)

        lower = LOWER_TEST.read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260919_runtime840_transient_allocation_capacity_preserves_order",
            lower,
        )
        self.assertIn("RUNTIME840_TRANSIENT_ALLOCATION_CAPACITY_BENCH_V1", lower)
        self.assertEqual(growth_events(4096), 11)
        self.assertEqual(growth_events(4096, 4096), 0)


def growth_events(item_count: int, initial_capacity: int = 0) -> int:
    capacity = initial_capacity
    events = 0
    for length in range(1, item_count + 1):
        if length > capacity:
            capacity = 4 if capacity == 0 else capacity * 2
            events += 1
    return events


if __name__ == "__main__":
    unittest.main()
