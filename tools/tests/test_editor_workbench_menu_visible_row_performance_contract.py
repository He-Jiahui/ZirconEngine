from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / "zircon_editor/src/ui/retained_host/host_contract"
ROWS = HOST / "paint_workbench_renderer/menus/rows.rs"
SCROLL = HOST / "paint_workbench_renderer/menus/geometry/scroll.rs"
BAR = HOST / "paint_workbench_renderer/menus/bar.rs"
POPUP = HOST / "paint_workbench_renderer/menus/popup.rs"
METRICS = HOST / "menu_popup_metrics.rs"
OVERFLOW = HOST / "host_page_overflow_menu.rs"


class EditorWorkbenchMenuVisibleRowPerformanceContractTests(unittest.TestCase):
    def test_popup_rows_iterate_shared_visible_range(self) -> None:
        source = ROWS.read_text(encoding="utf-8")
        draw = source.split("fn draw_menu_popup_rows", 1)[1]
        draw = draw.split("fn menu_row_text_frame", 1)[0]

        self.assertIn("for row in menu_popup_visible_row_range(", draw)
        self.assertIn("items.row_count()", draw)
        self.assertIn("popup.height", draw)
        self.assertIn("MENU_POPUP_EDGE_INSET", draw)
        self.assertNotIn("for row in 0..items.row_count()", draw)

    def test_shared_visible_range_owns_strict_intersection_math(self) -> None:
        source = METRICS.read_text(encoding="utf-8")
        function = source.split("fn menu_popup_visible_row_range", 1)[1]

        for fragment in (
            "scroll_offset - first_row_offset - MENU_POPUP_ROW_HEIGHT",
            "scroll_offset + viewport_height - first_row_offset",
            "first_intersection.floor()",
            "last_exclusive",
        ):
            self.assertIn(fragment, function)

    def test_page_overflow_delegates_to_shared_visible_range(self) -> None:
        source = OVERFLOW.read_text(encoding="utf-8")
        function = source.split("fn host_page_overflow_visible_row_range_for_scroll", 1)[1]
        function = function.split("fn host_page_overflow_scroll_offset_for_page", 1)[0]

        self.assertIn("menu_popup_visible_row_range(", function)
        self.assertNotIn("first_intersection", function)

    def test_scroll_geometry_consumes_selected_scalar_state(self) -> None:
        source = SCROLL.read_text(encoding="utf-8")
        bar = BAR.read_text(encoding="utf-8")
        popup = POPUP.read_text(encoding="utf-8")

        self.assertIn("menu_bar_scroll_px: f32", source)
        self.assertNotIn("paint_menu_state", source)
        self.assertNotIn("HostWindowPresentationData", source)
        self.assertIn("menu_state.menu_bar_scroll_px", bar)
        self.assertIn("menu_state.menu_bar_scroll_px", popup)


if __name__ == "__main__":
    unittest.main()
