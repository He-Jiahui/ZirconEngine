from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PANE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "docks/pane.rs"
)


class EditorWorkbenchDockPaneDamagePerformanceContractTests(unittest.TestCase):
    def test_pane_gate_requires_visible_extent_and_damage_intersection(self) -> None:
        source = PANE.read_text(encoding="utf-8")
        helper = source.split("fn pane_intersects_damage", 1)[1]
        helper = helper.split("fn draw_pane", 1)[0]

        self.assertIn("is_visible_frame(content)", helper)
        self.assertIn("paint_clip.map_or(true", helper)
        self.assertIn("intersect(content, damage).is_some()", helper)

    def test_pane_gate_precedes_shell_and_content_backend_fanout(self) -> None:
        source = PANE.read_text(encoding="utf-8")
        draw = source.split("fn draw_pane", 1)[1]

        gate = draw.index("if !pane_intersects_damage(content, frame.paint_clip())")
        early_return = draw.index("return;", gate)
        shell = draw.index("draw_pane_shell_and_body", early_return)
        content = draw.index("draw_pane_content_layers", shell)

        self.assertLess(gate, early_return)
        self.assertLess(early_return, shell)
        self.assertLess(shell, content)


if __name__ == "__main__":
    unittest.main()
