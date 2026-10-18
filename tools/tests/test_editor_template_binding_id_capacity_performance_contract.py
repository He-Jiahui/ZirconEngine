from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PROJECTION = ROOT / "zircon_editor/src/ui/template_runtime/runtime/projection.rs"
LOWER_TEST = (
    ROOT / "zircon_editor/src/ui/template_runtime/runtime/projection/tests/binding_capacity_tests.rs"
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


class EditorTemplateBindingIdCapacityPerformanceContractTests(unittest.TestCase):
    def test_authored_projection_reserves_node_binding_ids(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        body = function_body(source, "fn project_node(")

        self.assertIn("Vec::with_capacity(node.bindings.len())", body)
        self.assertIn("binding_ids.push(binding_ref.id.clone())", body)
        self.assertNotIn("let mut binding_ids = Vec::new()", body)

    def test_v2_projection_reserves_node_event_ids(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        body = function_body(source, "fn project_v2_binding_ids(")

        self.assertIn("Vec::with_capacity(node.events.len())", body)
        self.assertIn("binding_ids.push(binding_ref.id.clone())", body)
        self.assertNotIn("let mut binding_ids = Vec::new()", body)

    def test_lower_regression_and_deterministic_capacity_model_are_wired(self) -> None:
        projection_source = PROJECTION.read_text(encoding="utf-8")
        self.assertIn(
            "#[path = \"projection/tests/binding_capacity_tests.rs\"]", projection_source
        )
        self.assertIn("mod binding_capacity_tests;", projection_source)

        source = LOWER_TEST.read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260919_editor837_template_binding_id_capacity_preserves_order",
            source,
        )
        self.assertIn("EDITOR837_TEMPLATE_BINDING_ID_CAPACITY_BENCH_V1", source)

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
