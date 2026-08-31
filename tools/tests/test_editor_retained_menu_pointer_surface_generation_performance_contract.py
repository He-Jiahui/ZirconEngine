from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / "zircon_editor/src/ui/retained_host"
MENU = HOST / "menu_pointer"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    end_offset = source.index(end, offset + len(start))
    return source[offset:end_offset]


class EditorRetainedMenuPointerSurfaceGenerationPerformanceContractTests(
    unittest.TestCase
):
    def test_host_and_bridge_share_one_layout_arc(self) -> None:
        bridge = (MENU / "host_menu_pointer_bridge.rs").read_text(encoding="utf-8")
        app = (HOST / "app.rs").read_text(encoding="utf-8")
        startup = (
            HOST / "app/host_lifecycle/startup/state/interaction.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("layout: Arc<HostMenuPointerLayout>", bridge)
        self.assertIn("menu_pointer_layout: Arc<HostMenuPointerLayout>", app)
        self.assertIn(
            "menu_pointer_layout: Arc::new(HostMenuPointerLayout::default())", startup
        )

    def test_shared_sync_uses_identity_and_borrows_stable_state(self) -> None:
        source = (MENU / "host_menu_pointer_bridge_sync.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub(crate) fn sync_shared(", source)
        self.assertIn("layout: Arc<HostMenuPointerLayout>", source)
        self.assertIn("state: &HostMenuPointerState", source)
        self.assertIn("Arc::ptr_eq(&self.layout, &layout)", source)
        self.assertIn("self.state.clone_from(state)", source)

    def test_host_sync_canonicalizes_layout_and_skips_unchanged_publication(self) -> None:
        source = (HOST / "app/pointer_layout/menu.rs").read_text(encoding="utf-8")
        sync = function_body(
            source,
            "fn sync_menu_pointer_layout(",
            "fn apply_menu_pointer_state_to_ui",
        )

        self.assertIn("let next_layout = build_host_menu_pointer_layout(", sync)
        self.assertIn("self.menu_pointer_layout.as_ref() != &next_layout", sync)
        self.assertIn("Arc::clone(&self.menu_pointer_layout)", sync)
        self.assertIn("&self.menu_pointer_state", sync)
        self.assertIn("if pointer_changed", sync)
        self.assertNotIn("self.menu_pointer_layout.clone()", sync)
        self.assertNotIn("self.menu_pointer_state.clone()", sync)

    def test_equal_popup_items_retain_owned_tree_and_route_index(self) -> None:
        source = (MENU / "host_menu_pointer_bridge_popup_items.rs").read_text(
            encoding="utf-8"
        )
        equality = source.index("self.popup_items.as_slice() == next_items.as_ref()")
        ownership = source.index("next_items.into_owned()")
        indexing = source.index("menu_item_route_indices(&self.popup_items)")

        self.assertLess(equality, ownership)
        self.assertLess(equality, indexing)

    def test_stable_pointer_dispatch_does_not_republish_host_menu_state(self) -> None:
        source = (HOST / "app/menu_pointer.rs").read_text(encoding="utf-8")

        self.assertIn("fn apply_menu_pointer_dispatch_state(", source)
        self.assertIn("if self.menu_pointer_state == state", source)
        self.assertIn("return;", source)
        self.assertEqual(3, source.count("self.apply_menu_pointer_dispatch_state("))


if __name__ == "__main__":
    unittest.main()
