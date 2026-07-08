use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::super::super::paint_theme::METRICS;
use super::super::super::style_selector::{
    select_workbench_tooltip_style, WORKBENCH_TOOLTIP_BORDER,
};
use super::super::metrics::tooltip_metrics_from_host;
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
    assert_ne!(pressed.border, WORKBENCH_TOOLTIP_BORDER);

    node.pressed = false;
    let focused = select_workbench_tooltip_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.border, WORKBENCH_TOOLTIP_BORDER);
}

#[test]
fn workbench_tooltip_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.radius_control = 6.0;
    host.border_width = 2.0;
    host.font_body = 11.0;
    host.gap_s = 5.0;
    host.gap_m = 10.0;
    host.gap_l = 14.0;
    host.row_height = 30.0;

    let metrics = tooltip_metrics_from_host(host);

    assert_eq!(metrics.bubble_width, 120.0);
    assert_eq!(metrics.bubble_height, 56.0);
    assert_eq!(metrics.radius, 6.0);
    assert_eq!(metrics.border_width, 2.0);
    assert_eq!(metrics.shadow_offset_y, 10.0);
    assert_eq!(metrics.text_left, 10.0);
    assert_eq!(metrics.title_top, 8.0);
    assert_eq!(metrics.body_top, 28.0);
    assert_eq!(metrics.title_font_size, 15.0);
    assert_eq!(metrics.title_line_height, 19.0);
    assert_eq!(metrics.body_font_size, 13.0);
    assert_eq!(metrics.body_line_height, 17.0);
    assert_eq!(metrics.arrow_size, 10.0);
    assert_eq!(metrics.arrow_min, 5.0);
    assert_eq!(metrics.arrow_max, 18.0);
    assert_eq!(metrics.icon_size, 21.0);
    assert_eq!(metrics.icon_min, 11.0);
    assert_eq!(metrics.icon_max, 30.0);
}
