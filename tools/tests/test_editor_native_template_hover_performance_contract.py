from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST_CONTRACT_ROOT = ROOT / "zircon_editor/src/ui/retained_host/host_contract"
PAINT_HOVER = (
    HOST_CONTRACT_ROOT
    / "paint_template_nodes/template_node_pipeline/hover.rs"
)


class EditorNativeTemplateHoverPerformanceContractTests(unittest.TestCase):
    def test_non_matching_control_returns_before_row_models_are_cloned(self) -> None:
        source = PAINT_HOVER.read_text(encoding="utf-8")

        clone_start = source.index("let options: Vec<_>")
        preflight = source[:clone_start]
        predicate = source.split("fn template_hover_targets_node", 1)[1]
        predicate = predicate.split("fn apply_template_hover_to_node", 1)[0]
        self.assertIn("interaction.hovered_template_control_id.is_empty()", predicate)
        self.assertIn("node.control_id.as_str() ==", predicate)
        self.assertIn("if !template_hover_targets_node(node, interaction)", preflight)
        self.assertIn("return;", preflight)

    def test_presentation_snapshot_does_not_materialize_hover_into_models(self) -> None:
        snapshot = (
            HOST_CONTRACT_ROOT / "window/presentation/snapshot.rs"
        ).read_text(encoding="utf-8")
        draw = (
            HOST_CONTRACT_ROOT
            / "paint_template_nodes/template_node_pipeline/draw.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("apply_template_hover_to_presentation", snapshot)
        self.assertNotIn("apply_template_hover_to_floating_panes", snapshot)
        self.assertIn("apply_template_hover_to_node", draw)


if __name__ == "__main__":
    unittest.main()
