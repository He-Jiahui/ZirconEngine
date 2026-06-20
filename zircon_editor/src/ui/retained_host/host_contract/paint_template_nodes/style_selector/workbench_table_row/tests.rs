use super::selection::select_workbench_table_row_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn table_row_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchTableSelected".into();
    node.hovered = true;
    node.selected = true;
    node.button_style.loading = true;
    node.value_color = Color::from_rgb_u8(170, 181, 186);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(13, 65, 73, 255)));

    let style = select_workbench_table_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.background, PALETTE.surface_disabled);
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.action, PALETTE.text_disabled);
    assert_eq!(style.text_for_cell(0), PALETTE.text_disabled);
    assert_eq!(style.text_for_cell(3), PALETTE.text_disabled);
}
