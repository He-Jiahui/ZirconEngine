use super::super::super::super::data::{TemplatePaneMenuItemData, TemplatePaneOptionData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::{popup_menu_row_style, popup_option_row_style};
use super::support::{menu_item, option};

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
    assert_eq!(disabled.selection_mark, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);

    let focused = popup_option_row_style(&TemplatePaneOptionData {
        focused: true,
        ..focused_selected
    });
    assert_eq!(
        focused.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Focused
    );
    assert_eq!(focused.background, Some(PALETTE.surface_selected));
    assert_eq!(focused.selection_mark, Some(PALETTE.focus_ring));
    assert_eq!(focused.text, PALETTE.focus_ring);

    let checked = popup_menu_row_style(&checked_pressed);
    assert_eq!(
        checked.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
    );
    assert_eq!(checked.background, Some(PALETTE.surface_selected));
    assert_eq!(checked.selection_mark, Some(PALETTE.focus_ring));
    assert_eq!(checked.adornment, PALETTE.focus_ring);
}

#[test]
fn popup_row_style_selector_matches_runtime_extract_state_matrix_for_projected_rows() {
    use zircon_runtime_interface::ui::style::UiPainterResolvedState;

    let selected = popup_option_row_style(&option("selected", true, false, false, false));
    assert_eq!(selected.state, UiPainterResolvedState::Selected);
    assert_eq!(selected.background, Some(PALETTE.surface_selected));
    assert_eq!(selected.selection_mark, Some(PALETTE.focus_ring));

    let focused = popup_option_row_style(&TemplatePaneOptionData {
        focused: true,
        hovered: true,
        ..option("focused", false, false, false, false)
    });
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, Some(PALETTE.surface_selected));
    assert_eq!(focused.text, PALETTE.focus_ring);

    let disabled = popup_option_row_style(&TemplatePaneOptionData {
        selected: true,
        disabled: true,
        ..option("disabled", false, false, false, false)
    });
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.selection_mark, None);
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
    assert_eq!(loading.selection_mark, None);
    assert_eq!(loading.text, PALETTE.text_disabled);

    let raw_loading_menu = popup_menu_row_style(&menu_item(
        "Archive|loading,checked,hovered",
        true,
        false,
        true,
    ));
    assert_eq!(raw_loading_menu.state, UiPainterResolvedState::Loading);
    assert_eq!(raw_loading_menu.background, None);
    assert_eq!(raw_loading_menu.selection_mark, None);
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
    assert_eq!(projected_loading_menu.selection_mark, None);
    assert_eq!(projected_loading_menu.adornment, PALETTE.text_disabled);
}
