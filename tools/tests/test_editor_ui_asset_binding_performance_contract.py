from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BINDING_ROOT = ROOT / "zircon_editor/src/ui/asset_editor/binding"
BINDING_STATE = ROOT / "zircon_editor/src/ui/asset_editor/session/binding_state.rs"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetBindingPerformanceContractTests(unittest.TestCase):
    def test_binding_interactions_query_only_the_selected_authority(self) -> None:
        state = BINDING_STATE.read_text(encoding="utf-8")
        inspector = (BINDING_ROOT / "binding_inspector.rs").read_text(encoding="utf-8")

        self.assertNotIn("build_binding_fields(", state)
        for selector in (
            "selected_binding_count(",
            "binding_event_option(",
            "binding_action_kind_option(",
            "selected_binding_route_suggestion(",
            "selected_binding_action_suggestion(",
            "selected_binding_payload_key(",
            "selected_binding_payload_suggestion(",
        ):
            self.assertIn(selector, state)
            self.assertIn(f"pub(crate) fn {selector}", inspector)
        self.assertNotIn('split_once(" = ")', state)

    def test_binding_fields_flattens_payload_once_and_has_one_schema_authority(self) -> None:
        inspector = (BINDING_ROOT / "binding_inspector.rs").read_text(encoding="utf-8")
        payload = (
            BINDING_ROOT / "binding_inspector/payload_editing.rs"
        ).read_text(encoding="utf-8")
        body = function_body(
            inspector,
            "pub(crate) fn build_binding_fields(",
            "pub(crate) fn selected_binding_count(",
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
                "pub(crate) fn selected_binding_route_suggestion(",
                "pub(crate) fn selected_binding_action_suggestion(",
            ),
            function_body(
                source,
                "pub(crate) fn selected_binding_action_suggestion(",
                "pub(crate) fn selected_binding_payload_key(",
            ),
            function_body(
                source,
                "pub(crate) fn selected_binding_payload_suggestion(",
                "pub(crate) fn reconcile_selected_binding_index(",
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
            "pub(super) fn collect_binding_payload_item_entries",
            "pub(super) fn apply_binding_action_state(",
        )

        self.assertNotIn(".keys().cloned()", schema)
        self.assertNotIn(".keys().cloned()", collector)

    def test_binding_field_payload_projection_borrows_values(self) -> None:
        inspector = (BINDING_ROOT / "binding_inspector.rs").read_text(encoding="utf-8")
        payload = (
            BINDING_ROOT / "binding_inspector/payload_editing.rs"
        ).read_text(encoding="utf-8")
        projection = inspector[inspector.index("fn binding_payload_item_entries") :]
        collector = function_body(
            payload,
            "pub(super) fn collect_binding_payload_item_entries",
            "pub(super) fn apply_binding_action_state(",
        )

        self.assertIn("Vec<(String, &'a Value)>", projection)
        self.assertNotIn("binding_payload_root_value(binding)", projection)
        self.assertIn("entries: &mut Vec<(String, &'a Value)>", collector)
        self.assertNotIn("value.clone()", collector)

        schema_entries = function_body(
            inspector,
            "fn binding_payload_entries",
            "fn binding_payload_item_entries",
        )
        self.assertIn("impl Iterator<Item = (&'a String, &'a Value)>", schema_entries)
        self.assertNotIn("value.clone()", schema_entries)
        self.assertNotIn("collect::<Vec<_>>()", schema_entries)

    def test_payload_delete_materializes_the_next_map_once(self) -> None:
        source = (BINDING_ROOT / "binding_inspector.rs").read_text(encoding="utf-8")
        body = function_body(
            source,
            "pub(crate) fn delete_selected_binding_payload(",
            "fn selected_node<'a>(",
        )

        self.assertEqual(body.count("table.clone().into_iter().collect()"), 1)


if __name__ == "__main__":
    unittest.main()
