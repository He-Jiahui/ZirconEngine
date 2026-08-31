from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PIPELINE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_pipeline/draw.rs"
)
HOVER = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_pipeline/hover.rs"
)


class EditorTemplateNodeBorrowedPaintPerformanceContractTests(unittest.TestCase):
    def test_untransformed_non_hovered_node_is_pushed_borrowed(self) -> None:
        source = PIPELINE.read_text(encoding="utf-8")
        helper = source.split("fn push_untransformed_template_node_commands", 1)[1]
        helper = helper.split("fn draw_template_nodes", 1)[0]

        self.assertIn("template_hover_targets_node(source_node, interaction)", helper)
        self.assertIn("let mut node = source_node.clone();", helper)
        self.assertIn(
            "push_template_node_commands(commands, source_node, origin, clip, text_input_focus, row)",
            helper,
        )
        self.assertNotIn("clip.clone()", helper)

    def test_untransformed_branch_returns_before_transform_ownership(self) -> None:
        source = PIPELINE.read_text(encoding="utf-8")
        collector = source.split("let mut collect_row", 1)[1]

        stable_branch = collector.index("if transform.is_none() {")
        borrowed_dispatch = collector.index("push_untransformed_template_node_commands(")
        stable_return = collector.index("return;", borrowed_dispatch)
        transform_clone = collector.index("let source_node = source_node.clone();", stable_return)

        self.assertLess(stable_branch, borrowed_dispatch)
        self.assertLess(borrowed_dispatch, stable_return)
        self.assertLess(stable_return, transform_clone)

    def test_hover_match_predicate_is_shared_with_hover_application(self) -> None:
        source = HOVER.read_text(encoding="utf-8")

        self.assertIn("fn template_hover_targets_node", source)
        apply = source.split("fn apply_template_hover_to_node", 1)[1]
        apply = apply.split("fn apply_option_row_hover", 1)[0]
        self.assertIn("if !template_hover_targets_node(node, interaction)", apply)


if __name__ == "__main__":
    unittest.main()
