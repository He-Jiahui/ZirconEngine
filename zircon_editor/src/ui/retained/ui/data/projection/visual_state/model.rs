use crate::ui::retained_host::primitives::Color;

// Projection bundle for interactive visual-state fields.
pub(in super::super) struct ProjectedVisualState {
    pub(in super::super) checked: bool,
    pub(in super::super) expanded: bool,
    pub(in super::super) focused: bool,
    pub(in super::super) hovered: bool,
    pub(in super::super) pressed: bool,
    pub(in super::super) dragging: bool,
    pub(in super::super) enter_pressed: bool,
    pub(in super::super) drop_hovered: bool,
    pub(in super::super) active_drag_target: bool,
    pub(in super::super) state_layer_enabled: bool,
    pub(in super::super) state_layer_color: Color,
    pub(in super::super) ripple_enabled: bool,
    pub(in super::super) ripple_pressed_x: f32,
    pub(in super::super) ripple_pressed_y: f32,
    pub(in super::super) ripple_unclipped: bool,
    pub(in super::super) icon_color: Color,
    pub(in super::super) icon_stroke_width: f32,
}
