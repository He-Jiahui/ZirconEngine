use super::super::super::super::paint_theme::PALETTE;
use super::super::{AlertTone, select_workbench_alert_style, select_workbench_toast_style};
use super::support::positioned_alert_node;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn workbench_toast_style_uses_shared_state_priority() {
    let mut node = positioned_alert_node(
        "WorkbenchToastRoot",
        "Operation completed successfully",
        "success",
        8.0,
        8.0,
        280.0,
        32.0,
    );
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.disabled = true;

    let disabled = select_workbench_toast_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.action, PALETTE.text_disabled);

    node.disabled = false;
    let pressed = select_workbench_toast_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.border, PALETTE.focus_ring);

    node.pressed = false;
    let focused = select_workbench_toast_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.border, PALETTE.focus_ring);

    node.popup_open = true;
    let focused_open = select_workbench_toast_style(&node);
    assert_eq!(focused_open.state, UiPainterResolvedState::Focused);
    assert_eq!(focused_open.border, PALETTE.focus_ring);

    node.focused = false;
    let open = select_workbench_toast_style(&node);
    assert_eq!(open.state, UiPainterResolvedState::Open);
    assert_eq!(open.border, PALETTE.focus_ring);
}

#[test]
fn workbench_alert_style_uses_shared_state_priority() {
    let mut node = positioned_alert_node(
        "WorkbenchWarningAlert",
        "Warning Alert",
        "warning",
        8.0,
        8.0,
        160.0,
        32.0,
    );
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.disabled = true;

    let disabled = select_workbench_alert_style(&node, AlertTone::Warning);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.text, PALETTE.text_disabled);
    assert_eq!(disabled.surface, PALETTE.surface_disabled);

    node.disabled = false;
    let pressed = select_workbench_alert_style(&node, AlertTone::Warning);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.border, PALETTE.focus_ring);

    node.pressed = false;
    let focused = select_workbench_alert_style(&node, AlertTone::Warning);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.border, PALETTE.warning);
    assert_ne!(focused.border, PALETTE.focus_ring);
}
