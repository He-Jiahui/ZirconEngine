from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PREVIEW_ROOT = ROOT / "zircon_editor/src/ui/asset_editor/preview"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetPreviewPerformanceContractTests(unittest.TestCase):
    def test_preview_projection_indexes_control_ids_once(self) -> None:
        source = (PREVIEW_ROOT / "preview_projection.rs").read_text(encoding="utf-8")
        build = function_body(
            source,
            "pub fn build_preview_projection(",
            "pub fn preview_node_id_for_index(",
        )
        lookup = function_body(
            source,
            "pub fn preview_node_id_for_index(",
            "fn preview_item_component_label(",
        )

        self.assertIn("control_id_index(document)", build)
        self.assertIn("control_id_index(document)", lookup)
        self.assertNotIn("node_id_by_control_id", source)

    def test_mock_subject_scans_use_the_nodes_already_being_iterated(self) -> None:
        source = (PREVIEW_ROOT / "preview_mock/entries.rs").read_text(encoding="utf-8")
        resolved = function_body(
            source,
            "pub(super) fn resolved_preview_mock_subject_node_id<'a>(",
            "pub(super) fn selected_preview_mock_entry(",
        )
        subjects = function_body(
            source,
            "pub(super) fn preview_mock_subject_entries(",
            "pub(super) fn preview_mock_subject_label(",
        )

        for body in (resolved, subjects):
            self.assertIn("preview_mock_node_has_entries_for_node(node)", body)
            self.assertNotIn("preview_mock_node_has_entries(document, &node.node_id)", body)

    def test_preview_suggestion_action_consumes_only_the_selected_item(self) -> None:
        source = (PREVIEW_ROOT / "preview_mock.rs").read_text(encoding="utf-8")
        body = function_body(
            source,
            "pub(crate) fn apply_selected_preview_mock_suggestion(",
            "pub(crate) fn delete_selected_preview_mock_nested_entry(",
        )

        self.assertIn(".nth(suggestion_index)", body)
        self.assertNotIn(".get(suggestion_index)", body)
        self.assertNotIn(".cloned()", body)

    def test_expression_resolution_does_not_parse_references_or_functions_twice(self) -> None:
        source = (PREVIEW_ROOT / "mock_value_resolution.rs").read_text(encoding="utf-8")
        reference = function_body(
            source,
            "fn resolve_reference_expression(",
            "pub(super) fn collect_preview_mock_expression_dependencies(",
        )
        dependency = function_body(
            source,
            "fn resolve_reference_dependency(",
            "fn push_dependency(",
        )
        argument = function_body(
            source,
            "fn evaluate_expression_argument(",
            "fn parse_function_expression(",
        )

        self.assertNotIn("parse_preview_mock_reference", reference)
        self.assertNotIn("parse_preview_mock_reference", dependency)
        self.assertNotIn("evaluate_function_expression_from_text", argument)

    def test_preview_recursive_projection_sorts_borrowed_entries(self) -> None:
        sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in PREVIEW_ROOT.rglob("*.rs")
        )
        self.assertNotIn(".keys().cloned()", sources)

    def test_preview_resize_borrows_root_ids(self) -> None:
        source = (PREVIEW_ROOT / "preview_host.rs").read_text(encoding="utf-8")
        body = function_body(source, "    pub fn rebuild_with_size(", "    pub fn surface(")
        self.assertNotIn(".roots.clone()", body)

    def test_mock_reconciliation_reuses_each_resolved_node(self) -> None:
        source = (PREVIEW_ROOT / "preview_mock.rs").read_text(encoding="utf-8")
        reconcile = function_body(
            source,
            "pub(crate) fn reconcile_preview_mock_state(",
            "pub(crate) fn select_preview_mock_subject_node(",
        )
        select = function_body(
            source,
            "pub(crate) fn select_preview_mock_subject_node(",
            "pub(crate) fn select_preview_mock_subject(",
        )

        for body in (reconcile, select):
            self.assertIn("node.props", body)
            self.assertIn(".get(key)", body)
            self.assertNotIn("property_kind(document", body)

    def test_mock_sort_keys_and_bool_parse_do_not_allocate_normalized_strings(self) -> None:
        source = (PREVIEW_ROOT / "preview_mock/entries.rs").read_text(encoding="utf-8")
        parse_bool = function_body(source, "pub(super) fn parse_bool(", "pub(super) fn preview_mock_literal(")

        self.assertNotIn("(u8, String)", source)
        self.assertNotIn("to_ascii_lowercase", parse_bool)
        self.assertNotIn("fn property_kind(", source)


if __name__ == "__main__":
    unittest.main()
