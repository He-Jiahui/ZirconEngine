from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PROJECTION = ROOT / "zircon_editor/src/core/editor_event/listener/projection.rs"
LOWER_TEST = (
    ROOT / "zircon_editor/src/core/editor_event/listener/projection/tests/capacity_tests.rs"
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


class EditorListenerProjectionCapacityPerformanceContractTests(unittest.TestCase):
    def test_descriptor_projection_reserves_and_extends_once(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        body = function_body(source, "pub(crate) fn listener_descriptors(")

        self.assertIn("Vec::with_capacity(listeners.len())", body)
        self.assertIn("projected.extend(listeners.iter().map(listener_descriptor))", body)
        self.assertNotIn("listeners.iter().map(listener_descriptor).collect()", body)

    def test_delivery_projection_reserves_and_extends_once(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        body = function_body(source, "pub(crate) fn listener_deliveries(")

        self.assertIn("Vec::with_capacity(deliveries.len())", body)
        self.assertIn("projected.extend(deliveries.iter().map(|delivery|", body)
        self.assertNotIn(".collect()", body)

    def test_lower_regression_and_deterministic_capacity_model_are_wired(self) -> None:
        projection_source = PROJECTION.read_text(encoding="utf-8")
        self.assertIn("#[path = \"projection/tests/capacity_tests.rs\"]", projection_source)
        self.assertIn("mod capacity_tests;", projection_source)

        source = LOWER_TEST.read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260830bo_editor_listener_projection_capacity_preserves_count",
            source,
        )
        self.assertIn("EDITOR313_LISTENER_PROJECTION_CAPACITY_BENCH_V1", source)

        item_count = 4096
        self.assertEqual(growth_events(item_count), 11)
        self.assertEqual(growth_events(item_count, item_count), 0)


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
