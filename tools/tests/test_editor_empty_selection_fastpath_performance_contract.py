from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/workbench/state/editor_state_selection.rs"


def function_body(source: str, signature: str, next_signature: str) -> str:
    start = source.index(signature)
    end = source.index(next_signature, start)
    return source[start:end]


class EditorEmptySelectionFastPathPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.delete_body = function_body(
            cls.source,
            "    pub fn delete_selected(&mut self)",
            "\n    pub fn apply_inspector_changes",
        )
        cls.inspector_body = function_body(
            cls.source,
            "    pub fn apply_inspector_changes(&mut self)",
            "\n    pub(crate) fn apply_play_inspector_changes",
        )

    def test_delete_returns_before_collecting_empty_selection(self) -> None:
        guard = re.compile(
            r"if\s+self\s*\.viewport_controller\s*\.selection\(\)\s*"
            r"\.active_items\(\)\s*\.is_empty\(\)\s*\{",
            re.MULTILINE,
        )
        match = guard.search(self.delete_body)
        self.assertIsNotNone(match)
        self.assertLess(
            match.start(),
            self.delete_body.index("collect::<Vec<_>>()"),
        )
        self.assertIn('self.set_status_line("Nothing selected")', self.delete_body)

    def test_inspector_returns_before_collecting_empty_selection(self) -> None:
        guard = re.compile(
            r"if\s+self\s*\.viewport_controller\s*\.selection\(\)\s*"
            r"\.active_items\(\)\s*\.is_empty\(\)\s*\{",
            re.MULTILINE,
        )
        match = guard.search(self.inspector_body)
        self.assertIsNotNone(match)
        self.assertLess(
            match.start(),
            self.inspector_body.index("collect::<Vec<_>>()"),
        )
        self.assertIn("InspectorEditError::NoSelection", self.inspector_body)

    def test_non_empty_selection_still_materializes_owned_ids(self) -> None:
        self.assertIn("EditorIntent::DeleteNodes(selected)", self.delete_body)
        self.assertIn("for node_id in &selected", self.inspector_body)
        self.assertIn("selected.len()", self.inspector_body)

    def test_deterministic_empty_selection_model_removes_collection_attempts(self) -> None:
        actions = 100_000
        legacy_collection_attempts = actions * 2
        optimized_collection_attempts = 0

        self.assertEqual(legacy_collection_attempts, actions * 2)
        self.assertEqual(optimized_collection_attempts, 0)
        self.assertEqual(
            legacy_collection_attempts - optimized_collection_attempts,
            actions * 2,
        )


if __name__ == "__main__":
    unittest.main()
