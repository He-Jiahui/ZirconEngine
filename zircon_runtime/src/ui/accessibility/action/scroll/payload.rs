use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionRequest,
    event_ui::UiNodeId,
    layout::{UiAxis, UiContainerKind, UiPoint},
};

use crate::ui::surface::UiSurface;

pub(super) fn scroll_to_offset(
    surface: &UiSurface,
    target: UiNodeId,
    request: &UiAccessibilityActionRequest,
) -> Option<f64> {
    request
        .numeric_value
        .or_else(|| {
            request
                .value
                .as_deref()
                .and_then(|value| value.parse::<f64>().ok())
        })
        .filter(|value| value.is_finite())
        .or_else(|| {
            request
                .scroll_offset
                .and_then(|offset| scroll_axis_offset(surface, target, offset))
        })
}

fn scroll_axis_offset(surface: &UiSurface, target: UiNodeId, offset: UiPoint) -> Option<f64> {
    let axis = surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| match node.container {
            UiContainerKind::ScrollableBox(config) => Some(config.axis),
            _ => None,
        })
        .unwrap_or_default();
    let value = match axis {
        UiAxis::Horizontal => offset.x,
        UiAxis::Vertical => offset.y,
    };
    value.is_finite().then_some(f64::from(value))
}
