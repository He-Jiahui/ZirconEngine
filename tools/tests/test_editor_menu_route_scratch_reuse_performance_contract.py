from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MENU_POINTER = ROOT / "zircon_editor/src/ui/retained_host/menu_pointer"
BRIDGE = MENU_POINTER / "host_menu_pointer_bridge.rs"
NEW = MENU_POINTER / "host_menu_pointer_bridge_new.rs"
ROUTE = MENU_POINTER / "host_menu_pointer_bridge_project_route.rs"


class EditorMenuRouteScratchReusePerformanceContractTests(unittest.TestCase):
    def test_bridge_retains_popup_projection_scratch(self) -> None:
        source = BRIDGE.read_text(encoding="utf-8")
        constructor = NEW.read_text(encoding="utf-8")

        self.assertIn("popup_item_path_scratch: Vec<usize>,", source)
        self.assertIn("popup_item_path_scratch: Vec::new(),", constructor)

    def test_popup_projection_reuses_and_clears_the_retained_scratch(self) -> None:
        source = ROUTE.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]

        self.assertIn(
            "pub(super) fn project_open_popup_route_at_point(\n        &mut self,",
            source,
        )
        self.assertIn("self.popup_item_path_scratch.clear();", source)
        self.assertIn(
            "fn prepare_popup_item_path_scratch(target: &mut Vec<usize>, open_depth: usize)",
            source,
        )
        self.assertIn("target.clear();", source)
        self.assertIn("target.reserve(open_depth.saturating_add(1));", source)
        self.assertIn(
            "item_path: &mut self.popup_item_path_scratch,",
            source,
        )
        prepare_at = source.index("prepare_popup_item_path_scratch(")
        project_at = source.index("let route = project_popup_layer(")
        clear_at = source.index("self.popup_item_path_scratch.clear();", project_at)
        self.assertLess(prepare_at, project_at)
        self.assertLess(project_at, clear_at)
        self.assertNotIn(
            "let mut item_path = popup_item_path(self.state.open_submenu_path.as_slice());",
            production,
        )

    def test_route_projection_mutability_is_limited_to_dispatch_paths(self) -> None:
        route = ROUTE.read_text(encoding="utf-8")
        dispatch = (
            MENU_POINTER / "host_menu_pointer_bridge_dispatch_event.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "pub(super) fn project_route_at_point(\n        &mut self,",
            route,
        )
        self.assertIn("self.project_route_at_point(route, point)", dispatch)
        self.assertNotIn(
            "pub(super) fn project_open_popup_route_at_point(\n        &self,",
            route,
        )


if __name__ == "__main__":
    unittest.main()
