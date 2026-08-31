from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STRUCTURED = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/selection_options/structured.rs"
)


class EditorSelectionOptionSourcePerformanceContractTests(unittest.TestCase):
    def test_empty_option_source_returns_before_structured_state_projection(self) -> None:
        source = STRUCTURED.read_text(encoding="utf-8")
        function = source.split("pub(super) fn projected_structured_options", 1)[1]

        empty_guard = function.index("if options.is_empty()")
        empty_return = function.index("return Vec::new()")
        projection = function.index("structured_options_for_node")

        self.assertLess(empty_guard, empty_return)
        self.assertLess(empty_return, projection)


if __name__ == "__main__":
    unittest.main()
