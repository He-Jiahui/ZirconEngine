use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::{
    WorkbenchButtonKind, ADD_COMPONENT_GLYPH, ADD_COMPONENT_TEXT, OUTLINED_BORDER,
    OUTLINED_SURFACE, OUTLINED_TEXT, PRIMARY_SURFACE,
};
use super::super::super::template_button_glyphs::ButtonGlyph;
use super::super::{
    button_glyph, button_kind, button_opacity, button_paint_rect, button_radius, button_style,
};
use super::support::{
    positioned_button_node, resolved_background, resolved_background_and_border, resolved_border,
    resolved_button_style, resolved_foreground, TemplatePaneNodeDataTestExt,
};
use zircon_runtime_interface::ui::style::ButtonInteractionState;

fn brightened(color: [u8; 4], brightness: f32) -> [u8; 4] {
    [
        brightened_channel(color[0], brightness),
        brightened_channel(color[1], brightness),
        brightened_channel(color[2], brightness),
        color[3],
    ]
}

fn brightened_channel(value: u8, brightness: f32) -> u8 {
    (f32::from(value) * brightness).round().clamp(0.0, 255.0) as u8
}

#[test]
fn disabled_workbench_button_suppresses_declared_style_but_keeps_opacity() {
    let mut node = positioned_button_node(
        "WorkbenchDisabledButton",
        "Disabled",
        "outlined",
        12.0,
        8.0,
        120.0,
        34.0,
    );
    node.disabled = true;
    node.button_style = resolved_button_style(
        [45, 51, 55, 255],
        [52, 61, 68, 255],
        [116, 127, 134, 255],
        0.72,
    );

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PALETTE.surface_disabled);
    assert_eq!(style.border, PALETTE.border_disabled);
    assert_eq!(style.text, PALETTE.text_disabled);
    assert_eq!(style.glyph, PALETTE.text_disabled);
    assert!((button_opacity(&node, 1.0) - 0.72).abs() < 0.001);
}

#[test]
fn add_component_button_uses_audited_offset_and_content_tones() {
    let mut node = positioned_button_node(
        "WorkbenchAddComponent",
        "Add Component",
        "outlined",
        12.0,
        8.0,
        180.0,
        34.0,
    );
    node.button_style = resolved_border([54, 64, 71, 255]);
    let style = button_style(&node, button_kind(&node));
    let rect = button_paint_rect(&node, &node.frame_rect());

    assert_eq!(rect.y, 9.5);
    assert_eq!(style.border, [54, 64, 71, 255]);
    assert_eq!(style.text, ADD_COMPONENT_TEXT);
    assert_eq!(style.glyph, ADD_COMPONENT_GLYPH);
    assert_eq!(button_glyph(&node), ButtonGlyph::Plus);
}

#[test]
fn workbench_secondary_button_uses_declared_surface_color() {
    let mut node = positioned_button_node(
        "WorkbenchSecondaryButton",
        "Secondary",
        "outlined",
        12.0,
        8.0,
        82.0,
        32.0,
    );
    node.button_style = resolved_background([26, 31, 35, 255]);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, [26, 31, 35, 255]);
    assert_eq!(style.border, OUTLINED_BORDER);
    assert_eq!(style.text, OUTLINED_TEXT);
    assert_eq!(button_radius(&node, &node.frame_rect()), 4.0);
}

#[test]
fn workbench_primary_row_uses_declared_metrics_and_brightness() {
    let mut primary = positioned_button_node(
        "WorkbenchPrimaryButton",
        "Primary",
        "filled",
        12.0,
        8.0,
        80.0,
        32.0,
    );
    primary.layout_offset_x = 3.0;
    primary.layout_offset_y = -1.0;
    primary.button_style = resolved_background_and_border([31, 48, 53, 255], [42, 166, 184, 255]);

    let primary_rect = button_paint_rect(&primary, &primary.frame_rect());
    let primary_style = button_style(&primary, button_kind(&primary));

    assert_eq!(primary_rect.x, 15.0);
    assert_eq!(primary_rect.y, 7.0);
    assert_eq!(primary_style.surface, [31, 48, 53, 255]);
    assert_eq!(primary_style.border, [42, 166, 184, 255]);

    let mut secondary = positioned_button_node(
        "WorkbenchSecondaryButton",
        "Secondary",
        "outlined",
        12.0,
        8.0,
        82.0,
        32.0,
    );
    secondary.layout_offset_x = 1.0;
    secondary.layout_offset_y = -1.0;
    secondary.label_brightness = 1.01;
    secondary.button_style = resolved_background([26, 31, 35, 255]);

    let secondary_rect = button_paint_rect(&secondary, &secondary.frame_rect());
    let secondary_style = button_style(&secondary, button_kind(&secondary));

    assert_eq!(secondary_rect.x, 13.0);
    assert_eq!(secondary_rect.y, 7.0);
    assert_eq!(secondary_style.surface, [26, 31, 35, 255]);
    assert_eq!(secondary_style.border, brightened(OUTLINED_BORDER, 1.01));
}

#[test]
fn editor_variant_button_ignores_legacy_declared_material_colors() {
    let mut node = positioned_button_node(
        "OpenAssetsView",
        "Open Assets",
        "primary",
        12.0,
        8.0,
        108.0,
        30.0,
    );
    node.action_id = "workbench.view.open.editor.assets".into();
    node.button_style = resolved_button_style(
        [103, 80, 164, 255],
        [126, 87, 194, 255],
        [255, 255, 255, 255],
        1.0,
    );

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PRIMARY_SURFACE);
    assert_eq!(style.border, OUTLINED_BORDER);
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(style.glyph, PALETTE.text);
}

#[test]
fn workbench_variant_row_uses_declared_surface_and_border() {
    let mut tertiary = positioned_button_node(
        "WorkbenchTertiaryButton",
        "Tertiary",
        "text",
        12.0,
        8.0,
        80.0,
        32.0,
    );
    tertiary.layout_offset_x = 1.0;
    tertiary.corner_radius = 5.0;
    tertiary.button_style = resolved_button_style(
        [23, 28, 32, 255],
        [37, 46, 53, 255],
        [135, 146, 153, 255],
        1.0,
    );
    let tertiary_style = button_style(&tertiary, button_kind(&tertiary));

    assert_eq!(tertiary_style.surface, [23, 28, 32, 255]);
    assert_eq!(tertiary_style.border, [37, 46, 53, 255]);
    assert_eq!(tertiary_style.text, [135, 146, 153, 255]);
    assert_eq!(button_radius(&tertiary, &tertiary.frame_rect()), 5.0);

    let mut outline = positioned_button_node(
        "WorkbenchOutlineButton",
        "Outline",
        "outlined",
        12.0,
        8.0,
        82.0,
        32.0,
    );
    outline.layout_offset_x = 1.0;
    outline.corner_radius = 5.0;
    outline.button_style =
        resolved_button_style([0, 0, 0, 0], [37, 46, 53, 255], [135, 146, 153, 255], 1.0);
    let outline_style = button_style(&outline, button_kind(&outline));

    assert_eq!(outline_style.surface, OUTLINED_SURFACE);
    assert_eq!(outline_style.border, [37, 46, 53, 255]);
    assert_eq!(outline_style.text, [135, 146, 153, 255]);
    assert_eq!(button_radius(&outline, &outline.frame_rect()), 5.0);
}

#[test]
fn workbench_icon_button_uses_declared_surface_and_border() {
    let mut node = positioned_button_node(
        "WorkbenchButtonIcon",
        "Icon",
        "outlined",
        12.0,
        8.0,
        80.0,
        32.0,
    );
    node.button_style = resolved_background_and_border([32, 38, 42, 255], [48, 56, 64, 255]);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, [32, 38, 42, 255]);
    assert_eq!(style.border, [48, 56, 64, 255]);
    assert_eq!(style.text, OUTLINED_TEXT);
    assert_eq!(style.glyph, OUTLINED_TEXT);
}

#[test]
fn workbench_icon_delete_row_uses_declared_content_tones_and_radius() {
    let mut icon = positioned_button_node(
        "WorkbenchButtonIcon",
        "Icon",
        "outlined",
        12.0,
        8.0,
        80.0,
        32.0,
    );
    icon.corner_radius = 5.0;
    icon.label_brightness = 1.02;
    icon.button_style = resolved_foreground([127, 138, 145, 255]);
    let icon_style = button_style(&icon, button_kind(&icon));
    assert_eq!(button_radius(&icon, &icon.frame_rect()), 5.0);
    assert_eq!(icon_style.text, [130, 141, 148, 255]);
    assert_eq!(icon_style.glyph, [130, 141, 148, 255]);

    let mut delete = positioned_button_node(
        "WorkbenchButtonDelete",
        "",
        "outlined",
        12.0,
        8.0,
        82.0,
        32.0,
    );
    delete.validation_level = "danger".into();
    delete.corner_radius = 5.0;
    delete.label_brightness = 1.02;
    delete.button_style = resolved_foreground([208, 90, 80, 255]);
    let delete_style = button_style(&delete, button_kind(&delete));
    assert_eq!(button_radius(&delete, &delete.frame_rect()), 5.0);
    assert_eq!(delete_style.text, [212, 92, 82, 255]);
    assert_eq!(delete_style.glyph, [212, 92, 82, 255]);
}

#[test]
fn workbench_button_applies_declared_visual_brightness() {
    let mut node = positioned_button_node(
        "WorkbenchButtonIcon",
        "Icon",
        "outlined",
        12.0,
        8.0,
        120.0,
        34.0,
    );
    node.label_brightness = 0.96;

    let style = button_style(&node, WorkbenchButtonKind::Secondary);

    assert_eq!(style.surface, brightened(OUTLINED_SURFACE, 0.96));
    assert_eq!(style.border, brightened(OUTLINED_BORDER, 0.96));
    assert_eq!(style.text, brightened(OUTLINED_TEXT, 0.96));
    assert_eq!(style.glyph, brightened(OUTLINED_TEXT, 0.96));
}

#[test]
fn workbench_button_style_selector_applies_state_priority_before_painting() {
    let mut node = positioned_button_node(
        "WorkbenchPrimaryButton",
        "Primary",
        "filled",
        12.0,
        8.0,
        120.0,
        34.0,
    );
    node.hovered = true;
    node.focused = true;
    let focused = button_style(&node, button_kind(&node));
    assert_eq!(focused.interaction, ButtonInteractionState::Focused);

    node.pressed = true;
    let pressed = button_style(&node, button_kind(&node));
    assert_eq!(pressed.interaction, ButtonInteractionState::Pressed);
    assert_eq!(pressed.border, OUTLINED_BORDER);

    node.disabled = true;
    let disabled = button_style(&node, button_kind(&node));
    assert_eq!(disabled.interaction, ButtonInteractionState::Disabled);
}

#[test]
fn asset_browser_tab_like_button_uses_slate_indicator_style() {
    let mut node = positioned_button_node(
        "AssetBrowserKindAllChip",
        "All",
        "outlined",
        12.0,
        8.0,
        72.0,
        24.0,
    );
    node.selected = true;
    node.focused = true;
    node.action_id = "workbench.asset.kind_filter.set".into();
    node.button_style =
        resolved_button_style(PALETTE.accent, PALETTE.focus_ring, PALETTE.focus_ring, 1.0);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PALETTE.surface_hover);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(style.glyph, PALETTE.text);
}

#[test]
fn inactive_asset_browser_tab_like_button_keeps_toolbar_surface_clear() {
    let mut node = positioned_button_node(
        "AssetBrowserKindTextureButton",
        "Texture",
        "outlined",
        12.0,
        8.0,
        78.0,
        24.0,
    );
    node.action_id = "workbench.asset.kind_filter.set".into();
    node.button_style =
        resolved_button_style(PALETTE.accent, PALETTE.focus_ring, PALETTE.focus_ring, 1.0);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, [0, 0, 0, 0]);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, PALETTE.text_muted);
    assert_eq!(style.glyph, PALETTE.text_muted);
}

#[test]
fn hovered_asset_browser_view_tab_uses_low_emphasis_hover_surface() {
    let mut node = positioned_button_node(
        "AssetBrowserViewModeThumbButton",
        "Thumb",
        "outlined",
        12.0,
        8.0,
        78.0,
        24.0,
    );
    node.hovered = true;
    node.action_id = "workbench.asset.view_mode.set".into();
    node.button_style =
        resolved_button_style(PALETTE.accent, PALETTE.focus_ring, PALETTE.focus_ring, 1.0);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PALETTE.surface_hover);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(style.glyph, PALETTE.text);
}

#[test]
fn workbench_module_tab_uses_slate_indicator_style() {
    let mut node = positioned_button_node(
        "WorkbenchModuleEffect",
        "Effect",
        "tab",
        12.0,
        8.0,
        68.0,
        34.0,
    );
    node.selected = true;
    node.checked = true;
    node.action_id = "workbench.module.effect".into();
    node.button_style =
        resolved_button_style(PALETTE.accent, PALETTE.focus_ring, PALETTE.focus_ring, 1.0);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PALETTE.surface_hover);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(style.glyph, PALETTE.text);
}

#[test]
fn inactive_workbench_module_tab_keeps_toolbar_surface_clear() {
    let mut node = positioned_button_node(
        "WorkbenchModuleScene",
        "Scene",
        "tab",
        12.0,
        8.0,
        64.0,
        34.0,
    );
    node.action_id = "workbench.module.scene".into();
    node.button_style =
        resolved_button_style(PALETTE.accent, PALETTE.focus_ring, PALETTE.focus_ring, 1.0);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, [0, 0, 0, 0]);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, PALETTE.text_muted);
    assert_eq!(style.glyph, PALETTE.text_muted);
}

#[test]
fn prominent_workbench_command_button_uses_muted_surface_with_accent_text() {
    let mut node = positioned_button_node(
        "WorkbenchModuleCompile",
        "Compile",
        "filled",
        12.0,
        8.0,
        84.0,
        34.0,
    );
    node.selected = true;
    node.checked = true;
    node.action_id = "workbench.module.compile".into();
    node.button_style = resolved_button_style(
        PALETTE.accent,
        PALETTE.focus_ring,
        PALETTE.shell_background,
        1.0,
    );

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PALETTE.surface_hover);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, PALETTE.accent);
    assert_eq!(style.glyph, PALETTE.accent);
}

#[test]
fn asset_import_command_button_uses_muted_surface_with_accent_text() {
    let mut node =
        positioned_button_node("ImportModel", "Import", "primary", 12.0, 8.0, 96.0, 26.0);
    node.action_id = "workbench.asset.import_model".into();
    node.button_style = resolved_button_style(
        PALETTE.accent,
        PALETTE.focus_ring,
        PALETTE.shell_background,
        1.0,
    );

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PALETTE.surface_pressed);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, PALETTE.accent);
    assert_eq!(style.glyph, PALETTE.accent);
}
