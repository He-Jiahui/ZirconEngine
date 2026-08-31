import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PREVIEW_ROOT = REPO_ROOT / "zircon_editor" / "src" / "ui" / "asset_editor" / "preview"
ENTRIES_PATH = PREVIEW_ROOT / "preview_mock" / "entries.rs"
PREVIEW_PATH = PREVIEW_ROOT / "preview_mock.rs"


def function_body(source: str, signature: str) -> str:
    match = re.search(signature + r"[^\{]*\{", source)
    if match is None:
        raise AssertionError(f"missing Rust function: {signature}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function: {signature}")
    return source[match.end() : index - 1]


class EditorPreviewMockSelectedEntryPerformanceContractTests(unittest.TestCase):
    def test_selected_entry_scans_borrowed_properties_and_materializes_one_value(self):
        source = ENTRIES_PATH.read_text(encoding="utf-8")
        selected = function_body(source, r"pub\(super\)\s+fn\s+selected_preview_mock_entry")
        selector = function_body(source, r"fn\s+selected_preview_mock_property")

        self.assertNotIn("preview_mock_entries(", selected)
        self.assertNotIn("selected_entry_index(", selected)
        self.assertIn("selected_preview_mock_property(", selected)
        self.assertIn("preview_mock_entry(", selected)
        self.assertIn(".get_key_value(selected)", selector)
        self.assertIn(".filter_map(", selector)
        self.assertIn(".min_by(", selector)
        self.assertIn("preview_mock_sort_key", selector)

    def test_list_and_selected_paths_share_one_entry_materializer(self):
        source = ENTRIES_PATH.read_text(encoding="utf-8")
        entries = function_body(source, r"pub\(super\)\s+fn\s+preview_mock_entries")
        materializer = function_body(source, r"fn\s+preview_mock_entry")

        self.assertIn("preview_mock_entry(", entries)
        self.assertEqual(entries.count("overrides.and_then("), 1)
        self.assertIn("override_value.unwrap_or(value).clone()", materializer)
        self.assertIn("overridden: override_value.is_some()", materializer)

    def test_reconcile_and_clear_do_not_rebuild_the_property_table_for_one_selection(self):
        source = PREVIEW_PATH.read_text(encoding="utf-8")
        reconcile = function_body(source, r"pub\(crate\)\s+fn\s+reconcile_preview_mock_state")
        clear = function_body(source, r"pub\(crate\)\s+fn\s+clear_selected_preview_mock_value")

        for body in (reconcile, clear):
            self.assertNotIn("preview_mock_entries(", body)
            self.assertIn("selected_preview_mock_entry(", body)
        self.assertEqual(reconcile.count("selected_preview_mock_entry("), 1)

    def test_nested_selection_consumes_the_existing_entry_and_moves_the_selected_value(self):
        source = ENTRIES_PATH.read_text(encoding="utf-8")
        body = function_body(
            source,
            r"pub\(super\)\s+fn\s+selected_preview_mock_nested_entry_state",
        )

        self.assertNotIn("selected_preview_mock_entry(", body)
        self.assertRegex(
            source,
            r"selected_preview_mock_nested_entry_state\s*\(\s*entry:\s*&UiAssetPreviewMockEntry",
        )
        self.assertIn("nested_entries.into_iter().nth(selected_index)", body)

    def test_nested_actions_mutate_the_single_materialized_parent_value(self):
        source = PREVIEW_PATH.read_text(encoding="utf-8")
        functions = (
            "set_selected_preview_mock_nested_value",
            "upsert_selected_preview_mock_nested_entry",
            "apply_selected_preview_mock_suggestion",
            "delete_selected_preview_mock_nested_entry",
        )

        for function in functions:
            body = function_body(source, rf"pub\(crate\)\s+fn\s+{function}")
            self.assertNotIn("entry.effective_value.clone()", body, function)
            self.assertIn("let mut next_value = entry.effective_value;", body, function)


if __name__ == "__main__":
    unittest.main()
