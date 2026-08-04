use zircon_runtime::ui::surface::{UiPropertyMutationRequest, UiSurface};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
    surface::{UiPointerActivationPhase, UiPointerButton, UiPointerEventKind, UiPointerRoute},
    tree::UiTemplateNodeMetadata,
    widget::UiWidgetBehavior,
};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn pointer_pressed_target(&self) -> Option<UiNodeId> {
        self.template_surface.surface.focus.pressed
    }

    pub(crate) fn pointer_focused_target(&self) -> Option<UiNodeId> {
        self.template_surface.surface.focus.focused
    }

    pub(crate) fn refresh_pointer_hover_feedback(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if route.activation_phase != UiPointerActivationPhase::Hover {
            return Ok(false);
        }

        let mut touched_hover_candidate = false;
        for node_id in &route.left {
            touched_hover_candidate |= self.set_pointer_hover_feedback(*node_id, false)?;
        }
        for node_id in &route.entered {
            touched_hover_candidate |= self.set_pointer_hover_feedback(*node_id, true)?;
        }
        touched_hover_candidate |= self.refresh_workbench_icon_tooltip(route)?;

        self.refresh_dirty_pointer_feedback(touched_hover_candidate)
    }

    pub(crate) fn refresh_pointer_press_feedback(
        &mut self,
        route: &UiPointerRoute,
        pressed_before_route: Option<UiNodeId>,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let mut touched_press_candidate =
            self.dismiss_workbench_icon_tooltip_on_primary_press(route)?;
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                if let Some(previous) =
                    pressed_before_route.filter(|previous| Some(*previous) != route.target)
                {
                    touched_press_candidate |= self.set_pointer_press_feedback(previous, false)?;
                }
                if let Some(target) = route.target {
                    touched_press_candidate |= self.set_pointer_press_feedback(target, true)?;
                }
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(pressed) = route.pressed {
                    touched_press_candidate |= self.set_pointer_press_feedback(pressed, false)?;
                }
            }
            _ => return Ok(false),
        }

        self.refresh_dirty_pointer_feedback(touched_press_candidate)
    }

    pub(crate) fn refresh_pointer_range_feedback(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let is_initial_range_press =
            route.activation_phase == UiPointerActivationPhase::PrimaryPress;
        if let Some(update) = pointer_range_update(&self.template_surface.surface, route) {
            let _ =
                self.template_surface
                    .surface
                    .mutate_property(UiPropertyMutationRequest::new(
                        update.node_id,
                        update.value_property,
                        UiValue::Float(update.value),
                    ))?;
            return self.refresh_dirty_pointer_feedback(true);
        }

        let runtime_owns_range_drag = !is_initial_range_press
            && pointer_range_target(&self.template_surface.surface, route).is_some();
        self.refresh_dirty_pointer_feedback(runtime_owns_range_drag)
    }

    pub(crate) fn refresh_text_input_pointer_feedback(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !route_focuses_text_input_control(&self.template_surface.surface, route) {
            return Ok(false);
        }

        self.refresh_dirty_pointer_feedback(true)
    }

    pub(crate) fn refresh_pointer_focus_feedback(
        &mut self,
        route: &UiPointerRoute,
        focused_before_route: Option<UiNodeId>,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if route.activation_phase != UiPointerActivationPhase::PrimaryPress {
            return Ok(false);
        }
        if route.button != Some(UiPointerButton::Primary) {
            return Ok(false);
        }
        if route.focused == focused_before_route {
            return Ok(false);
        }

        self.refresh_dirty_pointer_feedback(true)
    }

    fn refresh_dirty_pointer_feedback(
        &mut self,
        touched_candidate: bool,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !touched_candidate || !self.template_surface.surface.dirty_flags().any() {
            return Ok(false);
        }

        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    fn set_pointer_hover_feedback(
        &mut self,
        node_id: UiNodeId,
        hovered: bool,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let accepts_hover_feedback = self
            .template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .is_some_and(metadata_accepts_pointer_hover_feedback);
        if !accepts_hover_feedback {
            return Ok(false);
        }

        self.mutate_node_bool(node_id, "hovered", hovered)?;
        Ok(true)
    }

    fn set_pointer_press_feedback(
        &mut self,
        node_id: UiNodeId,
        pressed: bool,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let accepts_press_feedback = self
            .template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .is_some_and(metadata_accepts_pointer_press_feedback);
        if !accepts_press_feedback {
            return Ok(false);
        }

        self.mutate_node_bool(node_id, "pressed", pressed)?;
        Ok(true)
    }
}

struct PointerRangeUpdate {
    node_id: UiNodeId,
    value_property: String,
    value: f64,
}

const RANGE_MIN_PROPERTY: &str = "range_min";

const POINTER_HOVER_FEEDBACK_CLASSES: &[&str] = &[
    "workbench-icon-button",
    "workbench-rail-button",
    "workbench-toolbar-button",
    "workbench-control-button",
    "workbench-tab",
    "workbench-tree-item",
    "workbench-list-row",
    "workbench-table-row",
    "workbench-field",
    "workbench-component-field",
    "workbench-dropdown",
    "workbench-slider",
    "workbench-check",
    "workbench-radio",
    "workbench-toggle",
    "workbench-segmented-control",
];

fn pointer_range_update(surface: &UiSurface, route: &UiPointerRoute) -> Option<PointerRangeUpdate> {
    if route.activation_phase != UiPointerActivationPhase::PrimaryPress {
        return None;
    }
    let node_id = pointer_range_target(surface, route)?;
    let node = surface.tree.nodes.get(&node_id)?;
    let metadata = node.template_metadata.as_ref()?;
    let frame = surface.arranged_tree.get(node_id).map(|node| node.frame)?;
    let value_property = surface
        .input
        .pointer_drag_property(node_id)
        .unwrap_or_else(|| metadata.widget.value_property.as_deref().unwrap_or("value"));
    let value = pointer_range_value_from_point(metadata, frame, route.point, value_property)?;
    Some(PointerRangeUpdate {
        node_id,
        value_property: value_property.to_string(),
        value,
    })
}

fn pointer_range_target(surface: &UiSurface, route: &UiPointerRoute) -> Option<UiNodeId> {
    let node_id = match route.activation_phase {
        UiPointerActivationPhase::PrimaryPress
            if route.button == Some(UiPointerButton::Primary) =>
        {
            route.target
        }
        UiPointerActivationPhase::Hover if route.kind == UiPointerEventKind::Move => {
            route.captured.or(route.pressed)
        }
        UiPointerActivationPhase::PrimaryRelease
            if route.button == Some(UiPointerButton::Primary) =>
        {
            route.captured.or(route.pressed).or(route.click_target)
        }
        _ => None,
    }?;

    surface
        .tree
        .nodes
        .get(&node_id)
        .and_then(|node| node.template_metadata.as_ref())
        .filter(|metadata| metadata_accepts_pointer_range_feedback(metadata))
        .map(|_| node_id)
}

fn pointer_range_value_from_point(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    point: UiPoint,
    value_property: &str,
) -> Option<f64> {
    if frame.width <= f32::EPSILON {
        return None;
    }
    let min = metadata_f64(
        metadata,
        metadata.widget.min_property.as_deref().unwrap_or("min"),
    )
    .unwrap_or(0.0);
    let max = metadata_f64(
        metadata,
        metadata.widget.max_property.as_deref().unwrap_or("max"),
    )
    .unwrap_or(1.0);
    if max <= min {
        return None;
    }

    let fraction =
        ((f64::from(point.x) - f64::from(frame.x)) / f64::from(frame.width)).clamp(0.0, 1.0);
    let raw_value = min + (max - min) * fraction;
    let value = metadata_f64(
        metadata,
        metadata.widget.step_property.as_deref().unwrap_or("step"),
    )
    .filter(|step| *step > 0.0)
    .map(|step| min + ((raw_value - min) / step).round() * step)
    .unwrap_or(raw_value);
    let (lower_bound, upper_bound) = pointer_range_bounds(metadata, value_property, min, max);
    let value = value.clamp(lower_bound, upper_bound);
    Some(value)
}

fn pointer_range_bounds(
    metadata: &UiTemplateNodeMetadata,
    value_property: &str,
    min: f64,
    max: f64,
) -> (f64, f64) {
    if metadata.component != "RangeSlider" {
        return (min, max);
    }
    if value_property == RANGE_MIN_PROPERTY {
        let upper_bound = metadata_f64(
            metadata,
            metadata.widget.value_property.as_deref().unwrap_or("value"),
        )
        .unwrap_or(max)
        .clamp(min, max);
        (min, upper_bound)
    } else if let Some(range_min) = metadata_f64(metadata, RANGE_MIN_PROPERTY) {
        (range_min.clamp(min, max), max)
    } else {
        (min, max)
    }
}

fn route_focuses_text_input_control(surface: &UiSurface, route: &UiPointerRoute) -> bool {
    if route.activation_phase != UiPointerActivationPhase::PrimaryPress {
        return false;
    }
    if route.button != Some(UiPointerButton::Primary) {
        return false;
    }
    let Some(focused) = route.focused else {
        return false;
    };
    if !route.bubbled.contains(&focused) {
        return false;
    }

    surface
        .tree
        .nodes
        .get(&focused)
        .and_then(|node| node.template_metadata.as_ref())
        .is_some_and(metadata_is_text_input_control)
}

fn metadata_is_text_input_control(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::TextInput
        || metadata.component == "InputField"
        || metadata.component == "TextField"
        || (metadata.classes.iter().any(|class| {
            matches!(
                class.as_str(),
                "workbench-field" | "workbench-component-field"
            )
        }) && metadata_bool(metadata, "editable_text"))
}

fn metadata_accepts_pointer_hover_feedback(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata_interaction_enabled(metadata)
        && (metadata_bool(metadata, "input_hoverable")
            || metadata_has_pointer_feedback_class(metadata))
}

fn metadata_accepts_pointer_press_feedback(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata_interaction_enabled(metadata)
        && !metadata_is_text_input_control(metadata)
        && (metadata_bool(metadata, "input_clickable")
            || metadata_has_pointer_feedback_class(metadata))
}

fn metadata_accepts_pointer_range_feedback(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata_interaction_enabled(metadata)
        && (metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::Range
            || matches!(metadata.component.as_str(), "RangeField" | "Slider")
            || metadata
                .classes
                .iter()
                .any(|class| class == "workbench-slider"))
}

fn metadata_interaction_enabled(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata
        .attributes
        .get("enabled")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true)
        && !metadata_bool(metadata, "disabled")
}

fn metadata_has_pointer_feedback_class(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.classes.iter().any(|class| {
        POINTER_HOVER_FEEDBACK_CLASSES
            .iter()
            .any(|candidate| class.as_str() == *candidate)
    })
}

fn metadata_bool(metadata: &UiTemplateNodeMetadata, key: &str) -> bool {
    metadata
        .attributes
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn metadata_f64(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f64> {
    match metadata.attributes.get(key)? {
        toml::Value::Float(value) => Some(*value),
        toml::Value::Integer(value) => Some(*value as f64),
        toml::Value::String(value) => value.parse::<f64>().ok(),
        _ => None,
    }
}
