from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BINDING_ROOT = ROOT / "zircon_editor/src/ui/asset_editor/binding"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetBindingPerformanceContractTests(unittest.TestCase):
    def test_binding_fields_flattens_payload_once_and_has_one_schema_authority(self) -> None:
        inspector = (BINDING_ROOT / "binding_inspector.rs").read_text(encoding="utf-8")
        payload = (
            BINDING_ROOT / "binding_inspector/payload_editing.rs"
        ).read_text(encoding="utf-8")
        body = function_body(
            inspector,
            "pub(crate) fn build_binding_fields(",
            "pub(crate) fn reconcile_selected_binding_index(",
        )

        self.assertEqual(body.count("binding_payload_item_entries(binding)"), 1)
        self.assertIn("selected_payload_key_from_entries(", body)
        self.assertNotIn("binding_schema_items(binding)", body)
        self.assertNotIn("fn binding_schema_items(", payload)
        self.assertNotIn("fn binding_schema_value_literal(", payload)

    def test_single_suggestion_actions_consume_only_the_selected_item(self) -> None:
        source = (BINDING_ROOT / "binding_inspector.rs").read_text(encoding="utf-8")
        functions = (
            function_body(
                source,
                "pub(crate) fn apply_selected_binding_payload_suggestion(",
                "pub(crate) fn apply_selected_binding_route_suggestion(",
            ),
            function_body(
                source,
                "pub(crate) fn apply_selected_binding_route_suggestion(",
                "pub(crate) fn apply_selected_binding_action_suggestion(",
            ),
            function_body(
                source,
                "pub(crate) fn apply_selected_binding_action_suggestion(",
                "fn selected_node<'a>(",
            ),
        )

        for body in functions:
            self.assertIn(".nth(suggestion_index)", body)
            self.assertNotIn(".get(suggestion_index)", body)
            self.assertNotIn(".cloned()", body)

    def test_recursive_binding_projection_sorts_borrowed_table_entries(self) -> None:
        schema = (BINDING_ROOT / "schema_projection.rs").read_text(encoding="utf-8")
        payload = (
            BINDING_ROOT / "binding_inspector/payload_editing.rs"
        ).read_text(encoding="utf-8")
        collector = function_body(
            payload,
            "pub(super) fn collect_binding_payload_item_entries(",
            "pub(super) fn apply_binding_action_state(",
        )

        self.assertNotIn(".keys().cloned()", schema)
        self.assertNotIn(".keys().cloned()", collector)

    def test_payload_delete_materializes_the_next_map_once(self) -> None:
        source = (BINDING_ROOT / "binding_inspector.rs").read_text(encoding="utf-8")
        body = function_body(
            source,
            "pub(crate) fn delete_selected_binding_payload(",
            "pub(crate) fn apply_selected_binding_payload_suggestion(",
        )

        self.assertEqual(body.count("table.clone().into_iter().collect()"), 1)


if __name__ == "__main__":
    unittest.main()
