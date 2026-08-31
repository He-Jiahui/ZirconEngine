from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRIMARY_PRESS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "native_pointer/button_dispatch/primary_press.rs"
)
DISPATCH = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "native_popup_dismiss/dispatch.rs"
)
TARGET = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "native_popup_dismiss/target.rs"
)


class EditorNativePopupDismissPerformanceContractTests(unittest.TestCase):
    def test_primary_press_preserves_one_presentation_generation(self) -> None:
        caller = PRIMARY_PRESS.read_text(encoding="utf-8")
        dispatch = DISPATCH.read_text(encoding="utf-8")

        dismiss_call = caller[caller.index("dispatch_workbench_popup_outside_primary_press(") :]
        self.assertIn("ui,\n        generation,", dismiss_call)
        self.assertIn("generation: &HostPresentationGeneration", dispatch)

    def test_dispatch_borrows_generation_interaction_and_popup_candidates(self) -> None:
        source = DISPATCH.read_text(encoding="utf-8")

        self.assertIn("let interaction = generation.pane_interaction_state();", source)
        self.assertIn("let popup_rows = generation.workbench_hit_index().popup_rows();", source)
        self.assertNotIn("ui.get_pane_interaction_state()", source)
        self.assertIn("popup_rows,", source)

    def test_target_uses_indexed_borrowed_nodes(self) -> None:
        source = TARGET.read_text(encoding="utf-8")

        self.assertIn("popup_rows: &[usize]", source)
        self.assertIn("for row in popup_rows.iter().rev().copied()", source)
        self.assertIn("let Some(node) = nodes.get(row)", source)
        self.assertNotIn("nodes.row_data(row)", source)
        self.assertNotIn("0..nodes.row_count()", source)


if __name__ == "__main__":
    unittest.main()
