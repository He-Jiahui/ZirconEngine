use super::selection::select_workbench_list_row_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn list_row_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.selected = true;
    node.checked = true;
    node.button_style.loading = true;
    node.value_color = Color::from_rgb_u8(53, 199, 208);
    node.icon_color = Color::from_rgb_u8(122, 230, 240);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(13, 65, 73, 255)));

    let style = select_workbench_list_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.background, None);
    assert_eq!(style.border, None);
    assert_eq!(style.text, PALETTE.text_disabled);
    assert_eq!(style.adornment, PALETTE.text_disabled);
}

#[test]
fn selected_list_row_uses_muted_selected_fill_and_neutral_outline() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;
    node.focused = true;

    let style = select_workbench_list_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, Some(PALETTE.surface_pressed));
    assert_ne!(style.background, Some(PALETTE.surface_selected));
    assert_eq!(style.border, Some(PALETTE.border));
    assert_ne!(style.border, Some(PALETTE.accent));
    assert_ne!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.border_width, 1.0);
}
