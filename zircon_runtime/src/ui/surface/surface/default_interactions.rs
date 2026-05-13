use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiComponentEventKind, UiValue},
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{
        UiNavigationEventKind, UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute,
    },
    template::UiBindingRef,
    tree::UiTreeError,
};

use crate::ui::surface::{
    UiPropertyMutationReport, UiPropertyMutationRequest, UiPropertyMutationStatus,
};
use crate::ui::tree::UiRuntimeTreeAccessExt;

use super::UiSurface;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct UiDefaultRangePointerActionReport {
    pub handled_by: Option<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
    pub released_capture: Option<UiNodeId>,
    pub damage_node: Option<UiNodeId>,
}

impl UiSurface {
    pub(super) fn apply_default_pointer_component_actions(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<(), UiTreeError> {
        if route.activation_phase != UiPointerActivationPhase::PrimaryRelease {
            return Ok(());
        }
        let Some(node_id) = route.click_target else {
            return Ok(());
        };
        let Some(next_checked) = self.default_toggle_next_checked(node_id)? else {
            if self.apply_default_expanded_component_action(node_id, events)? {
                return Ok(());
            }
            if self.apply_default_popup_component_action(node_id, events)? {
                return Ok(());
            }
            return Ok(());
        };

        let report = self.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "checked",
            UiValue::Bool(next_checked),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(());
        }

        self.push_pointer_component_events(
            events,
            node_id,
            UiEventKind::Change,
            UiComponentEvent::ValueChanged {
                property: "checked".to_string(),
                value: UiValue::Bool(next_checked),
            },
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(())
    }

    fn apply_default_expanded_component_action(
        &mut self,
        node_id: UiNodeId,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<bool, UiTreeError> {
        let Some(next_expanded) = self.default_expanded_next(node_id)? else {
            return Ok(false);
        };
        let report = self.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "expanded",
            UiValue::Bool(next_expanded),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(false);
        }

        self.push_pointer_component_events(
            events,
            node_id,
            UiEventKind::Toggle,
            UiComponentEvent::ToggleExpanded {
                expanded: next_expanded,
            },
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(true)
    }

    fn apply_default_popup_component_action(
        &mut self,
        node_id: UiNodeId,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<bool, UiTreeError> {
        let Some(next_popup_open) = self.default_popup_open_next(node_id)? else {
            return Ok(false);
        };
        let report = self.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "popup_open",
            UiValue::Bool(next_popup_open),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(false);
        }

        let event = if next_popup_open {
            UiComponentEvent::OpenPopup
        } else {
            UiComponentEvent::ClosePopup
        };
        self.push_pointer_component_events_for_component_event_kind(
            events,
            node_id,
            UiEventKind::Click,
            event,
            UiPointerComponentEventReason::DefaultClick,
        )?;
        Ok(true)
    }

    pub(super) fn apply_default_range_pointer_actions(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<UiDefaultRangePointerActionReport, UiTreeError> {
        let mut action = UiDefaultRangePointerActionReport::default();
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                let Some(node_id) = route.target else {
                    return Ok(action);
                };
                if !self.is_default_range_node(node_id)? {
                    return Ok(action);
                }
                self.capture_pointer(node_id)?;
                self.push_pointer_component_events(
                    events,
                    node_id,
                    UiEventKind::DragBegin,
                    UiComponentEvent::BeginDrag {
                        property: "value".to_string(),
                    },
                    UiPointerComponentEventReason::PressBegin,
                )?;
                action.handled_by = Some(node_id);
                action.captured_by = Some(node_id);
                action.damage_node = Some(node_id);
            }
            UiPointerActivationPhase::Hover if matches!(route.kind, UiPointerEventKind::Move) => {
                let Some(node_id) = route.captured else {
                    return Ok(action);
                };
                if !self.is_default_range_node(node_id)? {
                    return Ok(action);
                }
                if let Some(delta) = self.apply_default_range_value_from_point(
                    node_id,
                    route.point,
                    events,
                    UiPointerComponentEventReason::DirectBinding,
                )? {
                    self.push_pointer_component_events(
                        events,
                        node_id,
                        UiEventKind::DragUpdate,
                        UiComponentEvent::DragDelta {
                            property: "value".to_string(),
                            delta,
                        },
                        UiPointerComponentEventReason::DirectBinding,
                    )?;
                    action.damage_node = Some(node_id);
                }
                action.handled_by = Some(node_id);
            }
            UiPointerActivationPhase::PrimaryRelease => {
                let Some(node_id) = route.captured.or(route.click_target) else {
                    return Ok(action);
                };
                if !self.is_default_range_node(node_id)? {
                    return Ok(action);
                }
                if self
                    .apply_default_range_value_from_point(
                        node_id,
                        route.point,
                        events,
                        UiPointerComponentEventReason::DefaultClick,
                    )?
                    .is_some()
                {
                    action.damage_node = Some(node_id);
                }
                self.push_pointer_component_events(
                    events,
                    node_id,
                    UiEventKind::DragEnd,
                    UiComponentEvent::EndDrag {
                        property: "value".to_string(),
                    },
                    UiPointerComponentEventReason::PressEnd,
                )?;
                action.handled_by = Some(node_id);
                action.released_capture = Some(node_id);
            }
            _ => {}
        }
        Ok(action)
    }

    fn apply_default_range_value_from_point(
        &mut self,
        node_id: UiNodeId,
        point: UiPoint,
        events: &mut Vec<UiPointerComponentEvent>,
        reason: UiPointerComponentEventReason,
    ) -> Result<Option<f64>, UiTreeError> {
        let Some(next_value) = self.default_range_click_value(node_id, point)? else {
            return Ok(None);
        };
        let current_value = self
            .default_range_current_value(node_id)?
            .unwrap_or(next_value);
        if self.apply_default_range_value(node_id, next_value, events, reason)? {
            Ok(Some(next_value - current_value))
        } else {
            Ok(None)
        }
    }

    fn apply_default_range_value(
        &mut self,
        node_id: UiNodeId,
        next_value: f64,
        events: &mut Vec<UiPointerComponentEvent>,
        reason: UiPointerComponentEventReason,
    ) -> Result<bool, UiTreeError> {
        let report = self.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "value",
            UiValue::Float(next_value),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(false);
        }
        self.push_pointer_component_events(
            events,
            node_id,
            UiEventKind::Change,
            UiComponentEvent::ValueChanged {
                property: "value".to_string(),
                value: UiValue::Float(next_value),
            },
            reason,
        )?;
        Ok(true)
    }

    fn default_toggle_next_checked(&self, node_id: UiNodeId) -> Result<Option<bool>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.state_flags.enabled {
            return Ok(None);
        }
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !is_default_toggle_component(&metadata.component) {
            return Ok(None);
        }
        Ok(Some(!node.state_flags.checked))
    }

    fn default_expanded_next(&self, node_id: UiNodeId) -> Result<Option<bool>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.state_flags.enabled {
            return Ok(None);
        }
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !is_default_expanded_component(&metadata.component) {
            return Ok(None);
        }
        let expanded = self
            .component_states
            .get(node_id)
            .map(|state| state.flags.expanded)
            .or_else(|| bool_attribute_value(&metadata.attributes, "expanded"))
            .unwrap_or_else(|| default_expanded_component_state(&metadata.component));
        Ok(Some(!expanded))
    }

    fn default_popup_open_next(&self, node_id: UiNodeId) -> Result<Option<bool>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.state_flags.enabled {
            return Ok(None);
        }
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !is_default_popup_component(&metadata.component) {
            return Ok(None);
        }
        let popup_open = self
            .component_states
            .get(node_id)
            .map(|state| state.flags.popup_open)
            .or_else(|| bool_attribute_value(&metadata.attributes, "popup_open"))
            .or_else(|| bool_attribute_value(&metadata.attributes, "open"))
            .unwrap_or(false);
        Ok(Some(!popup_open))
    }

    fn default_range_click_value(
        &self,
        node_id: UiNodeId,
        point: UiPoint,
    ) -> Result<Option<f64>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.state_flags.enabled {
            return Ok(None);
        }
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !is_default_range_component(&metadata.component) {
            return Ok(None);
        }
        let Some(frame) = self.arranged_tree.get(node_id).map(|node| node.frame) else {
            return Ok(None);
        };
        if frame.width <= f32::EPSILON {
            return Ok(None);
        }

        let min = f64_attribute_value(&metadata.attributes, "min").unwrap_or(0.0);
        let max = f64_attribute_value(&metadata.attributes, "max").unwrap_or(1.0);
        if max <= min {
            return Ok(None);
        }
        let fraction =
            ((f64::from(point.x) - f64::from(frame.x)) / f64::from(frame.width)).clamp(0.0, 1.0);
        let raw_value = min + (max - min) * fraction;
        let stepped_value = f64_attribute_value(&metadata.attributes, "step")
            .filter(|step| *step > 0.0)
            .map(|step| min + ((raw_value - min) / step).round() * step)
            .unwrap_or(raw_value)
            .clamp(min, max);

        Ok(Some(stepped_value))
    }

    pub(crate) fn mutate_default_range_step_value(
        &mut self,
        node_id: UiNodeId,
        direction: f64,
    ) -> Result<Option<(UiPropertyMutationReport, f64)>, UiTreeError> {
        let Some(next_value) = self.default_range_step_value(node_id, direction)? else {
            return Ok(None);
        };
        let report = self.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "value",
            UiValue::Float(next_value),
        ))?;
        Ok(Some((report, next_value)))
    }

    fn default_range_step_value(
        &self,
        node_id: UiNodeId,
        direction: f64,
    ) -> Result<Option<f64>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.state_flags.enabled {
            return Ok(None);
        }
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !is_default_range_component(&metadata.component) {
            return Ok(None);
        }
        let min = f64_attribute_value(&metadata.attributes, "min").unwrap_or(0.0);
        let max = f64_attribute_value(&metadata.attributes, "max").unwrap_or(1.0);
        if max <= min {
            return Ok(None);
        }
        let current = f64_attribute_value(&metadata.attributes, "value")
            .unwrap_or(min)
            .clamp(min, max);
        let step = f64_attribute_value(&metadata.attributes, "step")
            .filter(|step| *step > 0.0)
            .unwrap_or_else(|| ((max - min) / 100.0).max(f64::EPSILON));
        Ok(Some((current + direction.signum() * step).clamp(min, max)))
    }

    fn default_range_current_value(&self, node_id: UiNodeId) -> Result<Option<f64>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !is_default_range_component(&metadata.component) {
            return Ok(None);
        }
        let min = f64_attribute_value(&metadata.attributes, "min").unwrap_or(0.0);
        let max = f64_attribute_value(&metadata.attributes, "max").unwrap_or(1.0);
        Ok(
            f64_attribute_value(&metadata.attributes, "value").map(|value| {
                if max > min {
                    value.clamp(min, max)
                } else {
                    value
                }
            }),
        )
    }

    fn is_default_range_node(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(false);
        };
        Ok(node.state_flags.enabled && is_default_range_component(&metadata.component))
    }

    pub(super) fn uses_typed_default_click_action(
        &self,
        node_id: UiNodeId,
    ) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.state_flags.enabled {
            return Ok(false);
        }
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(false);
        };
        Ok(is_default_toggle_component(&metadata.component)
            || is_default_expanded_component(&metadata.component)
            || is_default_popup_component(&metadata.component)
            || is_default_range_component(&metadata.component))
    }

    fn push_pointer_component_events_for_component_event_kind(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        node_id: UiNodeId,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        reason: UiPointerComponentEventReason,
    ) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(());
        };
        let control_id = metadata
            .control_id
            .as_deref()
            .unwrap_or(node.node_path.0.as_str());
        let component_event_kind = event.kind();
        for binding in metadata.bindings.iter().filter(|binding| {
            binding.event == event_kind
                && binding_targets_component_event(binding, component_event_kind)
        }) {
            events.push(UiPointerComponentEvent::new(
                &self.tree.tree_id,
                node_id,
                control_id,
                binding.id.as_str(),
                event_kind,
                event.clone(),
                reason,
            ));
        }
        Ok(())
    }
}

pub(super) fn range_navigation_direction(kind: UiNavigationEventKind) -> Option<f64> {
    match kind {
        UiNavigationEventKind::Right | UiNavigationEventKind::Up => Some(1.0),
        UiNavigationEventKind::Left | UiNavigationEventKind::Down => Some(-1.0),
        _ => None,
    }
}

fn is_default_toggle_component(component: &str) -> bool {
    matches!(
        component,
        "Toggle" | "Checkbox" | "CheckBox" | "Switch" | "ToggleButton"
    )
}

fn is_default_expanded_component(component: &str) -> bool {
    matches!(
        component,
        "Group" | "Foldout" | "InspectorSection" | "TreeRow" | "TreeView"
    )
}

fn default_expanded_component_state(component: &str) -> bool {
    matches!(component, "Group" | "InspectorSection" | "TreeView")
}

fn is_default_popup_component(component: &str) -> bool {
    matches!(
        component,
        "Dropdown"
            | "ComboBox"
            | "EnumField"
            | "FlagsField"
            | "SearchSelect"
            | "ContextActionMenu"
            | "Popup"
    )
}

fn is_default_range_component(component: &str) -> bool {
    matches!(component, "RangeField" | "Slider")
}

fn bool_attribute_value(
    values: &std::collections::BTreeMap<String, toml::Value>,
    key: &str,
) -> Option<bool> {
    values.get(key).and_then(toml::Value::as_bool)
}

fn f64_attribute_value(
    values: &std::collections::BTreeMap<String, toml::Value>,
    key: &str,
) -> Option<f64> {
    match values.get(key)? {
        toml::Value::Float(value) => Some(*value),
        toml::Value::Integer(value) => Some(*value as f64),
        toml::Value::String(value) => value.parse::<f64>().ok(),
        _ => None,
    }
}

fn binding_targets_component_event(
    binding: &UiBindingRef,
    event_kind: UiComponentEventKind,
) -> bool {
    let Some(token) = component_event_kind_token(event_kind) else {
        return false;
    };
    binding.id.contains(token)
        || binding
            .route
            .as_deref()
            .is_some_and(|route| route.contains(token))
        || binding.action.as_ref().is_some_and(|action| {
            action
                .route
                .as_deref()
                .is_some_and(|route| route.contains(token))
                || action
                    .action
                    .as_deref()
                    .is_some_and(|action| action.contains(token))
        })
}

fn component_event_kind_token(event_kind: UiComponentEventKind) -> Option<&'static str> {
    match event_kind {
        UiComponentEventKind::ValueChanged => Some("ValueChanged"),
        UiComponentEventKind::Commit => Some("Commit"),
        UiComponentEventKind::Focus => Some("Focus"),
        UiComponentEventKind::Hover => Some("Hover"),
        UiComponentEventKind::Press => Some("Press"),
        UiComponentEventKind::BeginDrag => Some("BeginDrag"),
        UiComponentEventKind::DragDelta => Some("DragDelta"),
        UiComponentEventKind::LargeDragDelta => Some("LargeDragDelta"),
        UiComponentEventKind::EndDrag => Some("EndDrag"),
        UiComponentEventKind::DropHover => Some("DropHover"),
        UiComponentEventKind::ActiveDragTarget => Some("ActiveDragTarget"),
        UiComponentEventKind::OpenPopup => Some("OpenPopup"),
        UiComponentEventKind::OpenPopupAt => Some("OpenPopupAt"),
        UiComponentEventKind::ClosePopup => Some("ClosePopup"),
        UiComponentEventKind::SelectOption => Some("SelectOption"),
        UiComponentEventKind::ToggleExpanded => Some("ToggleExpanded"),
        UiComponentEventKind::AddElement => Some("AddElement"),
        UiComponentEventKind::SetElement => Some("SetElement"),
        UiComponentEventKind::RemoveElement => Some("RemoveElement"),
        UiComponentEventKind::MoveElement => Some("MoveElement"),
        UiComponentEventKind::AddMapEntry => Some("AddMapEntry"),
        UiComponentEventKind::SetMapEntry => Some("SetMapEntry"),
        UiComponentEventKind::RenameMapKey => Some("RenameMapKey"),
        UiComponentEventKind::RemoveMapEntry => Some("RemoveMapEntry"),
        UiComponentEventKind::DropReference => Some("DropReference"),
        UiComponentEventKind::ClearReference => Some("ClearReference"),
        UiComponentEventKind::LocateReference => Some("LocateReference"),
        UiComponentEventKind::OpenReference => Some("OpenReference"),
        UiComponentEventKind::SetVisibleRange => Some("SetVisibleRange"),
        UiComponentEventKind::SetPage => Some("SetPage"),
        UiComponentEventKind::SetWorldTransform => Some("SetWorldTransform"),
        UiComponentEventKind::SetWorldSurface => Some("SetWorldSurface"),
    }
}
