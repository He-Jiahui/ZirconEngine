use super::super::super::super::paint_theme::{METRICS, PALETTE, current_host_palette};
use super::super::super::style_selector::{
    WorkbenchButtonKind, add_component_glyph_color_from_host, add_component_text_color_from_host,
    workbench_button_border_width_from_host, workbench_button_transparent_surface,
};
use super::super::super::template_button_glyphs::ButtonGlyph;
use super::super::{
    add_component_button_offset_y_from_host, button_content_metrics_from_host,
    button_geometry_metrics_from_host, button_glyph, button_icon_size_from_host, button_kind,
    button_opacity, button_paint_rect, button_radius, button_style,
};
use super::support::{
    TemplatePaneNodeDataTestExt, positioned_button_node, resolved_background,
    resolved_background_and_border, resolved_border, resolved_button_style, resolved_foreground,
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
    assert_eq!(
        style.text,
        add_component_text_color_from_host(current_host_palette())
    );
    assert_eq!(
        style.glyph,
        add_component_glyph_color_from_host(current_host_palette())
    );
    assert_eq!(button_glyph(&node), ButtonGlyph::Plus);
}

#[test]
fn add_component_button_content_tones_project_from_host_palette() {
    let palette = current_host_palette();

    assert_eq!(
        add_component_text_color_from_host(palette),
        palette.text_muted
    );
    assert_eq!(add_component_glyph_color_from_host(palette), palette.text);
}

#[test]
fn add_component_button_offset_projects_from_host_border_metric() {
    let mut metrics = METRICS;
    assert_eq!(add_component_button_offset_y_from_host(metrics), 1.5);

    metrics.border_width = 2.0;
    assert_eq!(add_component_button_offset_y_from_host(metrics), 3.0);
}

#[test]
fn button_geometry_metrics_project_from_host_control_metrics() {
    let mut metrics = METRICS;
    metrics.radius_control = 6.0;
    metrics.border_width = 2.0;

    let projected = button_geometry_metrics_from_host(metrics);

    assert_eq!(projected.radius, 6.0);
    assert_eq!(projected.add_component_offset_y, 3.0);
}

#[test]
fn button_glyph_slots_project_from_host_control_metrics() {
    let mut metrics = METRICS;
    assert_eq!(button_icon_size_from_host(metrics), 16.0);

    metrics.font_large = 18.0;
    metrics.row_height = 32.0;
    metrics.gap_l = 12.0;
    metrics.button_icon_gap = 9.0;
    metrics.button_chevron_reserve = 24.0;
    metrics.button_pad_x = 11.0;
    metrics.font_body = 13.0;
    metrics.text_clip_guard = 5.0;
    metrics.gap_s = 3.0;
    metrics.gap_m = 6.0;
    metrics.button_pressed_offset_y = 2.0;

    let content_metrics = button_content_metrics_from_host(metrics);

    assert_eq!(button_icon_size_from_host(metrics), 20.0);
    assert_eq!(content_metrics.icon_gap, 9.0);
    assert_eq!(content_metrics.chevron_reserve, 24.0);
    assert_eq!(content_metrics.trailing_glyph_inset, 11.0);
    assert_eq!(content_metrics.font_size, 13.0);
    assert_eq!(content_metrics.text_clip_guard, 5.0);
    assert_eq!(content_metrics.utility_tab_pad_x, 3.0);
    assert_eq!(content_metrics.toolbar_chip_pad_x, 6.0);
    assert_eq!(content_metrics.button_pad_x, 11.0);
    assert_eq!(content_metrics.pressed_offset_y, 2.0);
}

#[test]
fn workbench_button_border_width_projects_from_host_control_metrics() {
    let mut metrics = METRICS;
    assert_eq!(workbench_button_border_width_from_host(metrics), 1.0);

    metrics.border_width = 2.0;
    assert_eq!(workbench_button_border_width_from_host(metrics), 2.0);
}

#[test]
fn workbench_button_transparent_surface_owns_fillless_chrome_role() {
    assert_eq!(workbench_button_transparent_surface(), [0, 0, 0, 0]);
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
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(button_radius(&node, &node.frame_rect()), 4.0);
}

#[test]
fn workbench_primary_row_uses_low_emphasis_tokens_and_declared_metrics() {
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
    assert_eq!(primary_style.surface, PALETTE.surface_pressed);
    assert_eq!(primary_style.border, PALETTE.border);

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
    assert_eq!(secondary_style.border, brightened(PALETTE.border, 1.01));
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

    assert_eq!(style.surface, PALETTE.surface_pressed);
    assert_eq!(style.border, PALETTE.border);
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

    assert_eq!(outline_style.surface, PALETTE.surface_pressed);
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
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(style.glyph, PALETTE.text);
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
fn workbench_danger_button_uses_low_emphasis_surface_with_danger_content() {
    let mut node = positioned_button_node(
        "WorkbenchDangerButton",
        "Delete",
        "danger",
        12.0,
        8.0,
        88.0,
        32.0,
    );
    node.validation_level = "danger".into();
    node.button_style = resolved_button_style(
        PALETTE.error_container,
        PALETTE.error,
        [208, 90, 80, 255],
        1.0,
    );

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PALETTE.surface_pressed);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.text, [208, 90, 80, 255]);
    assert_eq!(style.glyph, [208, 90, 80, 255]);
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

    assert_eq!(style.surface, brightened(PALETTE.surface_pressed, 0.96));
    assert_eq!(style.border, brightened(PALETTE.border, 0.96));
    assert_eq!(style.text, brightened(PALETTE.text, 0.96));
    assert_eq!(style.glyph, brightened(PALETTE.text, 0.96));
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
    assert_eq!(pressed.border, PALETTE.border);

    node.disabled = true;
    let disabled = button_style(&node, button_kind(&node));
    assert_eq!(disabled.interaction, ButtonInteractionState::Disabled);
}

#[test]
fn asset_browser_toolbar_chip_uses_segmented_selected_style_without_tab_indicator() {
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
    node.focused = false;
    node.action_id = "workbench.asset.kind_filter.set".into();
    node.button_style =
        resolved_button_style(PALETTE.accent, PALETTE.focus_ring, PALETTE.focus_ring, 1.0);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, PALETTE.surface);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(style.glyph, PALETTE.text);
}

#[test]
fn inactive_asset_browser_toolbar_chip_keeps_toolbar_surface_clear() {
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
fn selected_asset_browser_utility_tab_keeps_transparent_underline_style() {
    let mut node = positioned_button_node(
        "AssetBrowserPreviewTabButton",
        "Preview",
        "outlined",
        12.0,
        8.0,
        72.0,
        24.0,
    );
    node.selected = true;
    node.focused = false;
    node.action_id = "workbench.asset.utility_tab.set".into();
    node.button_style =
        resolved_button_style(PALETTE.accent, PALETTE.focus_ring, PALETTE.focus_ring, 1.0);

    let style = button_style(&node, button_kind(&node));

    assert_eq!(style.surface, [0, 0, 0, 0]);
    assert_eq!(style.border, PALETTE.border);
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
fn asset_import_command_button_uses_primary_accent_fill_with_theme_foreground() {
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

    assert_eq!(style.surface, PALETTE.accent);
    assert_eq!(style.border, PALETTE.accent);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, PALETTE.shell_background);
    assert_eq!(style.glyph, PALETTE.shell_background);
}
