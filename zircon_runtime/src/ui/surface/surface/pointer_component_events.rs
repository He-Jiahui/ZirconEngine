use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiPointerComponentEvent, UiPointerComponentEventReason, UiPointerDispatchResult,
        UiPointerEvent, UiTemplateActionInvocation,
    },
    event_ui::UiNodeId,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute},
    template::{UiBindingExpression, UiBindingRef},
    tree::{UiDirtyFlags, UiTreeError},
};

use super::UiSurface;

impl UiSurface {
    pub(super) fn apply_pointer_component_state(
        &mut self,
        route: &UiPointerRoute,
        focus_before_dispatch: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        let mut changed_node_ids = BTreeSet::new();
        for node_id in &route.entered {
            if self.component_states.set_hovered(*node_id, true) {
                changed_node_ids.insert(*node_id);
            }
        }
        for node_id in &route.left {
            if self.component_states.set_hovered(*node_id, false) {
                changed_node_ids.insert(*node_id);
            }
        }
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                if let Some(target) = route.target {
                    if self.node_interaction_enabled(target)?
                        && self.component_states.set_pressed(target, true)
                    {
                        changed_node_ids.insert(target);
                    }
                }
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(pressed) = route.pressed {
                    if self.component_states.set_pressed(pressed, false) {
                        changed_node_ids.insert(pressed);
                    }
                }
            }
            _ => {}
        }
        if matches!(route.kind, UiPointerEventKind::Cancel) {
            if let Some(pressed) = route.pressed {
                if self.component_states.set_pressed(pressed, false) {
                    changed_node_ids.insert(pressed);
                }
            }
        }
        if focus_before_dispatch != self.focus.focused {
            if let Some(previous) = focus_before_dispatch {
                if self.component_states.set_focused(previous, false) {
                    changed_node_ids.insert(previous);
                }
            }
            if let Some(current) = self.focus.focused {
                if self.component_states.set_focused(current, true) {
                    changed_node_ids.insert(current);
                }
            }
        }
        self.mark_component_states_render_dirty(&changed_node_ids)
    }

    pub(super) fn apply_pointer_transient_state_dirty(
        &mut self,
        route: &UiPointerRoute,
        pressed_before_dispatch: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                if let Some(previous) =
                    pressed_before_dispatch.filter(|previous| Some(*previous) != route.target)
                {
                    self.set_node_pressed_dirty(previous, false)?;
                }
                if let Some(target) = route.target {
                    if self.node_interaction_enabled(target)? {
                        self.set_node_pressed_dirty(target, true)?;
                    }
                }
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(pressed) = route.pressed {
                    self.set_node_pressed_dirty(pressed, false)?;
                }
            }
            _ => {}
        }
        if matches!(route.kind, UiPointerEventKind::Cancel) {
            if let Some(pressed) = route.pressed {
                self.set_node_pressed_dirty(pressed, false)?;
            }
        }
        Ok(())
    }

    fn set_node_pressed_dirty(
        &mut self,
        node_id: UiNodeId,
        pressed: bool,
    ) -> Result<(), UiTreeError> {
        let component_state_changed = self.component_states.set_pressed(node_id, pressed);
        let state_flags_changed = {
            let node = self
                .tree
                .nodes
                .get_mut(&node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if node.state_flags.pressed == pressed {
                false
            } else {
                node.state_flags.pressed = pressed;
                true
            }
        };
        if component_state_changed {
            self.mark_component_state_render_dirty(node_id)?;
        } else if state_flags_changed {
            self.mark_node_dirty(
                node_id,
                UiDirtyFlags {
                    render: true,
                    ..UiDirtyFlags::default()
                },
            )?;
        }
        Ok(())
    }

    pub(crate) fn mark_component_state_render_dirty(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<(), UiTreeError> {
        self.mark_component_states_render_dirty(&BTreeSet::from([node_id]))
    }

    pub(crate) fn mark_component_states_render_dirty(
        &mut self,
        changed_node_ids: &BTreeSet<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        let mut descendant_affecting_node_ids = BTreeSet::new();
        for node_id in changed_node_ids {
            if self
                .runtime_style
                .node_state_can_affect_descendants(&self.tree, *node_id)?
            {
                descendant_affecting_node_ids.insert(*node_id);
            }
        }
        let style_roots =
            minimal_changed_subtree_roots(&self.tree, &descendant_affecting_node_ids)?;
        let style_root_ids = style_roots.iter().copied().collect::<BTreeSet<_>>();
        for root_id in &style_roots {
            let _ = self.apply_runtime_state_style_subtree(*root_id, true)?;
        }
        for node_id in changed_node_ids {
            if !node_is_covered_by_roots(&self.tree, *node_id, &style_root_ids)? {
                let _ = self.apply_runtime_state_style_node(*node_id, true)?;
            }
        }
        for node_id in changed_node_ids {
            self.mark_node_dirty(
                *node_id,
                UiDirtyFlags {
                    render: true,
                    ..UiDirtyFlags::default()
                },
            )?;
        }
        Ok(())
    }

    pub(super) fn pointer_component_events(
        &self,
        route: &UiPointerRoute,
        event: &UiPointerEvent,
    ) -> Result<Vec<UiPointerComponentEvent>, UiTreeError> {
        let mut events = Vec::new();
        for node_id in &route.entered {
            self.push_pointer_component_events(
                &mut events,
                *node_id,
                UiEventKind::Hover,
                UiComponentEvent::Hover { hovered: true },
                UiPointerComponentEventReason::HoverEnter,
            )?;
        }
        for node_id in &route.left {
            self.push_pointer_component_events(
                &mut events,
                *node_id,
                UiEventKind::Hover,
                UiComponentEvent::Hover { hovered: false },
                UiPointerComponentEventReason::HoverLeave,
            )?;
        }

        if route.activation_phase == UiPointerActivationPhase::PrimaryPress {
            if let Some(node_id) = route.target {
                if self.node_interaction_enabled(node_id)? {
                    self.push_pointer_component_events(
                        &mut events,
                        node_id,
                        UiEventKind::Press,
                        UiComponentEvent::Press { pressed: true },
                        UiPointerComponentEventReason::PressBegin,
                    )?;
                }
            }
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryRelease {
            if let Some(node_id) = route.pressed {
                if self.node_interaction_enabled(node_id)? {
                    self.push_pointer_component_events(
                        &mut events,
                        node_id,
                        UiEventKind::Release,
                        UiComponentEvent::Press { pressed: false },
                        UiPointerComponentEventReason::PressEnd,
                    )?;
                }
            }
            if let Some(node_id) = route.click_target {
                if self.node_interaction_enabled(node_id)?
                    && !self.uses_typed_default_click_action(node_id)?
                {
                    self.push_pointer_component_events(
                        &mut events,
                        node_id,
                        UiEventKind::Click,
                        UiComponentEvent::Commit {
                            property: "activated".to_string(),
                            value: zircon_runtime_interface::ui::component::UiValue::Bool(true),
                        },
                        UiPointerComponentEventReason::DefaultClick,
                    )?;
                    if event.click_count >= 2 {
                        self.push_pointer_component_events(
                            &mut events,
                            node_id,
                            UiEventKind::DoubleClick,
                            UiComponentEvent::Commit {
                                property: "double_activated".to_string(),
                                value: zircon_runtime_interface::ui::component::UiValue::Bool(true),
                            },
                            UiPointerComponentEventReason::DefaultDoubleClick,
                        )?;
                    }
                }
            }
        }

        Ok(events)
    }

    pub(super) fn push_focus_component_events(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        old_focus: Option<UiNodeId>,
        new_focus: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        if old_focus == new_focus {
            return Ok(());
        }
        if let Some(node_id) = old_focus {
            self.push_pointer_component_events(
                events,
                node_id,
                UiEventKind::Blur,
                UiComponentEvent::Focus { focused: false },
                UiPointerComponentEventReason::FocusLost,
            )?;
        }
        if let Some(node_id) = new_focus {
            self.push_pointer_component_events(
                events,
                node_id,
                UiEventKind::Focus,
                UiComponentEvent::Focus { focused: true },
                UiPointerComponentEventReason::FocusGained,
            )?;
        }
        Ok(())
    }

    pub(super) fn push_state_damage_frames(
        &self,
        result: &mut UiPointerDispatchResult,
        route: &UiPointerRoute,
        focus_before_dispatch: Option<UiNodeId>,
    ) {
        for node_id in route.entered.iter().chain(route.left.iter()) {
            self.push_damage_frame(result, *node_id);
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryPress {
            if let Some(node_id) = route.target {
                self.push_damage_frame(result, node_id);
            }
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryRelease {
            if let Some(node_id) = route.pressed {
                self.push_damage_frame(result, node_id);
            }
            if let Some(node_id) = route.click_target {
                self.push_damage_frame(result, node_id);
            }
        }
        if focus_before_dispatch != self.focus.focused {
            if let Some(node_id) = focus_before_dispatch {
                self.push_damage_frame(result, node_id);
            }
            if let Some(node_id) = self.focus.focused {
                self.push_damage_frame(result, node_id);
            }
        }
    }

    pub(super) fn push_damage_frame(
        &self,
        result: &mut UiPointerDispatchResult,
        node_id: UiNodeId,
    ) {
        let Some(frame) = self.arranged_node(node_id).map(|node| node.frame) else {
            return;
        };
        if !result.requested_damage.contains(&frame) {
            result.requested_damage.push(frame);
        }
    }

    pub(super) fn push_pointer_component_events(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        node_id: UiNodeId,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        reason: UiPointerComponentEventReason,
    ) -> Result<(), UiTreeError> {
        self.push_pointer_component_events_with_drag_metrics(
            events, node_id, event_kind, event, reason, None,
        )
    }

    pub(super) fn push_pointer_component_events_with_drag_metrics(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        node_id: UiNodeId,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        reason: UiPointerComponentEventReason,
        drag: Option<zircon_runtime_interface::ui::component::UiDragMetrics>,
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
        for binding in metadata
            .bindings
            .iter()
            .filter(|binding| binding.event == event_kind)
        {
            let mut component_event = UiPointerComponentEvent::new(
                &self.tree.tree_id,
                node_id,
                control_id,
                binding.id.as_str(),
                event_kind,
                event.clone(),
                reason,
            );
            if let Some(drag) = drag {
                component_event = component_event.with_drag_metrics(drag);
            }
            if let Some(template_action) = self.template_action_for_binding(node_id, binding) {
                component_event = component_event.with_template_action(template_action);
            }
            events.push(component_event);
        }
        Ok(())
    }

    pub(super) fn template_action_for_binding(
        &self,
        source_node_id: UiNodeId,
        binding: &UiBindingRef,
    ) -> Option<UiTemplateActionInvocation> {
        if !self.tree.node(source_node_id)?.state_flags.enabled {
            return None;
        }
        let action = binding.action.as_ref()?;
        let route = action
            .route
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());
        let action_id = action
            .action
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());
        let route = match (route, action_id) {
            (None, Some(action_id)) if action.payload.is_empty() => {
                return Some(UiTemplateActionInvocation::action(action_id));
            }
            (Some(route), None) => route,
            _ => return None,
        };

        let payload = action
            .payload
            .iter()
            .map(|(key, value)| {
                Some((
                    key.clone(),
                    self.template_action_payload_value(source_node_id, value)?,
                ))
            })
            .collect::<Option<_>>()?;
        Some(UiTemplateActionInvocation::route(route, payload))
    }

    fn template_action_payload_value(
        &self,
        source_node_id: UiNodeId,
        value: &toml::Value,
    ) -> Option<UiValue> {
        let toml::Value::String(expression_text) = value else {
            return Some(UiValue::from_toml(value));
        };
        if !expression_text.trim_start().starts_with('=') {
            return Some(UiValue::String(expression_text.clone()));
        }

        UiBindingExpression::parse(expression_text)
            .ok()
            .and_then(|expression| {
                self.resolve_template_action_expression(source_node_id, &expression)
            })
    }

    fn resolve_template_action_expression(
        &self,
        source_node_id: UiNodeId,
        expression: &UiBindingExpression,
    ) -> Option<UiValue> {
        match expression {
            UiBindingExpression::Literal(value) => Some(value.clone()),
            UiBindingExpression::ParamRef(_) => None,
            UiBindingExpression::PropRef(property) => {
                self.template_action_property_value(source_node_id, property)
            }
            UiBindingExpression::ControlPropRef {
                control_id,
                property,
            } => self.template_action_control_property_value(control_id, property),
            UiBindingExpression::Equals(lhs, rhs) => Some(UiValue::Bool(
                self.resolve_template_action_expression(source_node_id, lhs)?
                    == self.resolve_template_action_expression(source_node_id, rhs)?,
            )),
            UiBindingExpression::NotEquals(lhs, rhs) => Some(UiValue::Bool(
                self.resolve_template_action_expression(source_node_id, lhs)?
                    != self.resolve_template_action_expression(source_node_id, rhs)?,
            )),
            UiBindingExpression::And(lhs, rhs) => Some(UiValue::Bool(
                template_action_bool(
                    &self.resolve_template_action_expression(source_node_id, lhs)?,
                )? && template_action_bool(
                    &self.resolve_template_action_expression(source_node_id, rhs)?,
                )?,
            )),
            UiBindingExpression::Or(lhs, rhs) => Some(UiValue::Bool(
                template_action_bool(
                    &self.resolve_template_action_expression(source_node_id, lhs)?,
                )? || template_action_bool(
                    &self.resolve_template_action_expression(source_node_id, rhs)?,
                )?,
            )),
            UiBindingExpression::Not(value) => Some(UiValue::Bool(!template_action_bool(
                &self.resolve_template_action_expression(source_node_id, value)?,
            )?)),
        }
    }

    fn template_action_control_property_value(
        &self,
        control_id: &str,
        property: &str,
    ) -> Option<UiValue> {
        self.control_index
            .node_id(&self.tree, control_id)
            .map(|node_id| self.template_action_property_value(node_id, property))
            .flatten()
    }

    fn template_action_property_value(&self, node_id: UiNodeId, property: &str) -> Option<UiValue> {
        self.component_states
            .get(node_id)
            .and_then(|state| state.value(property))
            .cloned()
            .or_else(|| {
                self.tree
                    .node(node_id)
                    .and_then(|node| node.template_metadata.as_ref())
                    .and_then(|metadata| metadata.attributes.get(property))
                    .map(UiValue::from_toml)
            })
    }
}

fn minimal_changed_subtree_roots(
    tree: &zircon_runtime_interface::ui::tree::UiTree,
    changed_node_ids: &BTreeSet<UiNodeId>,
) -> Result<Vec<UiNodeId>, UiTreeError> {
    let mut roots = Vec::new();
    for node_id in changed_node_ids {
        let node = tree
            .nodes
            .get(node_id)
            .ok_or(UiTreeError::MissingNode(*node_id))?;
        let mut ancestor = node.parent;
        let mut covered = false;
        while let Some(ancestor_id) = ancestor {
            if changed_node_ids.contains(&ancestor_id) {
                covered = true;
                break;
            }
            ancestor = tree
                .nodes
                .get(&ancestor_id)
                .ok_or(UiTreeError::MissingNode(ancestor_id))?
                .parent;
        }
        if !covered {
            roots.push(*node_id);
        }
    }
    Ok(roots)
}

fn node_is_covered_by_roots(
    tree: &zircon_runtime_interface::ui::tree::UiTree,
    node_id: UiNodeId,
    root_ids: &BTreeSet<UiNodeId>,
) -> Result<bool, UiTreeError> {
    let mut current = Some(node_id);
    while let Some(current_id) = current {
        if root_ids.contains(&current_id) {
            return Ok(true);
        }
        current = tree
            .nodes
            .get(&current_id)
            .ok_or(UiTreeError::MissingNode(current_id))?
            .parent;
    }
    Ok(false)
}

fn template_action_bool(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}
