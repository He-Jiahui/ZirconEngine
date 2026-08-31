from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXECUTE_RS = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/"
    "post_process/pass_graph/execute.rs"
)


def fallback_body() -> str:
    source = EXECUTE_RS.read_text(encoding="utf-8")
    return source.split("let produced_resource_count", 1)[1].split(
        "fn executed_post_process_effect_mask", 1
    )[0]


class Runtime09H2BorrowedPassResourcesContract(unittest.TestCase):
    def test_fallback_resource_sets_borrow_graph_names(self) -> None:
        source = EXECUTE_RS.read_text(encoding="utf-8")
        body = fallback_body()

        self.assertIn("use std::collections::HashSet;", source)
        self.assertNotIn("BTreeSet", body)
        self.assertNotIn(".cloned()", body)
        self.assertIn("HashSet<&str>", body)
        self.assertIn(".map(String::as_str)", body)

    def test_fallback_resource_sets_reserve_the_graph_bound(self) -> None:
        body = fallback_body()

        self.assertIn("produced_resource_count", body)
        self.assertIn("resource_reference_count", body)
        self.assertIn("HashSet::with_capacity(produced_resource_count)", body)
        self.assertIn("HashSet::with_capacity(resource_reference_count)", body)

    def test_fallback_keeps_ordered_node_execution(self) -> None:
        body = fallback_body()

        self.assertIn("for node in &graph.nodes", body)
        self.assertIn(
            "record.push_executed_post_process_node(node.name.clone())", body
        )

    def test_bound_resources_are_admitted_once(self) -> None:
        body = fallback_body()
        execution_loop = body.rsplit("for node in &graph.nodes", 1)[1]

        self.assertNotIn("resources.has_bound_resource(resource) &&", execution_loop)
        self.assertIn(".filter(|resource| resources.has_bound_resource(resource))", execution_loop)


if __name__ == "__main__":
    unittest.main()
