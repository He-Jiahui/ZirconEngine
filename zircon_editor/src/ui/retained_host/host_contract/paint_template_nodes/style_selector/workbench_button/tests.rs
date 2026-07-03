use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use palette::{DANGER_BORDER, DANGER_SURFACE};
use zircon_runtime_interface::ui::style::{ButtonInteractionState, UiRgbaColor, UiStyleColor};

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

    assert_eq!(style.interaction, ButtonInteractionState::Loading);
    assert_eq!(style.surface, PALETTE.surface_disabled);
    assert_eq!(style.border, PALETTE.border_disabled);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, PALETTE.text_disabled);
    assert_eq!(style.glyph, PALETTE.text_disabled);
}

#[test]
fn primary_button_owns_low_emphasis_chrome_tokens() {
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

    assert_eq!(style.surface, PRIMARY_SURFACE);
    assert_eq!(style.border, OUTLINED_BORDER);
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(style.glyph, PALETTE.text);
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

    assert_eq!(style.surface, DANGER_SURFACE);
    assert_eq!(style.border, DANGER_BORDER);
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

    assert_eq!(style.surface, [0, 0, 0, 0]);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, PALETTE.text);
    assert_eq!(style.glyph, PALETTE.text);
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

    assert_eq!(style.surface, PALETTE.surface);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text, PALETTE.text);
}
