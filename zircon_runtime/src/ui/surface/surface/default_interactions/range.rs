use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiBindingUpdateReport, UiEventKind},
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

use super::{is_default_range_behavior, UiDefaultRangePointerActionReport};

struct UiDefaultRangeValueUpdate {
    active_property: String,
    active_value: f64,
    side_effect: Option<(String, f64)>,
}

struct UiDefaultRangeValueApplyReport {
    property: String,
    delta: f64,
}

impl UiSurface {
    pub(in crate::ui::surface::surface) fn apply_default_range_pointer_actions(
        &mut self,
        route: &UiPointerRoute,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
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
                let value_property =
                    self.default_range_value_property_for_point(node_id, route.point)?;
                self.capture_pointer(node_id)?;
                let drag = self.input.begin_pointer_drag_with_property(
                    node_id,
                    route.point,
                    Some(value_property.clone()),
                );
                self.push_pointer_component_events_with_drag_metrics(
                    events,
                    node_id,
                    UiEventKind::DragBegin,
                    UiComponentEvent::BeginDrag {
                        property: value_property,
                    },
                    UiPointerComponentEventReason::PressBegin,
                    Some(drag),
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
                let value_property =
                    self.default_range_drag_value_property(node_id, route.point)?;
                let drag = self.input.update_pointer_drag(node_id, route.point);
                if let Some(update) = self.apply_default_range_value_from_point(
                    node_id,
                    &value_property,
                    route.point,
                    events,
                    binding_reports,
                    UiPointerComponentEventReason::DirectBinding,
                )? {
                    self.push_pointer_component_events_with_drag_metrics(
                        events,
                        node_id,
                        UiEventKind::DragUpdate,
                        UiComponentEvent::DragDelta {
                            property: update.property,
                            delta: update.delta,
                        },
                        UiPointerComponentEventReason::DirectBinding,
                        Some(drag),
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
                let value_property =
                    self.default_range_drag_value_property(node_id, route.point)?;
                let mut end_property = value_property.clone();
                if let Some(update) = self.apply_default_range_value_from_point(
                    node_id,
                    &value_property,
                    route.point,
                    events,
                    binding_reports,
                    UiPointerComponentEventReason::DefaultClick,
                )? {
                    end_property = update.property;
                    action.damage_node = Some(node_id);
                }
                let drag = self.input.end_pointer_drag(node_id, route.point);
                self.push_pointer_component_events_with_drag_metrics(
                    events,
                    node_id,
                    UiEventKind::DragEnd,
                    UiComponentEvent::EndDrag {
                        property: end_property,
                    },
                    UiPointerComponentEventReason::PressEnd,
                    Some(drag),
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
        binding_reports: &mut Vec<UiBindingUpdateReport>,
        reason: UiPointerComponentEventReason,
    ) -> Result<Option<UiDefaultRangeValueApplyReport>, UiTreeError> {
        let Some(raw_next_value) = self.default_range_click_value(node_id, point)? else {
            return Ok(None);
        };
        let update =
            self.default_range_value_update_for_property(node_id, property, raw_next_value)?;
        let current_value = self
            .default_range_current_value(node_id, &update.active_property)?
            .unwrap_or(update.active_value);
        let mut changed = false;
        if let Some((property, value)) = update.side_effect.as_ref() {
            changed |= self.apply_default_range_value(
                node_id,
                property,
                *value,
                events,
                binding_reports,
                reason,
            )?;
        }
        changed |= self.apply_default_range_value(
            node_id,
            &update.active_property,
            update.active_value,
            events,
            binding_reports,
            reason,
        )?;
        if !changed {
            return Ok(None);
        }
        self.input
            .set_pointer_drag_property(node_id, Some(update.active_property.clone()));
        Ok(Some(UiDefaultRangeValueApplyReport {
            property: update.active_property,
            delta: update.active_value - current_value,
        }))
    }

    fn apply_default_range_value(
        &mut self,
        node_id: UiNodeId,
        property: &str,
        next_value: f64,
        events: &mut Vec<UiPointerComponentEvent>,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
        reason: UiPointerComponentEventReason,
    ) -> Result<bool, UiTreeError> {
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            node_id,
            property,
            UiValue::Float(next_value),
        ))?;
        if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(false);
        }
        binding_reports.push(report.binding);
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
        let Some(frame) = self.arranged_node(node_id).map(|node| node.frame) else {
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
        self.mutate_default_range_step_value_with_source_kind(
            node_id,
            direction,
            UiBindingSourceKind::WidgetBehavior,
        )
    }

    pub(crate) fn mutate_default_range_step_value_with_source_kind(
        &mut self,
        node_id: UiNodeId,
        direction: f64,
        source_kind: UiBindingSourceKind,
    ) -> Result<Option<(UiPropertyMutationReport, f64)>, UiTreeError> {
        let Some(next_value) = self.default_range_step_value(node_id, direction)? else {
            return Ok(None);
        };
        let property = self.default_range_value_property(node_id)?;
        let report = self.mutate_property(
            UiPropertyMutationRequest::widget_behavior(
                node_id,
                property,
                UiValue::Float(next_value),
            )
            .with_binding_source_kind(source_kind),
        )?;
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
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
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

    fn default_range_current_value(
        &self,
        node_id: UiNodeId,
        property: &str,
    ) -> Result<Option<f64>, UiTreeError> {
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
            .default_range_numeric_value(node_id, metadata, property)
            .or_else(|| {
                (property == widget_value_property(metadata))
                    .then(|| metadata.widget.value.as_ref().and_then(UiValue::as_f64))
                    .flatten()
            })
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

    fn default_range_value_property_for_point(
        &self,
        node_id: UiNodeId,
        point: UiPoint,
    ) -> Result<String, UiTreeError> {
        let metadata = self.template_metadata(node_id)?;
        if metadata.component != "RangeSlider" {
            return Ok(widget_value_property(metadata).to_string());
        }
        let Some(next_value) = self.default_range_click_value(node_id, point)? else {
            return Ok(widget_value_property(metadata).to_string());
        };
        let upper_property = widget_value_property(metadata);
        let Some(upper_value) = self.default_range_current_value(node_id, upper_property)? else {
            return Ok(upper_property.to_string());
        };
        let Some(lower_value) = self.default_range_current_value(node_id, "range_min")? else {
            return Ok(upper_property.to_string());
        };
        if (next_value - lower_value).abs() < (next_value - upper_value).abs() {
            Ok("range_min".to_string())
        } else {
            Ok(upper_property.to_string())
        }
    }

    fn default_range_drag_value_property(
        &self,
        node_id: UiNodeId,
        point: UiPoint,
    ) -> Result<String, UiTreeError> {
        if let Some(property) = self.input.pointer_drag_property(node_id) {
            return Ok(property.to_string());
        }
        self.default_range_value_property_for_point(node_id, point)
    }

    fn default_range_constrained_value_for_property(
        &self,
        node_id: UiNodeId,
        property: &str,
        value: f64,
    ) -> Result<f64, UiTreeError> {
        let metadata = self.template_metadata(node_id)?;
        let min = self
            .default_range_numeric_value(node_id, metadata, widget_min_property(metadata))
            .unwrap_or(0.0);
        let max = self
            .default_range_numeric_value(node_id, metadata, widget_max_property(metadata))
            .unwrap_or(1.0);
        let mut constrained = if max > min {
            value.clamp(min, max)
        } else {
            value
        };
        if metadata.component != "RangeSlider" {
            return Ok(constrained);
        }

        let upper_property = widget_value_property(metadata);
        if property == "range_min" {
            if let Some(upper_value) = self.default_range_current_value(node_id, upper_property)? {
                constrained = constrained.min(upper_value);
            }
        } else if property == upper_property {
            if let Some(lower_value) = self.default_range_current_value(node_id, "range_min")? {
                constrained = constrained.max(lower_value);
            }
        }
        Ok(constrained)
    }

    fn default_range_value_update_for_property(
        &self,
        node_id: UiNodeId,
        property: &str,
        value: f64,
    ) -> Result<UiDefaultRangeValueUpdate, UiTreeError> {
        let metadata = self.template_metadata(node_id)?;
        let min = self
            .default_range_numeric_value(node_id, metadata, widget_min_property(metadata))
            .unwrap_or(0.0);
        let max = self
            .default_range_numeric_value(node_id, metadata, widget_max_property(metadata))
            .unwrap_or(1.0);
        let constrained = if max > min {
            value.clamp(min, max)
        } else {
            value
        };
        if metadata.component == "RangeSlider" && !range_slider_disable_swap(metadata) {
            let upper_property = widget_value_property(metadata);
            if property == "range_min" {
                if let Some(upper_value) =
                    self.default_range_current_value(node_id, upper_property)?
                {
                    if constrained > upper_value {
                        return Ok(UiDefaultRangeValueUpdate {
                            active_property: upper_property.to_string(),
                            active_value: constrained,
                            side_effect: Some(("range_min".to_string(), upper_value)),
                        });
                    }
                }
            } else if property == upper_property {
                if let Some(lower_value) = self.default_range_current_value(node_id, "range_min")? {
                    if constrained < lower_value {
                        return Ok(UiDefaultRangeValueUpdate {
                            active_property: "range_min".to_string(),
                            active_value: constrained,
                            side_effect: Some((upper_property.to_string(), lower_value)),
                        });
                    }
                }
            }
        }

        Ok(UiDefaultRangeValueUpdate {
            active_property: property.to_string(),
            active_value: self.default_range_constrained_value_for_property(
                node_id,
                property,
                constrained,
            )?,
            side_effect: None,
        })
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

fn range_slider_disable_swap(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute_value(&metadata.attributes, "disable_swap")
        .or_else(|| bool_attribute_value(&metadata.attributes, "disableSwap"))
        .unwrap_or(true)
}

fn bool_attribute_value(
    values: &std::collections::BTreeMap<String, toml::Value>,
    key: &str,
) -> Option<bool> {
    values.get(key)?.as_bool()
}
