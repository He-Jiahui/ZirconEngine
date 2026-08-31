use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::{
        UiPointerComponentEvent, UiPointerComponentEventReason, UiPointerDispatchResult,
        UiPointerEvent,
    },
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute},
    template::{UiBindingRef, UiCompiledBindingHandle},
    tree::UiTreeError,
};

use super::UiSurface;

mod state_invalidation;
mod template_action;

impl UiSurface {
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
        requested_damage: &mut Vec<UiFrame>,
        route: &UiPointerRoute,
        focus_before_dispatch: Option<UiNodeId>,
    ) {
        for node_id in route.entered.iter().chain(route.left.iter()) {
            self.push_damage_frame_to(requested_damage, *node_id);
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryPress {
            if let Some(node_id) = route.target {
                self.push_damage_frame_to(requested_damage, node_id);
            }
        }
        if route.activation_phase == UiPointerActivationPhase::PrimaryRelease {
            if let Some(node_id) = route.pressed {
                self.push_damage_frame_to(requested_damage, node_id);
            }
            if let Some(node_id) = route.click_target {
                self.push_damage_frame_to(requested_damage, node_id);
            }
        }
        if focus_before_dispatch != self.focus.focused {
            if let Some(node_id) = focus_before_dispatch {
                self.push_damage_frame_to(requested_damage, node_id);
            }
            if let Some(node_id) = self.focus.focused {
                self.push_damage_frame_to(requested_damage, node_id);
            }
        }
    }

    pub(super) fn push_damage_frame(
        &self,
        result: &mut UiPointerDispatchResult,
        node_id: UiNodeId,
    ) {
        self.push_damage_frame_to(&mut result.requested_damage, node_id);
    }

    fn push_damage_frame_to(&self, requested_damage: &mut Vec<UiFrame>, node_id: UiNodeId) {
        let Some(frame) = self.arranged_node(node_id).map(|node| node.frame) else {
            return;
        };
        if !requested_damage.contains(&frame) {
            requested_damage.push(frame);
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
        let mut event = Some(event);
        if let Some(sources) = self.compiled_binding_event_sources(node_id, event_kind) {
            let mut matches = sources
                .iter()
                .filter_map(|source| {
                    let binding = self.compiled_bindings.binding(source.handle)?;
                    if binding.event != event_kind {
                        return None;
                    }
                    Some((
                        self.compiled_bindings.binding_name(source.handle)?,
                        source.handle,
                    ))
                })
                .peekable();
            while let Some((binding_id, compiled_binding)) = matches.next() {
                let matched_event = if matches.peek().is_some() {
                    event.as_ref().expect("retained component event").clone()
                } else {
                    event.take().expect("one retained component event")
                };
                self.push_pointer_component_event_for_binding(
                    events,
                    node_id,
                    control_id,
                    binding_id,
                    event_kind,
                    matched_event,
                    reason,
                    None,
                    Some(compiled_binding),
                    drag,
                );
            }
        } else {
            let mut matches = metadata
                .bindings
                .iter()
                .enumerate()
                .filter(|(_, binding)| binding.event == event_kind)
                .map(|(source_binding_index, binding)| {
                    let compiled_binding = self.compiled_binding_handle_for_source(
                        node_id,
                        source_binding_index,
                        binding,
                        event_kind,
                    );
                    (binding.id.as_str(), binding, compiled_binding)
                })
                .peekable();
            while let Some((binding_id, binding, compiled_binding)) = matches.next() {
                let matched_event = if matches.peek().is_some() {
                    event.as_ref().expect("retained component event").clone()
                } else {
                    event.take().expect("one retained component event")
                };
                self.push_pointer_component_event_for_binding(
                    events,
                    node_id,
                    control_id,
                    binding_id,
                    event_kind,
                    matched_event,
                    reason,
                    Some(binding),
                    compiled_binding,
                    drag,
                );
            }
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn push_pointer_component_events_for_test(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        node_id: UiNodeId,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        reason: UiPointerComponentEventReason,
    ) -> Result<(), UiTreeError> {
        self.push_pointer_component_events(events, node_id, event_kind, event, reason)
    }

    #[cfg(test)]
    pub(crate) fn push_pointer_component_events_legacy_for_benchmark(
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
        if let Some(sources) = self.compiled_binding_event_sources(node_id, event_kind) {
            for source in sources {
                let Some(binding) = metadata.bindings.get(source.source_binding_index) else {
                    continue;
                };
                if binding.event != event_kind {
                    continue;
                }
                let compiled_binding = self
                    .compiled_binding_handle_for_source(
                        node_id,
                        source.source_binding_index,
                        binding,
                        event_kind,
                    )
                    .filter(|handle| *handle == source.handle);
                self.push_pointer_component_event_for_binding(
                    events,
                    node_id,
                    control_id,
                    binding.id.as_str(),
                    event_kind,
                    event.clone(),
                    reason,
                    Some(binding),
                    compiled_binding,
                    None,
                );
            }
        } else {
            for (source_binding_index, binding) in metadata
                .bindings
                .iter()
                .enumerate()
                .filter(|(_, binding)| binding.event == event_kind)
            {
                let compiled_binding = self.compiled_binding_handle_for_source(
                    node_id,
                    source_binding_index,
                    binding,
                    event_kind,
                );
                self.push_pointer_component_event_for_binding(
                    events,
                    node_id,
                    control_id,
                    binding.id.as_str(),
                    event_kind,
                    event.clone(),
                    reason,
                    Some(binding),
                    compiled_binding,
                    None,
                );
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn push_pointer_component_event_for_binding(
        &self,
        events: &mut Vec<UiPointerComponentEvent>,
        node_id: UiNodeId,
        control_id: &str,
        binding_id: &str,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        reason: UiPointerComponentEventReason,
        binding: Option<&UiBindingRef>,
        compiled_binding: Option<UiCompiledBindingHandle>,
        drag: Option<zircon_runtime_interface::ui::component::UiDragMetrics>,
    ) {
        let mut component_event = UiPointerComponentEvent::new(
            &self.tree.tree_id,
            node_id,
            control_id,
            binding_id,
            event_kind,
            event,
            reason,
        );
        if let Some(compiled_binding) = compiled_binding {
            component_event = component_event.with_compiled_binding(compiled_binding);
        }
        if let Some(drag) = drag {
            component_event = component_event.with_drag_metrics(drag);
        }
        let template_action = if self.compiled_bindings.binding_count() == 0 {
            binding.and_then(|binding| self.template_action_for_binding(node_id, binding))
        } else {
            compiled_binding.and_then(|handle| {
                let compiled = self.compiled_bindings.binding(handle)?;
                if !compiled.targets.is_empty() {
                    return None;
                }
                self.template_action_for_compiled_binding_with_overrides(
                    node_id,
                    handle,
                    BTreeMap::new(),
                )
            })
        };
        if let Some(template_action) = template_action {
            component_event = component_event.with_template_action(template_action);
        }
        events.push(component_event);
    }
}
