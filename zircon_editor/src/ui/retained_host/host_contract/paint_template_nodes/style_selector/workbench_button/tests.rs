use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
use palette::{
    workbench_button_command_palette_from_host, workbench_button_palette_from_host,
    workbench_button_selection_palette_from_host,
};
use zircon_runtime_interface::ui::style::{ButtonInteractionState, UiRgbaColor, UiStyleColor};

fn expected_button_palette() -> palette::WorkbenchButtonPalette {
    workbench_button_palette_from_host(current_host_palette())
}

fn expected_command_palette() -> palette::WorkbenchButtonCommandPalette {
    workbench_button_command_palette_from_host(current_host_palette())
}

fn expected_selection_palette() -> palette::WorkbenchButtonSelectionPalette {
    workbench_button_selection_palette_from_host(current_host_palette())
}

#[test]
fn workbench_button_state_palette_projects_from_host_palette() {
    let host_palette = current_host_palette();
    let button_palette = workbench_button_palette_from_host(host_palette);

    assert_eq!(button_palette.surface_base, host_palette.surface_pressed);
    assert_eq!(button_palette.surface_primary_rest, host_palette.accent);
    assert_eq!(
        button_palette.surface_primary_hover,
        host_palette.focus_ring
    );
    assert_eq!(button_palette.surface_hover, host_palette.surface_hover);
    assert_eq!(
        button_palette.surface_primary_pressed,
        host_palette.surface_selected
    );
    assert_eq!(
        button_palette.surface_secondary_pressed,
        host_palette.surface
    );
    assert_eq!(button_palette.surface_tertiary_pressed, host_palette.popup);
    assert_eq!(button_palette.surface_danger_pressed, host_palette.surface);
    assert_eq!(button_palette.transparent_surface, [0, 0, 0, 0]);
    assert_eq!(button_palette.border, host_palette.border);
    assert_eq!(button_palette.focus_border, host_palette.focus_ring);
    assert_eq!(button_palette.text, host_palette.text);
    assert_eq!(button_palette.text_muted, host_palette.text_muted);
    assert_eq!(button_palette.danger_text, host_palette.error);
    assert_eq!(
        button_palette.disabled_surface,
        host_palette.surface_disabled
    );
    assert_eq!(button_palette.disabled_border, host_palette.border_disabled);
    assert_eq!(button_palette.disabled_text, host_palette.text_disabled);
}

#[test]
fn workbench_button_command_palette_projects_from_host_palette() {
    let host_palette = current_host_palette();
    let command_palette = workbench_button_command_palette_from_host(host_palette);

    assert_eq!(
        command_palette.muted_rest_surface,
        host_palette.surface_pressed
    );
    assert_eq!(
        command_palette.muted_hot_surface,
        host_palette.surface_hover
    );
    assert_eq!(command_palette.muted_pressed_surface, host_palette.surface);
    assert_eq!(command_palette.muted_border, host_palette.border);
    assert_eq!(command_palette.muted_text, host_palette.accent);
    assert_eq!(command_palette.primary_rest_surface, host_palette.accent);
    assert_eq!(command_palette.primary_hot_surface, host_palette.focus_ring);
    assert_eq!(command_palette.primary_text, host_palette.shell_background);
}

#[test]
fn workbench_button_selection_palette_projects_from_host_palette() {
    let host_palette = current_host_palette();
    let selection_palette = workbench_button_selection_palette_from_host(host_palette);

    assert_eq!(
        selection_palette.tab_rest_surface,
        host_palette.surface_pressed
    );
    assert_eq!(
        selection_palette.tab_hot_surface,
        host_palette.surface_hover
    );
    assert_eq!(
        selection_palette.toolbar_chip_active_surface,
        host_palette.surface
    );
    assert_eq!(
        selection_palette.asset_tab_active_surface,
        host_palette.surface_pressed
    );
    assert_eq!(selection_palette.transparent_surface, [0, 0, 0, 0]);
    assert_eq!(selection_palette.border, host_palette.border);
    assert_eq!(selection_palette.text, host_palette.text);
    assert_eq!(selection_palette.text_muted, host_palette.text_muted);
}

#[test]
fn button_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.button_style.loading = true;
    node.label_brightness = 1.5;
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(41, 164, 184, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(28, 135, 152, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(8, 24, 27, 255)));

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Primary, true);
    let button_palette = expected_button_palette();

    assert_eq!(style.interaction, ButtonInteractionState::Loading);
    assert_eq!(style.surface, button_palette.disabled_surface);
    assert_eq!(style.border, button_palette.disabled_border);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, button_palette.disabled_text);
    assert_eq!(style.glyph, button_palette.disabled_text);
}

#[test]
fn primary_button_uses_the_starship_primary_surface_role() {
    let mut node = TemplatePaneNodeData {
        control_id: "WorkbenchPrimaryButton".into(),
        button_variant: "filled".into(),
        ..TemplatePaneNodeData::default()
    };
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(42, 166, 184, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(56, 189, 208, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(10, 16, 18, 255)));

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Primary, false);
    let button_palette = expected_button_palette();

    assert_eq!(style.surface, button_palette.surface_primary_rest);
    assert_eq!(style.border, button_palette.border);
    assert_eq!(style.text, button_palette.text);
    assert_eq!(style.glyph, button_palette.text);
}

#[test]
fn primary_button_hover_uses_the_brighter_primary_surface_role() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchPrimaryButton".into(),
        button_variant: "filled".into(),
        hovered: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Primary, false);
    let button_palette = expected_button_palette();

    assert_eq!(style.interaction, ButtonInteractionState::Hover);
    assert_eq!(style.surface, button_palette.surface_primary_hover);
    assert_eq!(style.border, button_palette.border);
    assert_eq!(style.text, button_palette.text);
}

#[test]
fn secondary_button_dynamic_states_ignore_normal_declared_chrome() {
    let surface = UiStyleColor::Rgba(UiRgbaColor::from_u8(81, 88, 94, 255));
    let border = UiStyleColor::Rgba(UiRgbaColor::from_u8(109, 116, 122, 255));
    let button_palette = expected_button_palette();

    let mut normal = TemplatePaneNodeData {
        control_id: "WorkbenchSecondaryButton".into(),
        ..TemplatePaneNodeData::default()
    };
    normal.button_style.element.background_color = Some(surface.clone());
    normal.button_style.element.border_color = Some(border.clone());
    let normal_style =
        select_workbench_button_style(&normal, WorkbenchButtonKind::Secondary, false);

    let mut hovered = TemplatePaneNodeData {
        control_id: "WorkbenchSecondaryButton".into(),
        ..TemplatePaneNodeData::default()
    };
    hovered.hovered = true;
    hovered.button_style.element.background_color = Some(surface.clone());
    hovered.button_style.element.border_color = Some(border.clone());
    let hovered_style =
        select_workbench_button_style(&hovered, WorkbenchButtonKind::Secondary, false);

    let mut pressed = TemplatePaneNodeData {
        control_id: "WorkbenchSecondaryButton".into(),
        ..TemplatePaneNodeData::default()
    };
    pressed.pressed = true;
    pressed.button_style.element.background_color = Some(surface.clone());
    pressed.button_style.element.border_color = Some(border.clone());
    let pressed_style =
        select_workbench_button_style(&pressed, WorkbenchButtonKind::Secondary, false);

    let mut focused = TemplatePaneNodeData {
        control_id: "WorkbenchSecondaryButton".into(),
        ..TemplatePaneNodeData::default()
    };
    focused.focused = true;
    focused.button_style.element.background_color = Some(surface);
    focused.button_style.element.border_color = Some(border);
    let focused_style =
        select_workbench_button_style(&focused, WorkbenchButtonKind::Secondary, false);

    let mut selected = TemplatePaneNodeData {
        control_id: "WorkbenchSecondaryButton".into(),
        selected: true,
        ..TemplatePaneNodeData::default()
    };
    selected.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(81, 88, 94, 255)));
    selected.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(109, 116, 122, 255)));
    let selected_style =
        select_workbench_button_style(&selected, WorkbenchButtonKind::Secondary, false);

    let mut checked = TemplatePaneNodeData {
        control_id: "WorkbenchSecondaryButton".into(),
        checked: true,
        ..TemplatePaneNodeData::default()
    };
    checked.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(81, 88, 94, 255)));
    checked.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(109, 116, 122, 255)));
    let checked_style =
        select_workbench_button_style(&checked, WorkbenchButtonKind::Secondary, false);

    assert_eq!(normal_style.surface, [81, 88, 94, 255]);
    assert_eq!(normal_style.border, [109, 116, 122, 255]);
    assert_eq!(hovered_style.surface, button_palette.surface_hover);
    assert_eq!(hovered_style.border, button_palette.border);
    assert_eq!(
        pressed_style.surface,
        button_palette.surface_secondary_pressed
    );
    assert_eq!(pressed_style.border, button_palette.border);
    assert_eq!(focused_style.surface, button_palette.surface_hover);
    assert_eq!(focused_style.border, button_palette.focus_border);
    assert_eq!(selected_style.surface, button_palette.surface_hover);
    assert_eq!(selected_style.border, button_palette.focus_border);
    assert_eq!(checked_style.surface, button_palette.surface_hover);
    assert_eq!(checked_style.border, button_palette.focus_border);
}

#[test]
fn muted_prominent_command_focus_does_not_promote_hover_surface() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchModuleCompile".into(),
        action_id: "workbench.module.compile".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let command_palette = expected_command_palette();

    assert_eq!(style.surface, command_palette.muted_rest_surface);
    assert_eq!(style.border, command_palette.muted_border);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, command_palette.muted_text);
    assert_eq!(style.glyph, command_palette.muted_text);
}

#[test]
fn primary_import_command_focus_does_not_promote_hover_surface() {
    let node = TemplatePaneNodeData {
        control_id: "ImportModel".into(),
        action_id: "workbench.asset.import_model".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Primary, false);
    let command_palette = expected_command_palette();

    assert_eq!(style.surface, command_palette.primary_rest_surface);
    assert_eq!(style.border, command_palette.primary_rest_surface);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, command_palette.primary_text);
    assert_eq!(style.glyph, command_palette.primary_text);
}

#[test]
fn danger_button_keeps_neutral_chrome_and_semantic_content() {
    let mut node = TemplatePaneNodeData {
        control_id: "WorkbenchDangerButton".into(),
        button_variant: "danger".into(),
        validation_level: "danger".into(),
        ..TemplatePaneNodeData::default()
    };
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(76, 36, 39, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(235, 96, 92, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(210, 92, 86, 255)));

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Danger, false);
    let button_palette = expected_button_palette();

    assert_eq!(style.surface, button_palette.surface_base);
    assert_eq!(style.border, button_palette.border);
    assert_eq!(style.text, [210, 92, 86, 255]);
    assert_eq!(style.glyph, [210, 92, 86, 255]);
}

#[test]
fn asset_browser_utility_tab_selected_uses_underline_not_filled_pill() {
    let node = TemplatePaneNodeData {
        control_id: "AssetBrowserPreviewTabButton".into(),
        action_id: "workbench.asset.utility_tab.set".into(),
        selected: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let selection_palette = expected_selection_palette();

    assert_eq!(style.surface, selection_palette.transparent_surface);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, selection_palette.text);
    assert_eq!(style.glyph, selection_palette.text);
}

#[test]
fn asset_browser_utility_tab_focus_does_not_promote_selected_text_tone() {
    let node = TemplatePaneNodeData {
        control_id: "AssetBrowserPreviewTabButton".into(),
        action_id: "workbench.asset.utility_tab.set".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let selection_palette = expected_selection_palette();

    assert_eq!(style.surface, selection_palette.transparent_surface);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, selection_palette.text_muted);
    assert_eq!(style.glyph, selection_palette.text_muted);
}

#[test]
fn asset_browser_toolbar_chip_selected_keeps_framed_surface() {
    let node = TemplatePaneNodeData {
        control_id: "AssetBrowserViewModeList".into(),
        action_id: "workbench.asset.view_mode.set".into(),
        selected: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let selection_palette = expected_selection_palette();

    assert_eq!(style.surface, selection_palette.toolbar_chip_active_surface);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, selection_palette.text);
}

#[test]
fn asset_browser_toolbar_chip_focus_does_not_promote_selected_frame() {
    let node = TemplatePaneNodeData {
        control_id: "AssetBrowserViewModeList".into(),
        action_id: "workbench.asset.view_mode.set".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let selection_palette = expected_selection_palette();

    assert_eq!(style.surface, selection_palette.transparent_surface);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, selection_palette.text_muted);
    assert_eq!(style.glyph, selection_palette.text_muted);
}

#[test]
fn asset_browser_tab_like_focus_does_not_promote_selected_surface() {
    let node = TemplatePaneNodeData {
        control_id: "AssetBrowserFavoritesTabButton".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let selection_palette = expected_selection_palette();

    assert_eq!(style.surface, selection_palette.transparent_surface);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, selection_palette.text_muted);
    assert_eq!(style.glyph, selection_palette.text_muted);
}

#[test]
fn workbench_module_tab_focus_does_not_promote_selected_surface() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchModuleScene".into(),
        action_id: "workbench.module.scene".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let selection_palette = expected_selection_palette();

    assert_eq!(style.surface, selection_palette.transparent_surface);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, selection_palette.text_muted);
    assert_eq!(style.glyph, selection_palette.text_muted);
}

#[test]
fn page_tab_focus_does_not_promote_selected_surface() {
    let node = TemplatePaneNodeData {
        control_id: "PageTab0".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let selection_palette = expected_selection_palette();

    assert_eq!(style.surface, selection_palette.tab_rest_surface);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, selection_palette.text_muted);
    assert_eq!(style.glyph, selection_palette.text_muted);
}

#[test]
fn dock_tab_focus_does_not_promote_selected_surface() {
    let node = TemplatePaneNodeData {
        control_id: "DockTab1".into(),
        focused: true,
        ..TemplatePaneNodeData::default()
    };

    let style = select_workbench_button_style(&node, WorkbenchButtonKind::Secondary, false);
    let selection_palette = expected_selection_palette();

    assert_eq!(style.surface, selection_palette.tab_rest_surface);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, selection_palette.text_muted);
    assert_eq!(style.glyph, selection_palette.text_muted);
}
