from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
FRAME = ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_frame/frame.rs"
HOST = ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/host_window.rs"
COMPONENTIZED = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "scene_layers/overlay/componentized.rs"
)


class EditorPaintFrameSharedStatePerformanceContract(unittest.TestCase):
    def test_frame_retains_the_existing_shared_interaction_owner(self):
        source = FRAME.read_text(encoding="utf-8")
        setter = source.split("fn set_pane_interaction_state(", 1)[1]
        setter = setter.split("fn pane_interaction_state(", 1)[0]

        self.assertIn("Option<Arc<HostPaneInteractionStateData>>", source)
        self.assertIn("interaction: Arc<HostPaneInteractionStateData>", setter)
        self.assertIn("Some(interaction)", setter)
        self.assertNotIn("interaction.clone()", setter)
        self.assertIn("self.pane_interaction_state.as_deref()", source)

    def test_all_workbench_entries_transfer_the_arc_without_borrow_clone(self):
        for path in (HOST, COMPONENTIZED):
            source = path.read_text(encoding="utf-8")
            calls = [
                line.strip()
                for line in source.splitlines()
                if "set_pane_interaction_state" in line
            ]
            self.assertTrue(calls, path)
            self.assertTrue(all("(interaction)" in line or "(pane_interaction_state)" in line for line in calls))
            self.assertTrue(all("&interaction" not in line for line in calls))


if __name__ == "__main__":
    unittest.main()
