use crate::ui::retained_host as host_contract;

use super::super::validation_state::ProjectedValidationState;
use super::super::visual_state::ProjectedVisualState;
use super::super::visual_style::ProjectedVisualStyle;

pub(super) fn assign_visual_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    validation_state: ProjectedValidationState,
    visual_state: ProjectedVisualState,
    visual_style: ProjectedVisualStyle,
) {
    node.validation_level = validation_state.level.into();
    node.validation_message = validation_state.message.into();
    node.disabled = validation_state.disabled;

    node.checked = visual_state.checked;
    node.expanded = visual_state.expanded;
    node.focused = visual_state.focused;
    node.hovered = visual_state.hovered;
    node.pressed = visual_state.pressed;
    node.dragging = visual_state.dragging;
    node.enter_pressed = visual_state.enter_pressed;
    node.state_layer_enabled = visual_state.state_layer_enabled;
    node.state_layer_color = visual_state.state_layer_color;
    node.ripple_enabled = visual_state.ripple_enabled;
    node.ripple_pressed_x = visual_state.ripple_pressed_x;
    node.ripple_pressed_y = visual_state.ripple_pressed_y;
    node.ripple_unclipped = visual_state.ripple_unclipped;
    node.drop_hovered = visual_state.drop_hovered;
    node.active_drag_target = visual_state.active_drag_target;
    node.icon_color = visual_state.icon_color;
    node.icon_stroke_width = visual_state.icon_stroke_width;

    node.component_category = visual_style.component_category.into();
    node.component_layout_role = visual_style.component_layout_role.into();
    node.component_variant = visual_style.component_variant.into();
    node.surface_variant = visual_style.surface_variant.into();
    node.text_tone = visual_style.text_tone.into();
    node.button_variant = visual_style.button_variant.into();
    node.button_style = visual_style.button_style;
    node.corner_radius = visual_style.corner_radius;
    node.border_width = visual_style.border_width;
    node.elevation = visual_style.elevation;
    node.z_index = visual_style.z_index;
    node.transition_kind = visual_style.transition.kind.into();
    node.transition_in = visual_style.transition.active;
    node.transition_entered = visual_style.transition.entered;
    node.transition_progress = visual_style.transition.progress;
    node.transition_duration_ms = visual_style.transition.duration_ms;
    node.transition_easing = visual_style.transition.easing.into();
    node.transition_direction = visual_style.transition.direction.into();
}
