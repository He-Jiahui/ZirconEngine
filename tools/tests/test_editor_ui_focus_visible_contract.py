import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


def source(relative_path: str) -> str:
    return (REPO_ROOT / relative_path).read_text(encoding="utf-8")


class EditorUiFocusVisibleContractTests(unittest.TestCase):
    def test_zui_focus_borders_use_the_dedicated_focus_ring_token(self):
        editor_ui_root = REPO_ROOT / "zircon_editor/assets/ui/editor"
        stale_focus_borders = []

        for asset_path in editor_ui_root.rglob("*.zui"):
            asset_source = asset_path.read_text(encoding="utf-8")
            if 'focus_border_color = "$editor.accent"' in asset_source:
                stale_focus_borders.append(asset_path.relative_to(REPO_ROOT).as_posix())

        self.assertEqual([], stale_focus_borders)

    def test_runtime_projects_focus_modality_into_retained_attributes(self):
        projection = source(
            "zircon_editor/src/ui/template_runtime/runtime/projection.rs"
        )

        self.assertIn(
            'attributes.insert(\n        "focus_visible".to_string(),', projection
        )
        self.assertIn(
            'attributes.insert("focus_visible_known".to_string(), Value::Boolean(true));',
            projection,
        )
        self.assertIn("component_state.flags.focus_visible", projection)

    def test_retained_adapter_and_host_node_preserve_runtime_focus_modality(self):
        adapter = source("zircon_editor/src/ui/template_runtime/retained_adapter.rs")
        projection = source(
            "zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs"
        )

        self.assertIn(
            'focus_visible_known: bool_attribute(&node.attributes, "focus_visible_known")',
            adapter,
        )
        self.assertIn("focus_visible: node.focus_visible,", projection)
        self.assertIn("focus_visible_known: node.focus_visible_known,", projection)

    def test_editor_selector_keeps_pointer_focus_semantic_but_visually_quiet(self):
        selector = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/state.rs"
        )

        self.assertIn("focused: node.focused", selector)
        self.assertIn("focus_visible: focus_visible_for_node(node)", selector)
        self.assertIn("fn focus_visible_for_node(", selector)
        self.assertIn("if node.focus_visible_known", selector)
        self.assertIn(
            "runtime_pointer_focus_remains_semantic_without_drawing_keyboard_focus",
            selector,
        )
        self.assertIn(
            "runtime_keyboard_focus_and_static_preview_keep_visible_focus", selector
        )

    def test_prominent_commands_use_the_shared_focus_authority(self):
        command = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_button/command.rs"
        )

        self.assertEqual(
            command.count("resolved_state_for_node(node).focus_visible"), 2
        )
        self.assertNotIn("if node.focus_visible_known", command)

    def test_specialized_focus_outlines_use_the_shared_focus_authority(self):
        outline_sources = (
            "material_primitives/text_field/style/stroke.rs",
            "material_state_layer/state.rs",
            "template_asset_placeholder_visuals.rs",
            "template_axis_value_field_style/border.rs",
            "template_chips/style.rs",
            "template_command_palette/panel/search/surface.rs",
            "template_inspector_rows/primitives/field.rs",
            "template_property_rows/fields/scalar.rs",
        )

        for relative_path in outline_sources:
            with self.subTest(relative_path=relative_path):
                painter = source(
                    "zircon_editor/src/ui/retained_host/host_contract/"
                    f"paint_template_nodes/{relative_path}"
                )
                production = painter.split("#[cfg(test)]", maxsplit=1)[0]

                self.assertIn("focus_visible_for_node(node)", production)
                self.assertNotIn("node.focused", production)

    def test_material_state_layer_keeps_selection_distinct_from_focus(self):
        state_layer = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "material_state_layer/state.rs"
        )
        production = state_layer.split("#[cfg(test)]", maxsplit=1)[0]

        self.assertIn("Selected,", production)
        self.assertIn("focus_visible_for_node(node)", production)
        self.assertIn("node.selected || node.checked", production)
        self.assertIn("palette.surface_selected", production)
        self.assertIn("DropTarget,", production)
        self.assertIn("node.drop_hovered || node.active_drag_target", production)
        self.assertIn("palette.text", production)
        self.assertIn("palette.accent", production)
        self.assertNotIn(
            "focus_visible_for_node(node) || node.selected || node.checked",
            production,
        )

    def test_selected_chrome_does_not_reuse_the_focus_outline(self):
        icon_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_icon_button/selection/border.rs"
        )
        generic_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_style/colors/border.rs"
        )

        self.assertIn("UiPainterResolvedState::Selected", icon_border)
        self.assertIn("UiPainterResolvedState::Checked", icon_border)
        self.assertIn("=> Some(palette.panel_border)", icon_border)
        self.assertRegex(
            generic_border,
            r"if asset_thumbnail_card_uses_selected_border\(node\) \{\s*"
            r"return PALETTE\.border;",
        )
        self.assertRegex(
            generic_border,
            r"if node\.selected \|\| node\.checked \{\s*return PALETTE\.border;\s*\}",
        )
        self.assertNotIn(
            "ButtonInteractionState::Focused\n    ) || node.selected",
            generic_border,
        )
        self.assertEqual(generic_border.count("PALETTE.focus_ring"), 1)

    def test_semantic_accent_content_does_not_borrow_the_focus_token(self):
        text_color = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_style/colors/text.rs"
        )
        icon_glyph = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_icon_button/selection/glyph.rs"
        )

        self.assertIn(
            '"accent" | "primary" | "default" => palette.accent', text_color
        )
        self.assertNotIn(
            '"accent" | "primary" | "default" => palette.focus_ring', text_color
        )
        self.assertIn("=> palette.accent", icon_glyph)
        self.assertNotIn("palette.focus_ring", icon_glyph)

    def test_status_controls_reserve_focus_ring_for_focus_and_drop_target(self):
        chips = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_status_control/chips.rs"
        )
        icons = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_status_control/icons.rs"
        )

        self.assertEqual(chips.count("palette.focus_ring"), 1)
        self.assertEqual(icons.count("palette.focus_ring"), 1)
        self.assertIn("=> palette.border", chips)
        self.assertIn("=> palette.border", icons)
        self.assertIn("=> palette.accent", icons)

    def test_active_content_uses_accent_instead_of_focus_ring(self):
        chips = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_chips/style.rs"
        ).split("#[cfg(test)]", maxsplit=1)[0]
        dropdown = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_dropdown/palette.rs"
        )
        toast = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_toast/palette.rs"
        ).split("#[cfg(test)]", maxsplit=1)[0]
        toast_state = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_toast/state.rs"
        ).split("#[cfg(test)]", maxsplit=1)[0]
        tint = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "visual_assets/tint.rs"
        )

        self.assertIn("pub accent: [u8; 4]", chips)
        self.assertIn("palette.accent", chips)
        self.assertNotIn(
            "focus_visible_for_node(node) || node.pressed || node.popup_open", chips
        )
        self.assertIn("active_chevron: palette.accent", dropdown)
        self.assertIn("action: palette.accent", toast)
        self.assertIn("focus_border: palette.focus_ring", toast)
        self.assertIn("style.border = palette.focus_border", toast_state)
        self.assertIn("PALETTE.accent", tint)
        self.assertNotIn("ICON_TINT_ACTIVE: [u8; 4] = PALETTE.focus_ring", tint)

    def test_pressed_dropdowns_and_rows_do_not_draw_keyboard_focus_outlines(self):
        dropdown_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_dropdown/colors/border.rs"
        )
        row_borders = (
            (
                "style_selector/workbench_list_row/surface.rs",
                "fn list_row_border_from_palette",
            ),
            (
                "style_selector/workbench_tree_row/surface.rs",
                "fn tree_row_border_from_palette",
            ),
            (
                "style_selector/workbench_table_row/colors/border.rs",
                "fn table_row_border",
            ),
        )

        self.assertIn(
            "UiPainterResolvedState::Focused | UiPainterResolvedState::DropHovered",
            dropdown_border,
        )
        self.assertIn(
            "UiPainterResolvedState::Pressed | UiPainterResolvedState::Open",
            dropdown_border,
        )
        for relative_path, function_name in row_borders:
            with self.subTest(relative_path=relative_path):
                painter = source(
                    "zircon_editor/src/ui/retained_host/host_contract/"
                    f"paint_template_nodes/{relative_path}"
                )
                border_function = painter.split(function_name, maxsplit=1)[1]
                self.assertNotIn("UiPainterResolvedState::Pressed", border_function)

    def test_selection_control_hot_states_do_not_impersonate_focus(self):
        border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_selection_control/selection/border.rs"
        )

        self.assertIn("uses_focus_outline(state)", border)
        self.assertNotIn("is_hot(state)", border)
        self.assertNotIn(
            "state == UiPainterResolvedState::Focused || is_hot(state)", border
        )

    def test_pressed_shell_chrome_does_not_reuse_keyboard_focus_color(self):
        alert_state = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_alert/state.rs"
        ).split("#[cfg(test)]", maxsplit=1)[0]
        dialog_border = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_dialogs/style/colors/border.rs"
        )
        text_field_surface = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_text_field/surface.rs"
        )

        self.assertNotIn("palette.active_border", alert_state)
        self.assertNotIn("palette.active_border", dialog_border)
        self.assertIn("UiPainterResolvedState::Pressed | UiPainterResolvedState::Open", text_field_surface)
        self.assertIn("UiPainterResolvedState::Focused => palette.focus_border", text_field_surface)

    def test_tooltip_content_uses_accent_while_focus_ring_remains_an_outline(self):
        palette = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_tooltip/palette.rs"
        )
        state = source(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "style_selector/workbench_tooltip/state.rs"
        )

        self.assertIn("icon: palette.accent", palette)
        self.assertIn("style.icon = palette.hover_icon", state)
        pressed_branch = state.split("UiPainterResolvedState::Pressed =>", maxsplit=1)[1]
        pressed_branch = pressed_branch.split("UiPainterResolvedState::Focused =>", maxsplit=1)[0]
        self.assertNotIn("palette.focused_border", pressed_branch)


if __name__ == "__main__":
    unittest.main()
