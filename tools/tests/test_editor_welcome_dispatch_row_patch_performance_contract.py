from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
APPLY = ROOT / "zircon_editor/src/ui/retained_host/ui/apply_presentation.rs"


class EditorWelcomeDispatchRowPatchPerformanceContractTests(unittest.TestCase):
    def test_welcome_dispatch_patches_only_recognized_rows(self) -> None:
        source = APPLY.read_text(encoding="utf-8")
        start = source.index("fn welcome_nodes_with_native_dispatch(")
        end = source.index("\nfn project_welcome_pane(", start)
        function = source[start:end]
        compact = "".join(function.split())

        self.assertIn("for(row,node)innodes.iter().enumerate()", compact)
        self.assertIn("nodes.with_row_patches(row_patches)", function)
        self.assertNotIn("model_rc(", function)
        self.assertNotIn("row_data(", function)
        self.assertEqual(function.count("node.clone()"), 1)


if __name__ == "__main__":
    unittest.main()
