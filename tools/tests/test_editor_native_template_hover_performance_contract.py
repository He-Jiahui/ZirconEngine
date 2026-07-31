from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOVER_ROOT = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/window/template_hover"
)


class EditorNativeTemplateHoverPerformanceContractTests(unittest.TestCase):
    def test_node_model_is_borrow_scanned_before_wide_rows_are_cloned(self) -> None:
        source = (HOVER_ROOT / "nodes.rs").read_text(encoding="utf-8")

        clone_start = source.index("let values: Vec<_>")
        preflight = source[:clone_start]
        self.assertIn("let Some(hovered_row)", preflight)
        self.assertRegex(preflight, r"nodes\s*\.get\(row\)")
        self.assertIn("row == hovered_row", source[clone_start:])

    def test_floating_windows_skip_model_replacement_when_hover_target_is_absent(self) -> None:
        source = (HOVER_ROOT / "panes.rs").read_text(encoding="utf-8")
        function_start = source.index("pub(super) fn apply_template_hover_to_floating_panes")
        function = source[function_start:]
        clone_start = function.index("let floating_windows: Vec<_>")
        preflight = function[:clone_start]

        self.assertIn("floating_windows.get(row)", preflight)
        self.assertIn("pane_contains_template_hover_target", preflight)
        self.assertIn("return;", preflight)


if __name__ == "__main__":
    unittest.main()
