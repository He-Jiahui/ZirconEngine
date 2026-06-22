use std::collections::BTreeMap;

mod flags;
mod icon;
mod model;
mod ripple;
mod state_layer;

pub(super) use self::model::ProjectedVisualState;

pub(super) fn projected_visual_state(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedVisualState {
    let flags = flags::projected_interaction_flags(attributes);
    let state_layer = state_layer::projected_state_layer(attributes);
    let ripple = ripple::projected_ripple_state(attributes);
    let icon = icon::projected_icon_state(attributes);

    ProjectedVisualState {
        checked: flags.checked,
        expanded: flags.expanded,
        focused: flags.focused,
        hovered: flags.hovered,
        pressed: flags.pressed,
        dragging: flags.dragging,
        enter_pressed: flags.enter_pressed,
        drop_hovered: flags.drop_hovered,
        active_drag_target: flags.active_drag_target,
        state_layer_enabled: state_layer.enabled,
        state_layer_color: state_layer.color,
        ripple_enabled: ripple.enabled,
        ripple_pressed_x: ripple.pressed_x,
        ripple_pressed_y: ripple.pressed_y,
        ripple_unclipped: ripple.unclipped,
        icon_color: icon.color,
        icon_stroke_width: icon.stroke_width,
    }
}
