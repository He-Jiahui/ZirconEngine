use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::{
        UiPointerComponentEvent, UiPointerComponentEventReason, UiPointerDispatchResult,
        UiPointerEvent,
    },
    event_ui::UiNodeId,
    surface::{UiPointerActivationPhase, UiPointerEventKind, UiPointerRoute},
    tree::{UiDirtyFlags, UiTreeError},
};

use super::UiSurface;

impl UiSurface {
    pub(super) fn apply_pointer_component_state(
        &mut self,
        route: &UiPointerRoute,
        focus_before_dispatch: Option<UiNodeId>,
    ) -> Result<(), UiTreeError> {
        for node_id in &route.entered {
            if self.component_states.set_hovered(*node_id, true) {
                self.mark_component_state_render_dirty(*node_id)?;
            }
        }
        for node_id in &route.left {
            if self.component_states.set_hovered(*node_id, false) {
                self.mark_component_state_render_dirty(*node_id)?;
            }
        }
        match route.activation_phase {
            UiPointerActivationPhase::PrimaryPress => {
                if let Some(target) = route.target {
                    if self.node_interaction_enabled(target)?
                        && self.component_states.set_pressed(target, true)
                    {
                        self.mark_component_state_render_dirty(target)?;
                    }
                }
            }
            UiPointerActivationPhase::PrimaryRelease => {
                if let Some(pressed) = route.pressed {
                    if self.component_states.set_pressed(pressed, false) {
                        self.mark_component_state_render_dirty(pressed)?;
                    }
                }
            }
            _ => {}
        }
        if matches!(route.kind, UiPointerEventKind::Cancel) {
            if let Some(pressed) = route.pressed {
                if self.component_states.set_pressed(pressed, false) {
                    self.mark_component_state_render_dirty(pressed)?;
                }
            }
        }
        if focus_before_dispatch == self.focus.focused {
            return Ok(());
        }
        if let Some(previous) = focus_before_dispatch {
            if self.component_states.set_focused(previous, false) {
                self.mark_component_state_render_dirty(previous)?;
            }
        }
        if let Some(current) = self.focus.focused {
            if self.component_states.set_focused(current, true) {
                self.mark_component_state_render_dirty(current)?;
            }
        }
        Ok(())
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
        if component_state_changed || state_flags_changed {
            self.mark_component_state_render_dirty(node_id)?;
        }
        Ok(())
    }

    pub(crate) fn mark_component_state_render_dirty(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<(), UiTreeError> {
        let _ = self.apply_runtime_state_style_subtree(node_id, true)?;
        self.mark_node_dirty(
            node_id,
            UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            },
        )
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
        let Some(frame) = self.arranged_tree.get(node_id).map(|node| node.frame) else {
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
            events.push(component_event);
        }
        Ok(())
    }
}
