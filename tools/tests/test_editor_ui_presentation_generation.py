from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorUiPresentationGenerationContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_present_path_borrows_one_generation_and_enters_its_paint_scope(self) -> None:
        present = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs"
        )

        self.assertIn("get_host_presentation_generation()", present)
        self.assertIn("generation.enter_paint_scope()", present)
        self.assertIn("generation.structure()", present)
        self.assertNotIn("let presentation = event_loop_state.host.get_host_presentation();", present)

    def test_workbench_generation_route_uses_the_committed_hit_index(self) -> None:
        route = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/workbench.rs"
        )

        self.assertIn("generation.workbench_hit_index()", route)
        self.assertIn("hit_test_workbench_window_template_node_with_index", route)
        self.assertNotIn("nodes.iter()", route)

    def test_theme_authority_is_snapshot_based_and_lock_free_for_readers(self) -> None:
        theme = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs"
        )
        child_sources = "\n".join(
            self.read(
                "zircon_editor/src/ui/retained_host/host_contract/paint_theme/" + name
            )
            for name in ("metrics.rs", "palette_projection.rs", "typography.rs")
        )

        self.assertIn("ArcSwap<HostPaintThemeSnapshot>", theme)
        self.assertIn("capture_host_paint_theme_snapshot", theme)
        self.assertNotIn("RwLock", theme + child_sources)

    def test_transient_hover_is_applied_only_to_nodes_selected_for_paint(self) -> None:
        snapshot = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/window/presentation/snapshot.rs"
        )
        draw = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_node_pipeline/draw.rs"
        )

        self.assertNotIn("apply_template_hover_to_presentation", snapshot)
        self.assertIn("apply_template_hover_to_node", draw)
        self.assertIn("visit_paint_workbench_rows", draw)
        self.assertNotIn("paint_workbench_row_indices", draw)


if __name__ == "__main__":
    unittest.main()
