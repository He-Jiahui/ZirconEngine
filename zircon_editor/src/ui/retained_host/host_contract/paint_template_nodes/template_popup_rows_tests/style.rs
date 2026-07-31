use super::super::super::super::data::{TemplatePaneMenuItemData, TemplatePaneOptionData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::super::paint_theme::{HostControlMetrics, METRICS};
use super::super::metrics::workbench_popup_row_metrics_from_host;
use super::super::{popup_menu_row_style, popup_option_row_style};
use super::support::{menu_item, option};

#[test]
fn popup_row_metrics_project_from_host_control_metrics() {
    let host = HostControlMetrics {
        radius_control: 6.0,
        border_width: 2.0,
        font_body: 11.0,
        line_height_ratio: 1.25,
        font_large: 15.0,
        input_pad: [9.0, 10.0, 2.0, 5.0],
        gap_m: 10.0,
        gap_l: 13.0,
        ..METRICS
    };

    let metrics = workbench_popup_row_metrics_from_host(host);

    assert_eq!(metrics.font_size, 11.0);
    assert!((metrics.line_height - 13.75).abs() < 0.001);
    assert_eq!(metrics.text_left, 9.0);
    assert_eq!(metrics.text_right, 10.0);
    assert_eq!(metrics.text_top, 2.0);
    assert_eq!(metrics.text_bottom, 5.0);
    assert_eq!(metrics.shortcut_left_ratio, 0.58);
    assert_eq!(metrics.shortcut_width_ratio, 0.38);
    assert_eq!(metrics.surface_radius, 4.0);
    assert_eq!(metrics.outline_width, 2.0);
    assert_eq!(metrics.adornment_right, 13.0);
    assert_eq!(metrics.adornment_size, 15.0);
    assert_eq!(metrics.adornment_reserved_width, 35.0);
}

#[test]
fn popup_row_style_selector_resolves_state_priority_for_options_and_menu_items() {
    let disabled_pressed = TemplatePaneOptionData {
        pressed: true,
        ..option("disabled", false, false, false, true)
    };
    let focused_selected = option("selected", true, false, false, false);
    let checked_pressed = TemplatePaneMenuItemData {
        pressed: true,
        ..menu_item("Checked", true, false, false)
    };

    let disabled = popup_option_row_style(&disabled_pressed);
    assert_eq!(
        disabled.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Disabled
    );
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.outline, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);

    let focused = popup_option_row_style(&TemplatePaneOptionData {
        focused: true,
        ..focused_selected
    });
    assert_eq!(
        focused.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Focused
    );
    assert_eq!(focused.background, Some(PALETTE.surface_pressed));
    assert_ne!(focused.background, Some(PALETTE.surface_selected));
    assert_eq!(focused.outline, Some(PALETTE.border));
    assert_ne!(focused.outline, Some(PALETTE.accent));
    assert_eq!(focused.text, PALETTE.text);

    let checked = popup_menu_row_style(&checked_pressed);
    assert_eq!(
        checked.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
    );
    assert_eq!(checked.background, Some(PALETTE.surface_pressed));
    assert_ne!(checked.background, Some(PALETTE.surface_selected));
    assert_eq!(checked.outline, Some(PALETTE.border));
    assert_ne!(checked.outline, Some(PALETTE.accent));
    assert_eq!(checked.adornment, PALETTE.text);
}

#[test]
fn popup_row_style_selector_matches_runtime_extract_state_matrix_for_projected_rows() {
    use zircon_runtime_interface::ui::style::UiPainterResolvedState;

    let selected = popup_option_row_style(&option("selected", true, false, false, false));
    assert_eq!(selected.state, UiPainterResolvedState::Selected);
    assert_eq!(selected.background, Some(PALETTE.surface_pressed));
    assert_ne!(selected.background, Some(PALETTE.surface_selected));
    assert_eq!(selected.outline, Some(PALETTE.border));
    assert_ne!(selected.outline, Some(PALETTE.accent));

    let focused_only = popup_option_row_style(&TemplatePaneOptionData {
        focused: true,
        ..option("focused", false, false, false, false)
    });
    assert_eq!(focused_only.state, UiPainterResolvedState::Focused);
    assert_eq!(focused_only.background, None);
    assert_eq!(focused_only.outline, Some(PALETTE.border));
    assert_eq!(focused_only.text, PALETTE.text);

    let focused = popup_option_row_style(&TemplatePaneOptionData {
        focused: true,
        hovered: true,
        ..option("focused", false, false, false, false)
    });
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, Some(PALETTE.surface_hover));
    assert_eq!(focused.outline, Some(PALETTE.border));
    assert_eq!(focused.text, PALETTE.text);

    let disabled = popup_option_row_style(&TemplatePaneOptionData {
        selected: true,
        disabled: true,
        ..option("disabled", false, false, false, false)
    });
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.outline, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);

    let loading = popup_option_row_style(&TemplatePaneOptionData {
        selected: true,
        special: true,
        hovered: true,
        pressed: true,
        loading: true,
        ..option("loading", false, false, false, false)
    });
    assert_eq!(loading.state, UiPainterResolvedState::Loading);
    assert_eq!(loading.background, None);
    assert_eq!(loading.outline, None);
    assert_eq!(loading.text, PALETTE.text_disabled);

    let raw_loading_menu = popup_menu_row_style(&menu_item(
        "Archive|loading,checked,hovered",
        true,
        false,
        true,
    ));
    assert_eq!(raw_loading_menu.state, UiPainterResolvedState::Loading);
    assert_eq!(raw_loading_menu.background, None);
    assert_eq!(raw_loading_menu.outline, None);
    assert_eq!(raw_loading_menu.text, PALETTE.text_disabled);

    let projected_loading_menu = popup_menu_row_style(&TemplatePaneMenuItemData {
        checked: true,
        hovered: true,
        pressed: true,
        loading: true,
        ..menu_item("Archive", false, false, false)
    });
    assert_eq!(
        projected_loading_menu.state,
        UiPainterResolvedState::Loading
    );
    assert_eq!(projected_loading_menu.background, None);
    assert_eq!(projected_loading_menu.outline, None);
    assert_eq!(projected_loading_menu.adornment, PALETTE.text_disabled);
}
