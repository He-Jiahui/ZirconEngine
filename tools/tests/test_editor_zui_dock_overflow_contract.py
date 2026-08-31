import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


def source(relative_path: str) -> str:
    return (REPO_ROOT / relative_path).read_text(encoding="utf-8")


class EditorZuiDockOverflowContractTests(unittest.TestCase):
    def test_layout_publishes_one_reserved_anchor_and_hidden_tab_authority(self):
        layout = source(
            "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
            "chrome_template_projection/dock_header.rs"
        )
        scene = source(
            "zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs"
        )
        dock_patch = source(
            "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
            "scene_projection/dock_patch.rs"
        )

        self.assertIn("fn adaptive_dock_tab_layout(", layout)
        self.assertIn("overflow_width + DOCUMENT_TAB_GAP", layout)
        self.assertIn('control_id: DOCK_TAB_OVERFLOW_CONTROL_ID.into()', layout)
        self.assertIn('apply_template_icon(&mut overflow_node, "ellipsis-horizontal-outline")', layout)
        self.assertIn("overflow_frame: dock_overflow_frame(&document_header_nodes)", scene)
        self.assertIn("window.overflow_frame = dock_overflow_frame(&header_nodes)", scene)
        self.assertGreaterEqual(dock_patch.count("dock_overflow_frame(&header_nodes)"), 6)

    def test_popup_uses_published_owner_geometry_for_paint_and_hit_testing(self):
        geometry = source(
            "zircon_editor/src/ui/retained_host/host_contract/host_dock_overflow_menu.rs"
        )
        painter = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/scene_layers/overlay/dock_overflow.rs"
        )
        routes = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
            "routing/chrome/tabs.rs"
        )

        self.assertIn("HostDockOverflowProjection", geometry)
        self.assertIn("anchor_frame: translated(", geometry)
        self.assertIn("tab.frame.width <= f32::EPSILON", geometry)
        self.assertIn("host_dock_overflow_row_hit_in_popup", geometry)
        self.assertIn("host_dock_overflow_visible_row_range_with_state", painter)
        self.assertIn("host_dock_overflow_hidden_indices", painter)
        self.assertIn("route_dock_overflow(", routes)
        self.assertIn("ChromePointerRoute::DockOverflow", routes)

    def test_pointer_keyboard_scroll_and_dismiss_share_the_same_popup_state(self):
        state = source(
            "zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/"
            "dock_overflow.rs"
        )
        press = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
            "button_dispatch/dock_overflow_menu.rs"
        )
        move = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
            "move_dispatch/dock_overflow.rs"
        )
        scroll = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
            "scroll_dispatch/dock_overflow.rs"
        )
        keyboard = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/"
            "target/dock_overflow.rs"
        )
        actions = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch/actions.rs"
        )
        context = source(
            "zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs"
        )
        focus = source(
            "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/focus.rs"
        )

        for field in ("open", "surface_key", "hovered_tab_index", "scroll_offset"):
            self.assertIn(f"pub {field}:", state)
        self.assertIn("invoke_document_tab_pointer_clicked", press)
        self.assertIn("invoke_drawer_header_pointer_clicked", press)
        self.assertIn("host_dock_overflow_row_hit_in_popup", move)
        self.assertIn("host_dock_overflow_scroll_offset_for_delta", scroll)
        self.assertIn("HOST_DOCK_OVERFLOW_DISPATCH_KIND", keyboard)
        self.assertIn("host_dock_overflow_scroll_offset_for_tab", actions)
        self.assertIn("state.host_dock_overflow_menu_state.open", context)
        self.assertIn("HostDockOverflowMenuStateData::default()", focus)

    def test_dock_popup_is_drawn_in_both_host_scene_paths(self):
        scene = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/scene_layers.rs"
        )
        componentized = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/scene_layers/overlay/componentized.rs"
        )

        self.assertIn("draw_host_dock_overflow_menu(frame, presentation)", scene)
        self.assertIn("draw_host_dock_overflow_menu(frame, presentation)", componentized)


if __name__ == "__main__":
    unittest.main()
