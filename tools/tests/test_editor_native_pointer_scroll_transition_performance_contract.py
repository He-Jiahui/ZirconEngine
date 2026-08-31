from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCROLL = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch"
)


class EditorNativePointerScrollTransitionPerformanceContractTests(unittest.TestCase):
    def test_menu_redraws_only_after_interaction_generation_changes(self) -> None:
        source = (SCROLL / "menu.rs").read_text(encoding="utf-8")

        before = source.index("let before = generation.interaction_generation();")
        callback = source.index("invoke_menu_pointer_scrolled")
        unchanged = source.index("before == ui.get_host_interaction_generation()")
        redraw = source.index("NativePointerDispatchResult::region(")

        self.assertLess(before, callback)
        self.assertLess(callback, unchanged)
        self.assertLess(unchanged, redraw)
        self.assertIn("NativePointerDispatchResult::idle()", source)

    def test_page_overflow_consumes_unchanged_boundary_scroll_without_redraw(self) -> None:
        source = (SCROLL / "page_overflow.rs").read_text(encoding="utf-8")

        unchanged = source.index("if !state_changed")
        redraw = source.index("NativePointerDispatchResult::region(popup)")

        self.assertLess(unchanged, redraw)
        self.assertIn(
            "Some(NativePointerDispatchResult::idle())",
            source[unchanged:redraw],
        )

    def test_handled_pane_scroll_redraws_only_after_interaction_change(self) -> None:
        source = (SCROLL / "pane/entry.rs").read_text(encoding="utf-8")

        before = source.index("let before = ui.get_host_interaction_generation();")
        native = source.index("if dispatch_native_pane_scroll")
        unchanged = source.index("before == ui.get_host_interaction_generation()")
        redraw = source.index("NativePointerDispatchResult::region(pointer.frame.clone())")

        self.assertLess(before, native)
        self.assertLess(native, unchanged)
        self.assertLess(unchanged, redraw)

    def test_passive_and_unhandled_pane_scrolls_are_idle(self) -> None:
        source = (SCROLL / "pane/entry.rs").read_text(encoding="utf-8")

        passive = source.index("if is_passive_pane_scroll_target")
        tail = source[passive:]
        self.assertGreaterEqual(tail.count("NativePointerDispatchResult::idle()"), 2)
        self.assertEqual(0, tail.count("NativePointerDispatchResult::region(damage_frame)"))


if __name__ == "__main__":
    unittest.main()
