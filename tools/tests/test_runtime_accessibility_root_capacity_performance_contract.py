from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXTRACT = ROOT / "zircon_runtime/src/ui/accessibility/extract.rs"
LOWER_TEST = ROOT / "zircon_runtime/src/ui/accessibility/extract/tests/root_capacity_tests.rs"


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


class RuntimeAccessibilityRootCapacityPerformanceContractTests(unittest.TestCase):
    def test_root_projection_reserves_published_root_bound(self) -> None:
        source = EXTRACT.read_text(encoding="utf-8")
        body = function_body(source, "fn build_accessibility_snapshot(")

        self.assertIn(
            "Vec::with_capacity(surface.tree.roots.len())",
            body,
        )
        self.assertIn("roots.push(root)", body)
        self.assertNotIn("let mut roots = Vec::new()", body)

    def test_lower_regression_and_deterministic_capacity_model_are_wired(self) -> None:
        source = EXTRACT.read_text(encoding="utf-8")
        self.assertIn("#[path = \"extract/tests/root_capacity_tests.rs\"]", source)
        self.assertIn("mod root_capacity_tests;", source)

        lower = LOWER_TEST.read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260919_runtime838_accessibility_root_capacity_preserves_order",
            lower,
        )
        self.assertIn("RUNTIME838_ACCESSIBILITY_ROOT_CAPACITY_BENCH_V1", lower)

        root_count = 4096
        self.assertEqual(growth_events(root_count), 11)
        self.assertEqual(growth_events(root_count, root_count), 0)


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
