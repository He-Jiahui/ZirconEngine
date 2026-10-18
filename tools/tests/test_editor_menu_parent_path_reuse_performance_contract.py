from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MENU_POINTER = ROOT / "zircon_editor/src/ui/retained_host/menu_pointer"
TREE = MENU_POINTER / "menu_item_tree.rs"
MOVE = MENU_POINTER / "host_menu_pointer_bridge_handle_move.rs"
SCROLL = MENU_POINTER / "host_menu_pointer_bridge_handle_scroll.rs"
CLICK = MENU_POINTER / "host_menu_pointer_bridge_handle_click.rs"


class EditorMenuParentPathReusePerformanceContractTests(unittest.TestCase):
    def test_parent_path_copy_reuses_the_retained_state_buffer(self) -> None:
        source = TREE.read_text(encoding="utf-8")

        self.assertIn(
            "pub(in crate::ui::retained_host::menu_pointer) fn reuse_parent_path(",
            source,
        )
        self.assertIn("target: &mut Vec<usize>", source)
        self.assertIn("let parent_len = path.len().saturating_sub(1);", source)
        self.assertIn("target.clear();", source)
        self.assertIn("target.extend_from_slice(&path[..parent_len]);", source)
        self.assertNotIn(
            "fn parent_path(path: &[usize]) -> Vec<usize>",
            source,
        )

    def test_pointer_routes_compare_borrowed_parent_before_copying(self) -> None:
        move = MOVE.read_text(encoding="utf-8")
        scroll = SCROLL.read_text(encoding="utf-8")
        click = CLICK.read_text(encoding="utf-8")

        for source in (move, scroll, click):
            self.assertIn("use super::menu_item_tree::reuse_parent_path;", source)
            self.assertNotIn("::parent_path;", source)

        for source in (move, scroll):
            self.assertIn("let parent_len = item_path.len().saturating_sub(1);", source)
            self.assertIn(
                "self.state.open_submenu_path.as_slice() != &item_path[..parent_len]",
                source,
            )
            self.assertIn(
                "reuse_parent_path(&mut self.state.open_submenu_path, item_path);",
                source,
            )

        self.assertIn(
            "reuse_parent_path(&mut self.state.open_submenu_path, item_path);",
            click,
        )


if __name__ == "__main__":
    unittest.main()
