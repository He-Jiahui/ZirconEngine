use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{project_host_palette, PALETTE};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn selection_controls_enabled_labels_keep_readable_muted_tone() {
    let node = TemplatePaneNodeData::default();

    for kind in [
        WorkbenchSelectionControlKind::Checkbox,
        WorkbenchSelectionControlKind::Radio,
    ] {
        let style = select_workbench_selection_control_style(&node, kind);

        assert_eq!(style.state, UiPainterResolvedState::Normal);
        assert_eq!(style.text, PALETTE.text);
        assert_eq!(style.label, PALETTE.text_muted);
        assert_ne!(style.label, PALETTE.text_disabled);
    }
}

#[test]
fn selection_control_palette_projects_from_host_appearance_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.popup = UiRgbaColor::from_u8(8, 10, 12, 255);
    tokens.palette.surface_selected = UiRgbaColor::from_u8(28, 44, 50, 255);
    tokens.palette.surface_hover = UiRgbaColor::from_u8(38, 46, 52, 255);
    tokens.palette.track = UiRgbaColor::from_u8(18, 22, 26, 255);
    tokens.palette.text_secondary = UiRgbaColor::from_u8(145, 154, 162, 255);
    tokens.palette.focus_ring = UiRgbaColor::from_u8(66, 116, 128, 255);

    let palette = super::palette::workbench_selection_control_palette_from_host(
        project_host_palette(&tokens),
    );

    assert_eq!(palette.mark_idle_fill, [8, 10, 12, 255]);
    assert_eq!(palette.checkbox_checked_fill, [28, 44, 50, 255]);
    assert_eq!(palette.toggle_checked_surface, [28, 44, 50, 255]);
    assert_eq!(palette.toggle_hover_surface, [38, 46, 52, 255]);
    assert_eq!(palette.toggle_track, [18, 22, 26, 255]);
    assert_eq!(palette.text_muted, [145, 154, 162, 255]);
    assert_eq!(palette.focus_ring, [66, 116, 128, 255]);
}

#[test]
fn selection_controls_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.checked = true;
    node.selected = true;
    node.hovered = true;
    node.pressed = true;
    node.drop_hovered = true;
    node.button_style.loading = true;
    node.value_color = Color::from_rgb_u8(67, 216, 226);
    node.label_color = Color::from_rgb_u8(131, 141, 148);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(32, 159, 168, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(34, 161, 170, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(255, 255, 255, 255)));

    for kind in [
        WorkbenchSelectionControlKind::Checkbox,
        WorkbenchSelectionControlKind::Radio,
        WorkbenchSelectionControlKind::Toggle,
    ] {
        let style = select_workbench_selection_control_style(&node, kind);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.surface, PALETTE.surface_disabled);
        assert_eq!(style.border, PALETTE.border_disabled);
        assert_eq!(style.thumb, PALETTE.text_disabled);
        assert_eq!(style.accent, PALETTE.text_disabled);
        assert_eq!(style.text, PALETTE.text_disabled);
        assert_eq!(style.label, PALETTE.text_disabled);
    }
}

#[test]
fn selection_controls_checked_state_uses_low_emphasis_markers() {
    let mut node = TemplatePaneNodeData::default();
    node.checked = true;
    node.selected = true;
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(53, 199, 208, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(49, 191, 201, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(255, 255, 255, 255)));

    let checkbox =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Checkbox);
    let radio =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Radio);
    let toggle =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Toggle);

    assert_eq!(checkbox.surface, PALETTE.surface_selected);
    assert_eq!(checkbox.border, PALETTE.accent);
    assert_eq!(radio.surface, PALETTE.surface_pressed);
    assert_eq!(radio.border, PALETTE.separator_strong);
    assert_eq!(toggle.surface, PALETTE.surface_selected);
    assert_eq!(toggle.border, PALETTE.separator_strong);
    assert_eq!(toggle.thumb, PALETTE.text_muted);
}

#[test]
fn focused_unchecked_toggle_keeps_track_surface_with_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let toggle =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Toggle);

    assert_eq!(toggle.state, UiPainterResolvedState::Focused);
    assert_eq!(toggle.surface, PALETTE.track);
    assert_eq!(toggle.border, PALETTE.focus_ring);
    assert_eq!(toggle.thumb, PALETTE.text_muted);
}

#[test]
fn focused_hovered_unchecked_toggle_keeps_hover_surface_with_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    node.hovered = true;

    let toggle =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Toggle);

    assert_eq!(toggle.state, UiPainterResolvedState::Focused);
    assert_eq!(toggle.surface, PALETTE.surface_hover);
    assert_eq!(toggle.border, PALETTE.focus_ring);
}

#[test]
fn focused_unchecked_checkbox_keeps_idle_surface_with_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let checkbox =
        select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Checkbox);

    assert_eq!(checkbox.state, UiPainterResolvedState::Focused);
    assert_eq!(checkbox.surface, PALETTE.popup);
    assert_eq!(checkbox.border, PALETTE.focus_ring);
}

#[test]
fn unchecked_hover_and_press_keep_idle_borders_without_focus_outline() {
    for mut node in [
        TemplatePaneNodeData {
            hovered: true,
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            pressed: true,
            ..TemplatePaneNodeData::default()
        },
    ] {
        node.focus_visible = false;
        node.focus_visible_known = true;

        let checkbox = select_workbench_selection_control_style(
            &node,
            WorkbenchSelectionControlKind::Checkbox,
        );
        let radio =
            select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Radio);
        let toggle =
            select_workbench_selection_control_style(&node, WorkbenchSelectionControlKind::Toggle);

        assert_eq!(checkbox.border, PALETTE.separator_strong);
        assert_eq!(radio.border, PALETTE.separator_strong);
        assert_eq!(toggle.border, PALETTE.border);
        assert_ne!(checkbox.border, PALETTE.focus_ring);
        assert_ne!(toggle.border, PALETTE.focus_ring);
    }
}

#[test]
fn unchecked_drop_target_keeps_the_dedicated_focus_outline() {
    let node = TemplatePaneNodeData {
        drop_hovered: true,
        ..TemplatePaneNodeData::default()
    };

    for kind in [
        WorkbenchSelectionControlKind::Checkbox,
        WorkbenchSelectionControlKind::Radio,
        WorkbenchSelectionControlKind::Toggle,
    ] {
        assert_eq!(
            select_workbench_selection_control_style(&node, kind).border,
            PALETTE.focus_ring
        );
    }
}
