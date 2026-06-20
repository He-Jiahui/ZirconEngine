use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn tree_row_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.selected = true;
    node.checked = true;
    node.button_style.loading = true;

    let style = select_workbench_tree_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.background, None);
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.text, PALETTE.text_disabled);
    assert_eq!(style.icon, PALETTE.text_disabled);
    assert_eq!(style.secondary, PALETTE.text_disabled);
    assert_eq!(style.action, PALETTE.text_disabled);
}
