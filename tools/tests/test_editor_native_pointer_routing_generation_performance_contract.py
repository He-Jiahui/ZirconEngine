from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / "zircon_editor/src/ui/retained_host/host_contract"
ROUTING = HOST / "native_pointer/routing"


class EditorNativePointerRoutingGenerationPerformanceContractTests(unittest.TestCase):
    def test_pane_routes_receive_split_interaction_state_explicitly(self) -> None:
        route = (ROUTING / "panes/entry/route.rs").read_text(encoding="utf-8")
        button = (
            HOST / "native_pointer/button_dispatch/entry/body_routes/pane.rs"
        ).read_text(encoding="utf-8")
        move = (
            HOST / "native_pointer/move_dispatch/pane/entry.rs"
        ).read_text(encoding="utf-8")
        scroll = (
            HOST / "native_pointer/scroll_dispatch/pane/entry.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("interaction: &HostPaneInteractionStateData", route)
        self.assertNotIn("presentation.pane_interaction_state", route)
        for source in (button, move, scroll):
            self.assertIn("pane_interaction_state()", source)

    def test_candidate_routes_borrow_every_model_row(self) -> None:
        offenders = []
        for source_path in ROUTING.rglob("*.rs"):
            source = source_path.read_text(encoding="utf-8")
            if "row_data(" in source:
                offenders.append(str(source_path.relative_to(ROOT)))

        self.assertEqual([], offenders)

    def test_floating_header_and_pane_routes_both_visit_topmost_first(self) -> None:
        chrome = (ROUTING / "chrome/floating.rs").read_text(encoding="utf-8")
        pane = (ROUTING / "panes/entry/floating.rs").read_text(encoding="utf-8")

        self.assertIn("floating_windows.iter().rev()", chrome)
        self.assertIn("floating_windows.iter().rev()", pane)

    def test_console_scroll_route_has_nonzero_interaction_regression(self) -> None:
        tests = (
            ROUTING / "panes/entry/route/tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "pane_pointer_route_borrows_generation_owned_targets_and_materializes_only_for_activation",
            tests,
        )
        self.assertIn("console_scroll_px: 18.0", tests)


if __name__ == "__main__":
    unittest.main()
