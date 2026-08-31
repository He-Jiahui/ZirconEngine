import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_ROOT = REPO_ROOT / "zircon_editor/assets/ui/editor"
STRICT_THEME = "res://ui/theme/editor_workbench_strict.zui"
LEGACY_THEME = "res://ui/theme/editor_base.zui"
LEGACY_THEME_ASSET_ALLOWLIST = {
    "component_showcase.zui",
    "product_binding_fixture.zui",
}
CHROME_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "chrome_template_projection.rs"
)
MENU_POINTER_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/menu_pointer/"
    "build_host_menu_pointer_layout.rs"
)
ACTIVITY_RAIL_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "chrome_template_projection/activity_rail.rs"
)
DOCK_HEADER_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "chrome_template_projection/dock_header.rs"
)
SIDE_DOCK_HEADER_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "chrome_template_projection/dock_header/side.rs"
)
MENU_CHROME_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "chrome_template_projection/menu_chrome.rs"
)
PAGE_TAB_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "chrome_template_projection/page_tabs.rs"
)
RETAINED_SURFACE_VARIANTS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_style/colors/surface/variants.rs"
)
MENU_POPUP_CONTRACT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/menu_popup_contract.rs"
)

HOST_CHROME_CLASSES = {
    "workbench_activity_rail.zui": {
        "activity_rail_root": "workbench-shell-root",
        "activity_rail_panel": "workbench-rail",
        "activity_rail_button_0": "workbench-rail-button",
        "activity_rail_button_1": "workbench-rail-button",
    },
    "workbench_dock_header.zui": {
        "dock_header_root": "workbench-shell-root",
        "dock_header_bar": "workbench-tabs",
        "dock_tab_0": "workbench-tab",
        "dock_tab_close_0": "workbench-icon-button",
        "dock_tab_1": "workbench-tab",
        "dock_tab_close_1": "workbench-icon-button",
        "dock_tab_2": "workbench-tab",
    },
    "workbench_menu_chrome.zui": {
        "workbench_menu_chrome_root": "workbench-shell-root",
        "workbench_menu_top_bar": "workbench-topbar",
        "workbench_menu_separator": "workbench-divider",
        **{
            f"workbench_menu_slot_{index}": "workbench-tab"
            for index in range(7)
        },
    },
    "workbench_menu_popup.zui": {
        "workbench_menu_popup_panel": "workbench-popup-menu",
        **{
            f"workbench_menu_popup_item_row_{index}": "workbench-list-row"
            for index in range(16)
        },
    },
    "workbench_page_chrome.zui": {
        "workbench_page_chrome_root": "workbench-shell-root",
        "workbench_page_bar": "workbench-tabs",
        "workbench_page_tab_0": "workbench-tab",
        "workbench_page_tab_close_0": "workbench-icon-button",
        "workbench_page_tab_1": "workbench-tab",
        "workbench_page_tab_close_1": "workbench-icon-button",
        "workbench_page_tab_2": "workbench-tab",
        "workbench_page_tab_close_2": "workbench-icon-button",
    },
    "workbench_status_bar.zui": {
        "workbench_status_bar_root": "workbench-shell-root",
        "workbench_status_bar_separator": "workbench-divider",
        "workbench_status_bar_panel": "workbench-status",
        "status_primary_label": "workbench-status-item",
        "status_secondary_label": "workbench-status-item",
        "status_viewport_label": "workbench-status-item",
    },
}

QUIET_ROUNDED_NODES = {
    "workbench_activity_rail.zui": {
        "activity_rail_button_0",
        "activity_rail_button_1",
    },
    "workbench_dock_header.zui": {
        "dock_tab_0",
        "dock_tab_close_0",
        "dock_tab_1",
        "dock_tab_close_1",
        "dock_tab_2",
    },
    "workbench_menu_chrome.zui": {
        f"workbench_menu_slot_{index}" for index in range(7)
    },
    "workbench_menu_popup.zui": {
        f"workbench_menu_popup_item_row_{index}" for index in range(16)
    },
    "workbench_page_chrome.zui": {
        "workbench_page_tab_0",
        "workbench_page_tab_close_0",
        "workbench_page_tab_1",
        "workbench_page_tab_close_1",
        "workbench_page_tab_2",
        "workbench_page_tab_close_2",
    },
}


def load_document(asset_name: str) -> dict:
    with (ASSET_ROOT / asset_name).open("rb") as source:
        return tomllib.load(source)


def rust_f32_constant(source: str, name: str) -> float:
    match = re.search(rf"const {name}: f32 = (?P<value>\d+(?:\.\d+)?);", source)
    if match is None:
        raise AssertionError(f"missing Rust f32 constant: {name}")
    return float(match.group("value"))


class EditorZuiHostChromeStyleContractTests(unittest.TestCase):
    def test_live_host_chrome_uses_one_strict_theme_authority(self):
        for asset_name in HOST_CHROME_CLASSES:
            imports = load_document(asset_name)["imports"]
            self.assertEqual([STRICT_THEME], imports["styles"], asset_name)

        legacy_consumers = {
            path.name
            for path in ASSET_ROOT.rglob("*.zui")
            if LEGACY_THEME
            in load_document(path.relative_to(ASSET_ROOT).as_posix())
            .get("imports", {})
            .get("styles", [])
        }
        self.assertEqual(LEGACY_THEME_ASSET_ALLOWLIST, legacy_consumers)

    def test_live_host_chrome_uses_workbench_semantic_style_roles(self):
        for asset_name, expected_classes in HOST_CHROME_CLASSES.items():
            nodes = load_document(asset_name)["nodes"]
            for node_name, expected_class in expected_classes.items():
                self.assertIn(
                    expected_class,
                    nodes[node_name].get("classes", []),
                    f"{asset_name}:{node_name}",
                )

        popup_nodes = load_document("workbench_menu_popup.zui")["nodes"]
        self.assertNotIn(
            "workbench-popup-menu",
            popup_nodes["workbench_menu_popup_root"].get("classes", []),
            "the popup surface must be painted once by its backing panel",
        )

        for asset_name in (
            "workbench_dock_header.zui",
            "workbench_page_chrome.zui",
        ):
            nodes = load_document(asset_name)["nodes"]
            for node_name, node in nodes.items():
                if "tab_close" in node_name:
                    self.assertIn(
                        "workbench-quiet-action",
                        node.get("classes", []),
                        f"{asset_name}:{node_name}",
                    )

    def test_live_host_chrome_uses_shared_typography_and_radius_tokens(self):
        for asset_name in HOST_CHROME_CLASSES:
            nodes = load_document(asset_name)["nodes"]
            for node_name, node in nodes.items():
                props = node.get("props", {})
                font_size = props.get("font_size")
                if font_size is not None:
                    self.assertIsInstance(
                        font_size, str, f"{asset_name}:{node_name}:font_size"
                    )
                    self.assertTrue(
                        font_size.startswith("$editor.typography."),
                        f"{asset_name}:{node_name}:font_size",
                    )

                radius = props.get("radius")
                if isinstance(radius, (int, float)) and radius > 0.0:
                    self.assertGreaterEqual(radius, 6.0, f"{asset_name}:{node_name}")

        for asset_name, node_names in QUIET_ROUNDED_NODES.items():
            nodes = load_document(asset_name)["nodes"]
            for node_name in node_names:
                props = nodes[node_name].get("props", {})
                self.assertEqual(
                    "transparent",
                    props.get("surface_variant"),
                    f"{asset_name}:{node_name}:surface_variant",
                )
                self.assertEqual(
                    "$editor.control.radius.small",
                    props.get("radius"),
                    f"{asset_name}:{node_name}:radius",
                )

        retained_variants = RETAINED_SURFACE_VARIANTS.read_text(encoding="utf-8")
        self.assertIn('"transparent" => [0, 0, 0, 0]', retained_variants)

        activity_projection = ACTIVITY_RAIL_PROJECTION.read_text(encoding="utf-8")
        self.assertIn(
            'if tab.active { "inset" } else { "transparent" }',
            activity_projection,
        )

        chrome_projection = CHROME_PROJECTION.read_text(encoding="utf-8")
        self.assertIn(
            'node.surface_variant = if tab.active { "inset" } else { "transparent" }',
            chrome_projection,
        )

    def test_menu_popup_stencil_matches_shared_pointer_and_paint_geometry(self):
        nodes = load_document("workbench_menu_popup.zui")["nodes"]
        contract = MENU_POPUP_CONTRACT.read_text(encoding="utf-8")
        padding = rust_f32_constant(contract, "MENU_POPUP_PADDING")
        row_height = rust_f32_constant(contract, "MENU_POPUP_ROW_HEIGHT")
        row_gap = rust_f32_constant(contract, "MENU_POPUP_ROW_GAP")

        for index in range(16):
            expected_y = padding + index * (row_height + row_gap)
            for kind in ("row", "label", "shortcut"):
                node = nodes[f"workbench_menu_popup_item_{kind}_{index}"]
                self.assertEqual(expected_y, node["layout"]["position"]["y"])
                self.assertEqual(
                    {
                        "min": row_height,
                        "preferred": row_height,
                        "max": row_height,
                        "stretch": "Fixed",
                    },
                    node["layout"]["height"],
                )

    def test_host_projection_keeps_document_imports_as_style_authority(self):
        projection = CHROME_PROJECTION.read_text(encoding="utf-8")
        menu_pointer = MENU_POINTER_PROJECTION.read_text(encoding="utf-8")

        for source in (projection, menu_pointer):
            self.assertNotIn("editor_base.zui", source)
            self.assertNotIn("editor_tokens.zui", source)

        self.assertGreaterEqual(projection.count("&[]"), 2)
        self.assertGreaterEqual(menu_pointer.count("&[]"), 2)

    def test_fallback_host_chrome_keeps_quiet_controls_rounded(self):
        sources = {
            path.name: path.read_text(encoding="utf-8")
            for path in (
                CHROME_PROJECTION,
                ACTIVITY_RAIL_PROJECTION,
                DOCK_HEADER_PROJECTION,
                SIDE_DOCK_HEADER_PROJECTION,
                MENU_CHROME_PROJECTION,
                PAGE_TAB_PROJECTION,
            )
        }

        for source_name, source in sources.items():
            self.assertNotIn(
                'else { "" }',
                source,
                f"{source_name} fallback controls must name the transparent surface",
            )
            self.assertNotIn(
                'surface_variant: "".into()',
                source,
                f"{source_name} fallback controls must name the transparent surface",
            )
            self.assertIn(
                "corner_radius: fallback_chrome_control_radius(),",
                source,
                f"{source_name} fallback controls must use the shared small radius",
            )

        self.assertIn(
            "return fallback_activity_rail_nodes(",
            sources[ACTIVITY_RAIL_PROJECTION.name],
            "an unavailable activity-rail stencil must keep usable host chrome",
        )

    def test_page_chrome_uses_dynamic_projection_when_the_stencil_cannot_represent_visible_tabs(self):
        projection = CHROME_PROJECTION.read_text(encoding="utf-8")
        authored_tests = (
            CHROME_PROJECTION.parent
            / "chrome_template_projection/tests/authored_projection.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("page_chrome_stencil_matches_dynamic_layout(", projection)
        self.assertIn(
            "!page_chrome_stencil_matches_dynamic_layout(&nodes, tabs, width)",
            projection,
        )
        self.assertIn(
            "page_chrome_switches_to_dynamic_overflow_when_the_stencil_is_too_small",
            authored_tests,
        )

    def test_dock_header_falls_back_when_any_live_tab_slot_is_missing(self):
        projection = CHROME_PROJECTION.read_text(encoding="utf-8")
        authored_tests = (
            CHROME_PROJECTION.parent
            / "chrome_template_projection/tests/authored_projection.rs"
        ).read_text(encoding="utf-8")
        predicate = projection.split("fn tab_chrome_needs_fallback(", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]

        self.assertIn("ControlFrameIndex::from_nodes(nodes)", predicate)
        self.assertIn("(0..tabs.row_count()).any", predicate)
        self.assertIn("!frame_index.has_positive_width", predicate)
        self.assertIn(
            "dock_header_nodes_project_tabs_beyond_the_authored_stencil",
            authored_tests,
        )

    def test_dock_header_fallback_compacts_tabs_before_they_overflow(self):
        projection = CHROME_PROJECTION.read_text(encoding="utf-8")
        dock_projection = DOCK_HEADER_PROJECTION.read_text(encoding="utf-8")
        side_projection = SIDE_DOCK_HEADER_PROJECTION.read_text(encoding="utf-8")
        authored_tests = (
            CHROME_PROJECTION.parent
            / "chrome_template_projection/tests/authored_projection.rs"
        ).read_text(encoding="utf-8")
        predicate = projection.split("fn tab_chrome_needs_fallback(", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]

        self.assertIn("adaptive_dock_tab_slots(", dock_projection)
        self.assertIn("adaptive_dock_tab_layout(", side_projection)
        self.assertIn("slot.shows_label", dock_projection)
        self.assertIn("frame_within_horizontal_bounds", predicate)
        self.assertIn(
            "dock_header_nodes_compact_inactive_tabs_inside_narrow_document_lane",
            authored_tests,
        )
        self.assertIn(
            "tab_chrome_falls_back_when_an_authored_tab_leaves_the_bar_bounds",
            authored_tests,
        )

    def test_dock_header_publishes_a_reachable_overflow_control(self):
        projection = CHROME_PROJECTION.read_text(encoding="utf-8")
        dock_projection = DOCK_HEADER_PROJECTION.read_text(encoding="utf-8")
        authored_tests = (
            CHROME_PROJECTION.parent
            / "chrome_template_projection/tests/authored_projection.rs"
        ).read_text(encoding="utf-8")

        self.assertIn('DOCK_TAB_OVERFLOW_CONTROL_ID: &str = "DockTabOverflow"', projection)
        self.assertIn("dock_overflow_frame(", projection)
        self.assertIn("adaptive_dock_tab_layout(", dock_projection)
        self.assertIn("FallbackDockTabOverflow", dock_projection)
        self.assertIn("ellipsis-horizontal-outline", dock_projection)
        self.assertIn(
            "dock_header_overflow_reserves_a_reachable_control_for_hidden_tabs",
            authored_tests,
        )


if __name__ == "__main__":
    unittest.main()
