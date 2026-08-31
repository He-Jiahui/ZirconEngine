import unittest
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MENU_CONTRACT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/menu_popup_contract.rs"
)
MAIN_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/asset_creation_menu.rs"
)
CONTEXT_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/context_menu.rs"
)
MODULE_OVERFLOW_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/module_overflow_menu.rs"
)
WINDOW_MENU_STATE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/"
    "window_menu_state.rs"
)
WORKBENCH_WINDOW = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
)


class EditorContentMeasuredPopupWidthContractTests(unittest.TestCase):
    def test_shared_width_measurement_accepts_a_trailing_adornment_reserve(self):
        source = MENU_CONTRACT.read_text(encoding="utf-8")

        self.assertIn("content_measured_menu_popup_width_with_trailing_reserve", source)
        self.assertIn("(label, shortcut, trailing_reserve)", source)
        self.assertIn("+ trailing_reserve.max(0.0)", source)

    def test_dynamic_main_menu_measures_only_when_source_identity_changes(self):
        source = MAIN_MENU.read_text(encoding="utf-8")

        self.assertIn("desired_width: f32", source)
        self.assertIn("measure_main_menu_width", source)
        self.assertIn("apply_asset_creation_menu_extent", source)
        self.assertIn("self.asset_creation_menu.desired_width", source)
        self.assertIn("node.constraints.width", source)

    def test_dynamic_context_menu_measures_runtime_rows_before_refresh(self):
        source = CONTEXT_MENU.read_text(encoding="utf-8")

        self.assertIn("apply_context_menu_extent(request)", source)
        self.assertIn("content_measured_structured_menu_popup_width", source)
        self.assertIn("node.constraints.width", source)
        self.assertIn("node.constraints.height", source)
        self.assertLess(
            source.index("apply_context_menu_extent(request)"),
            source.index("refresh_after_state_change"),
        )

    def test_dynamic_module_overflow_measures_visible_rows_against_authored_floor(self):
        source = MODULE_OVERFLOW_MENU.read_text(encoding="utf-8")
        window = WORKBENCH_WINDOW.read_text(encoding="utf-8")

        self.assertIn("content_measured_structured_menu_popup_width", source)
        self.assertIn("apply_workbench_module_overflow_menu_extent", source)
        self.assertIn("node.constraints.width", source)
        self.assertIn('layout_min_width = 172.0', window)

    def test_workbench_menus_measure_extent_on_open_and_authored_height_covers_rows(self):
        source = WINDOW_MENU_STATE.read_text(encoding="utf-8")
        window = WORKBENCH_WINDOW.read_text(encoding="utf-8")

        self.assertIn("self.apply_workbench_window_menu_extent(menu.menu_control_id)?", source)
        self.assertIn("menu_popup_content_height(menu_items.len())", source)

        expected_counts = {
            "toolbar_main_menu": 5,
            "toolbar_run_mode_menu": 4,
            "toolbar_layout_menu": 4,
            "toolbar_module_overflow_menu": 5,
            "assets_world_tools_menu": 8,
            "assets_gameplay_tools_menu": 7,
            "assets_production_tools_menu": 7,
            "ability_animation_tools_menu": 8,
            "render_tools_menu": 3,
            "hud_tools_menu": 10,
        }
        for node_name, item_count in expected_counts.items():
            match = re.search(
                rf"\[nodes\.{node_name}\]\n(.*?)(?=\n\[nodes\.)",
                window,
                re.DOTALL,
            )
            self.assertIsNotNone(match, node_name)
            block = match.group(1)
            menu_items = re.search(r"menu_items = \[(.*?)\]", block, re.DOTALL)
            self.assertIsNotNone(menu_items, node_name)
            authored_items = re.findall(r'"(?:[^"\\]|\\.)*"', menu_items.group(1))
            self.assertEqual(len(authored_items), item_count, node_name)
            height = re.search(r"height = \{ min = ([0-9.]+)", block)
            self.assertIsNotNone(height, node_name)
            expected_height = 12.0 + item_count * 28.0 + (item_count - 1) * 2.0
            self.assertGreaterEqual(float(height.group(1)), expected_height, node_name)


if __name__ == "__main__":
    unittest.main()
