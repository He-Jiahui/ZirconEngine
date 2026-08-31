import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ROW_STYLE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_rows/surface/row/style.rs"
)
ROW_SURFACE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_rows/surface/row.rs"
)
RUNTIME_POPUP_ROWS = REPO_ROOT / "zircon_runtime/src/ui/surface/render/popup_rows.rs"
RUNTIME_POPUP_MENU = REPO_ROOT / "zircon_runtime/src/ui/surface/render/popup_menu.rs"
RUNTIME_POPUP_OPTIONS = REPO_ROOT / "zircon_runtime/src/ui/surface/render/popup_options.rs"
EDITOR_POPUP_LAYOUT_ROWS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/rows.rs"
)
EDITOR_POPUP_LAYOUT_METRICS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/metrics.rs"
)
EDITOR_POPUP_MENU_PAINT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_popup_rows/menu/entry.rs"
)
EDITOR_POPUP_MENU_HIT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/popup_rows/menu.rs"
)
EDITOR_POPUP_MENU_KEYBOARD = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/menu.rs"
)
TEMPLATE_PANE_NODE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/node.rs"
)
PANE_LAYOUT_OFFSETS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/text_layout/offsets.rs"
)
PANE_CONTENT_ASSIGNMENT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/template_node_data/content.rs"
)
WORKBENCH_WINDOW_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs"
)
PHYSICAL_MOUNT_PROJECTION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/ui/workbench_window_projection/mount.rs"
)


class EditorPopupRowVisualContractTests(unittest.TestCase):
    def test_focus_outline_is_not_discarded_with_a_transparent_fill(self):
        style_source = ROW_STYLE.read_text(encoding="utf-8")
        surface_source = ROW_SURFACE.read_text(encoding="utf-8")

        self.assertNotIn("let fill = style.background?;", style_source)
        self.assertIn(
            "if style.background.is_none() && style.outline.is_none()",
            style_source,
        )
        self.assertIn("fill: style.background", style_source)
        self.assertIn("style.fill,", surface_source)
        self.assertNotIn("Some(style.fill)", surface_source)

    def test_popup_rows_consume_one_projected_padding_and_spacing_authority(self):
        runtime_rows = RUNTIME_POPUP_ROWS.read_text(encoding="utf-8")
        self.assertIn("pub(super) fn popup_row_frame(", runtime_rows)
        for path in (RUNTIME_POPUP_MENU, RUNTIME_POPUP_OPTIONS):
            source = path.read_text(encoding="utf-8")
            self.assertIn("popup_row_frame(metadata, popup_frame", source, path.name)

        pane_node = TEMPLATE_PANE_NODE.read_text(encoding="utf-8")
        runtime_projection = RUNTIME_POPUP_ROWS.read_text(encoding="utf-8")
        pane_offsets = PANE_LAYOUT_OFFSETS.read_text(encoding="utf-8")
        pane_assignment = PANE_CONTENT_ASSIGNMENT.read_text(encoding="utf-8")
        window_projection = WORKBENCH_WINDOW_PROJECTION.read_text(encoding="utf-8")
        mount_projection = PHYSICAL_MOUNT_PROJECTION.read_text(encoding="utf-8")
        for field in (
            "layout_padding_left",
            "layout_padding_right",
            "layout_padding_top",
            "layout_padding_bottom",
            "layout_spacing",
        ):
            self.assertIn(f"pub {field}: f32", pane_node)
            self.assertIn(f'"{field}"', runtime_projection)
            self.assertIn(f'"{field}"', pane_offsets)
            self.assertIn(f"node.{field} = text_layout.{field};", pane_assignment)
            self.assertIn(f'"{field}"', window_projection)
            self.assertIn(f"&mut node.{field}", mount_projection)

        editor_layout = EDITOR_POPUP_LAYOUT_ROWS.read_text(encoding="utf-8")
        self.assertIn("node: &TemplatePaneNodeData", editor_layout)
        for path in (
            EDITOR_POPUP_MENU_PAINT,
            EDITOR_POPUP_MENU_HIT,
            EDITOR_POPUP_MENU_KEYBOARD,
        ):
            source = path.read_text(encoding="utf-8")
            self.assertIn("menu_item_row_frame(node,", source, path.name)

    def test_clamped_menu_rows_derive_height_from_the_arranged_popup_frame(self):
        rows = EDITOR_POPUP_LAYOUT_ROWS.read_text(encoding="utf-8")
        metrics = EDITOR_POPUP_LAYOUT_METRICS.read_text(encoding="utf-8")

        self.assertEqual(2, rows.count("let row_height = popup_row_height("))
        self.assertNotIn("menu_item_row_height", rows)
        self.assertNotIn("fn menu_item_row_height", metrics)


if __name__ == "__main__":
    unittest.main()
