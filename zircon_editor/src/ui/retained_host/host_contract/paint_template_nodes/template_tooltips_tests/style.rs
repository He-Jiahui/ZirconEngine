use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::super::style_selector::{
    select_workbench_tooltip_style, WORKBENCH_TOOLTIP_BORDER,
};
use super::support::tooltip_node;

#[test]
fn workbench_tooltip_style_uses_shared_state_priority() {
    let mut node = tooltip_node();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.disabled = true;

    let disabled = select_workbench_tooltip_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_ne!(disabled.border, WORKBENCH_TOOLTIP_BORDER);

    node.disabled = false;
    let pressed = select_workbench_tooltip_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);

    node.pressed = false;
    let focused = select_workbench_tooltip_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
}
