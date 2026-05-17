use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{
        UiNavigationEventKind, UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute,
    },
    tree::{UiTemplateNodeMetadata, UiTreeError},
};

use crate::ui::surface::{
    UiPropertyMutationReport, UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface,
};
use crate::ui::tree::UiRuntimeTreeAccessExt;

use super::{is_default_range_behavior, UiDefaultRangePointerActionReport};

impl UiSurface {
    pub(in crate::ui::surface::surface) fn apply_default_range_pointer_actions(
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
                let value_property = self.default_range_value_property(node_id)?;
                self.capture_pointer(node_id)?;
                self.push_pointer_component_events(
                    events,
                    node_id,
                    UiEventKind::DragBegin,
                    UiComponentEvent::BeginDrag {
                        property: value_property,
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
                let value_property = self.default_range_value_property(node_id)?;
                if let Some(delta) = self.apply_default_range_value_from_point(
                    node_id,
                    &value_property,
                    route.point,
                    events,
                    UiPointerComponentEventReason::DirectBinding,
                )? {
                    self.push_pointer_component_events(
                        events,
                        node_id,
                        UiEventKind::DragUpdate,
                        UiComponentEvent::DragDelta {
                            property: value_property,
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
                let value_property = self.default_range_value_property(node_id)?;
                if self
                    .apply_default_range_value_from_point(
                        node_id,
                        &value_property,
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
                        property: value_property,
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
        property: &str,
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
        if self.apply_default_range_value(node_id, property, next_value, events, reason)? {
            Ok(Some(next_value - current_value))
        } else {
            Ok(None)
        }
    }

    fn apply_default_range_value(
        &mut self,
        node_id: UiNodeId,
        property: &str,
        next_value: f64,
        events: &mut Vec<UiPointerComponentEvent>,
        reason: UiPointerComponentEventReason,
    ) -> Result<bool, UiTreeError> {
        let report = self.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            property,
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
                property: property.to_string(),
                value: UiValue::Float(next_value),
            },
            reason,
        )?;
        Ok(true)
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
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata)
            || !is_default_range_behavior(metadata)
        {
            return Ok(None);
        }
        let Some(frame) = self.arranged_tree.get(node_id).map(|node| node.frame) else {
            return Ok(None);
        };
        if frame.width <= f32::EPSILON {
            return Ok(None);
        }

        let min = self
            .default_range_numeric_value(node_id, metadata, widget_min_property(metadata))
            .unwrap_or(0.0);
        let max = self
            .default_range_numeric_value(node_id, metadata, widget_max_property(metadata))
            .unwrap_or(1.0);
        if max <= min {
            return Ok(None);
        }
        let fraction =
            ((f64::from(point.x) - f64::from(frame.x)) / f64::from(frame.width)).clamp(0.0, 1.0);
        let raw_value = min + (max - min) * fraction;
        let stepped_value = self
            .default_range_numeric_value(node_id, metadata, widget_step_property(metadata))
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
        let property = self.default_range_value_property(node_id)?;
        let report = self.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            property,
            UiValue::Float(next_value),
        ))?;
        Ok(Some((report, next_value)))
    }

    pub(in crate::ui::surface::surface) fn mutate_default_range_endpoint_value(
        &mut self,
        node_id: UiNodeId,
        use_max: bool,
    ) -> Result<Option<(UiPropertyMutationReport, f64)>, UiTreeError> {
        let Some(next_value) = self.default_range_endpoint_value(node_id, use_max)? else {
            return Ok(None);
        };
        let property = self.default_range_value_property(node_id)?;
        let report = self.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            property,
            UiValue::Float(next_value),
        ))?;
        Ok(Some((report, next_value)))
    }

    fn default_range_endpoint_value(
        &self,
        node_id: UiNodeId,
        use_max: bool,
    ) -> Result<Option<f64>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata)
            || !is_default_range_behavior(metadata)
        {
            return Ok(None);
        }
        let min = self
            .default_range_numeric_value(node_id, metadata, widget_min_property(metadata))
            .unwrap_or(0.0);
        let max = self
            .default_range_numeric_value(node_id, metadata, widget_max_property(metadata))
            .unwrap_or(1.0);
        if max <= min {
            return Ok(None);
        }
        Ok(Some(if use_max { max } else { min }))
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
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata)
            || !is_default_range_behavior(metadata)
        {
            return Ok(None);
        }
        let min = self
            .default_range_numeric_value(node_id, metadata, widget_min_property(metadata))
            .unwrap_or(0.0);
        let max = self
            .default_range_numeric_value(node_id, metadata, widget_max_property(metadata))
            .unwrap_or(1.0);
        if max <= min {
            return Ok(None);
        }
        let current = self
            .default_range_numeric_value(node_id, metadata, widget_value_property(metadata))
            .or_else(|| metadata.widget.value.as_ref().and_then(UiValue::as_f64))
            .unwrap_or(min)
            .clamp(min, max);
        let step = self
            .default_range_numeric_value(node_id, metadata, widget_step_property(metadata))
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
        if !is_default_range_behavior(metadata) {
            return Ok(None);
        }
        let min = self
            .default_range_numeric_value(node_id, metadata, widget_min_property(metadata))
            .unwrap_or(0.0);
        let max = self
            .default_range_numeric_value(node_id, metadata, widget_max_property(metadata))
            .unwrap_or(1.0);
        Ok(self
            .default_range_numeric_value(node_id, metadata, widget_value_property(metadata))
            .or_else(|| metadata.widget.value.as_ref().and_then(UiValue::as_f64))
            .map(|value| {
                if max > min {
                    value.clamp(min, max)
                } else {
                    value
                }
            }))
    }

    fn default_range_numeric_value(
        &self,
        node_id: UiNodeId,
        metadata: &UiTemplateNodeMetadata,
        property: &str,
    ) -> Option<f64> {
        f64_attribute_value(&metadata.attributes, property).or_else(|| {
            self.component_states
                .get(node_id)
                .and_then(|state| state.value(property))
                .and_then(UiValue::as_f64)
        })
    }

    fn is_default_range_node(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(false);
        };
        Ok(self.widget_interaction_enabled(node_id, node, metadata)
            && is_default_range_behavior(metadata))
    }

    fn default_range_value_property(&self, node_id: UiNodeId) -> Result<String, UiTreeError> {
        let metadata = self.template_metadata(node_id)?;
        Ok(widget_value_property(metadata).to_string())
    }
}

pub(in crate::ui::surface::surface) enum UiDefaultRangeNavigationAction {
    Step(f64),
    Minimum,
    Maximum,
}

pub(in crate::ui::surface::surface) fn range_navigation_action(
    kind: UiNavigationEventKind,
) -> Option<UiDefaultRangeNavigationAction> {
    match kind {
        UiNavigationEventKind::Right | UiNavigationEventKind::Up => {
            Some(UiDefaultRangeNavigationAction::Step(1.0))
        }
        UiNavigationEventKind::Left | UiNavigationEventKind::Down => {
            Some(UiDefaultRangeNavigationAction::Step(-1.0))
        }
        UiNavigationEventKind::Home => Some(UiDefaultRangeNavigationAction::Minimum),
        UiNavigationEventKind::End => Some(UiDefaultRangeNavigationAction::Maximum),
        _ => None,
    }
}

fn widget_value_property(metadata: &UiTemplateNodeMetadata) -> &str {
    metadata.widget.value_property.as_deref().unwrap_or("value")
}

fn widget_min_property(metadata: &UiTemplateNodeMetadata) -> &str {
    metadata.widget.min_property.as_deref().unwrap_or("min")
}

fn widget_max_property(metadata: &UiTemplateNodeMetadata) -> &str {
    metadata.widget.max_property.as_deref().unwrap_or("max")
}

fn widget_step_property(metadata: &UiTemplateNodeMetadata) -> &str {
    metadata.widget.step_property.as_deref().unwrap_or("step")
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
