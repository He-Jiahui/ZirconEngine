from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SELECTION = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "shell_content_selection.rs"
)


class EditorShellContentSelectionPerformanceContractTests(unittest.TestCase):
    def test_side_pane_priority_is_selected_in_one_slot_traversal(self) -> None:
        source = SELECTION.read_text(encoding="utf-8")
        start = source.index("pub(crate) fn side_pane_selection")
        end = source.index("\npub(crate) fn document_pane_selection", start)
        function = source[start:end]
        compact = "".join(function.split())

        self.assertEqual(compact.count("slots.iter()"), 1)
        self.assertIn("letmutfirst_nonempty=None;", compact)
        self.assertIn("letmutfirst_active=None;", compact)
        self.assertIn(
            "stack.mode!=ActivityDrawerMode::Collapsed{return", compact
        )
        self.assertIn("first_active.or(first_nonempty)?", compact)


if __name__ == "__main__":
    unittest.main()
