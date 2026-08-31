from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/retained_host/route_intent/map.rs"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    end_offset = source.index(end, offset + len(start))
    return source[offset:end_offset]


class EditorRetainedRouteIntentSingleBindingPerformanceContractTests(
    unittest.TestCase
):
    def test_route_intents_use_one_node_binding_hash_map(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]

        self.assertIn("struct EditorRouteBinding", production)
        self.assertIn(
            "bindings_by_node: HashMap<UiNodeId, EditorRouteBinding>", production
        )
        self.assertEqual(1, production.count("HashMap<"))
        self.assertNotIn("intent_by_route", production)

    def test_pointer_intent_resolves_directly_from_the_hit_node(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(
            source,
            "pub(crate) fn intent_for_pointer_dispatch(",
            "pub(crate) fn shell_pointer_route_for_node(",
        )

        self.assertIn("pointer_dispatch_route_node(dispatch)", body)
        self.assertIn("self.intent_for_node(node_id)", body)
        self.assertNotIn("route_id_for_pointer_dispatch", body)

    def test_dead_route_id_reverse_lookup_is_removed(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertNotIn("fn intent_for_route_id(", source)
        self.assertNotIn("fn route_id_for_pointer_dispatch(", source)
        self.assertIn("fn route_id_for_node(", source)


if __name__ == "__main__":
    unittest.main()
