use crate::ui::dispatch::{
    UiInputDispatchOutcome, UiInputManager, UiNavigationDispatcher, UiPointerDispatcher,
};
use crate::ui::surface::input::{
    apply_dispatch_reply, apply_dispatch_reply_steps, dispatch_input_event, is_valid_input_owner,
};
use crate::ui::tree::{
    UiRuntimeTreeFocusExt, UiRuntimeTreeInteractionExt, UiRuntimeTreeRoutingExt,
    UiRuntimeTreeScrollExt,
};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchReply, UiDispatchReplyStep, UiInputDispatchResult, UiInputEvent,
        UiInputModifiers, UiNavigationDispatchResult, UiPointerDispatchEffect,
        UiPointerDispatchResult, UiPointerEvent,
    },
    event_ui::UiNodeId,
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
    layout::UiPoint,
    surface::{
        UiHitTestQuery, UiNavigationEventKind, UiNavigationRoute, UiPointerActivationPhase,
        UiPointerButton, UiPointerEventKind, UiPointerRoute,
    },
    tree::UiTreeError,
    window::{UiWindowInputPumpBatch, UiWindowInputPumpEvent},
};

use super::{default_interactions, UiSurface};

impl UiSurface {
    pub fn capture_pointer(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        if !is_valid_input_owner(self, node_id) {
            return Err(UiTreeError::MissingNode(node_id));
        }
        if let Some(previous) = self.focus.captured.filter(|owner| owner != &node_id) {
            self.input.clear_high_precision_for(previous);
        }
        self.input.clear_pointer_capture_for(node_id);
        self.focus.captured = Some(node_id);
        Ok(())
    }

    pub fn release_pointer_capture(&mut self) -> Option<UiNodeId> {
        let released = self.focus.captured.take();
        if let Some(owner) = released {
            self.input.clear_pointer_capture_for(owner);
            self.input.clear_pointer_drag_for(owner);
        } else {
            self.input.clear_pointer_capture();
        }
        released
    }

    pub fn apply_dispatch_reply(
        &mut self,
        event: UiInputEvent,
        reply: UiDispatchReply,
    ) -> UiInputDispatchResult {
        apply_dispatch_reply(self, event, reply)
    }

    pub fn apply_dispatch_reply_steps(
        &mut self,
        event: UiInputEvent,
        steps: impl IntoIterator<Item = UiDispatchReplyStep>,
    ) -> UiInputDispatchResult {
        apply_dispatch_reply_steps(self, event, steps)
    }

    pub fn dispatch_input_event(
        &mut self,
        pointer_dispatcher: &UiPointerDispatcher,
        navigation_dispatcher: &UiNavigationDispatcher,
        event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        dispatch_input_event(self, pointer_dispatcher, navigation_dispatcher, event)
    }

    pub fn dispatch_input_event_with_manager(
        &mut self,
        manager: &mut UiInputManager,
        event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        manager.dispatch_input_event(self, event)
    }

    pub fn dispatch_window_input_pump_event(
        &mut self,
        manager: &mut UiInputManager,
        event: UiWindowInputPumpEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        manager.dispatch_window_input_pump_event(self, event)
    }

    pub fn dispatch_window_input_pump_batch(
        &mut self,
        manager: &mut UiInputManager,
        batch: UiWindowInputPumpBatch,
    ) -> Result<UiInputDispatchOutcome, UiTreeError> {
        manager.dispatch_window_input_pump_batch(self, batch)
    }

    pub fn route_pointer_event(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(
            kind,
            UiHitTestQuery::new(point),
            None,
            UiInputModifiers::default(),
            0.0,
        )
    }

    pub fn route_pointer_event_with_query(
        &mut self,
        kind: UiPointerEventKind,
        query: UiHitTestQuery,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(kind, query, None, UiInputModifiers::default(), 0.0)
    }

    pub fn route_pointer_event_with_button(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
        button: UiPointerButton,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(
            kind,
            UiHitTestQuery::new(point),
            Some(button),
            UiInputModifiers::default(),
            0.0,
        )
    }

    pub fn route_pointer_event_with_query_and_button(
        &mut self,
        kind: UiPointerEventKind,
        query: UiHitTestQuery,
        button: UiPointerButton,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(
            kind,
            query,
            Some(button),
            UiInputModifiers::default(),
            0.0,
        )
    }

    pub fn dispatch_pointer_event(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let point = event.point;
        self.dispatch_pointer_event_with_query(dispatcher, event, UiHitTestQuery::new(point))
    }

    pub(crate) fn dispatch_pointer_event_with_modifiers(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
        modifiers: UiInputModifiers,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let point = event.point;
        self.dispatch_pointer_event_with_query_and_modifiers(
            dispatcher,
            event,
            UiHitTestQuery::new(point),
            modifiers,
        )
    }

    pub fn dispatch_pointer_event_with_query(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
        query: UiHitTestQuery,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        self.dispatch_pointer_event_with_query_and_modifiers(
            dispatcher,
            event,
            query,
            UiInputModifiers::default(),
        )
    }

    fn dispatch_pointer_event_with_query_and_modifiers(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
        query: UiHitTestQuery,
        modifiers: UiInputModifiers,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let focus_before_dispatch = self.focus.focused;
        let capture_before_dispatch = self.focus.captured;
        let pressed_before_dispatch = self.focus.pressed;
        let route = self.route_pointer_event_with_details(
            event.kind,
            query,
            event.button,
            modifiers,
            event.scroll_delta,
        )?;
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if let Some(node_id) = result.captured_by {
            if capture_before_dispatch != Some(node_id) {
                result.diagnostics.capture_started = true;
            }
            if let Some(previous) = capture_before_dispatch.filter(|owner| owner != &node_id) {
                self.input.clear_high_precision_for(previous);
            }
            self.focus.captured = Some(node_id);
        }
        if let Some(node_id) = result.released_capture {
            if self.focus.captured == Some(node_id) || route.captured == Some(node_id) {
                self.focus.captured = None;
                self.input.clear_pointer_capture_for(node_id);
                result.diagnostics.capture_released = true;
            }
        }
        if let Some(node_id) = result.focus_changed_to {
            let focus_visible = result
                .invocations
                .iter()
                .rev()
                .find_map(|invocation| match invocation.effect {
                    UiPointerDispatchEffect::SetFocus { focus_visible }
                        if invocation.node_id == node_id =>
                    {
                        Some(focus_visible)
                    }
                    _ => None,
                })
                .unwrap_or(false);
            self.focus_node_with_reason(
                node_id,
                UiFocusChangeReason::Input,
                if focus_visible {
                    UiFocusVisible::visible(UiFocusVisibleReason::PointerInteraction)
                } else {
                    UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction)
                },
            )?;
        }
        if result.focus_cleared {
            self.clear_focus();
        }
        if matches!(event.kind, UiPointerEventKind::Scroll)
            && result.handled_by.is_none()
            && result.blocked_by.is_none()
        {
            let candidates = if !route.stacked.is_empty() {
                route.stacked.as_slice()
            } else {
                route.root_targets.as_slice()
            };
            for node_id in self.tree.scrollable_candidates(candidates)? {
                if self.tree.scroll_by(node_id, event.scroll_delta)? {
                    result.handled_by = Some(node_id);
                    result.diagnostics.scroll_defaulted = true;
                    break;
                }
            }
        }
        result.diagnostics.focus_changed = focus_before_dispatch != self.focus.focused;
        result.diagnostics.capture_released = result.diagnostics.capture_released
            || (matches!(
                event.kind,
                UiPointerEventKind::Up | UiPointerEventKind::Cancel
            ) && capture_before_dispatch.is_some()
                && self.focus.captured.is_none());
        if result.diagnostics.capture_released {
            if let Some(owner) = capture_before_dispatch
                .or(result.released_capture)
                .or(route.captured)
            {
                self.input.clear_pointer_capture_for(owner);
            } else {
                self.input.clear_pointer_capture();
            }
        }
        result.diagnostics.default_click_rejected = route.activation_phase
            == UiPointerActivationPhase::PrimaryRelease
            && route.pressed.is_some()
            && route.click_target.is_none();
        self.apply_pointer_component_state(&route, focus_before_dispatch)?;
        self.apply_pointer_transient_state_dirty(&route, pressed_before_dispatch)?;
        result.component_events = self.pointer_component_events(&route, &event)?;
        let range_action = self.apply_default_range_pointer_actions(
            &route,
            &mut result.component_events,
            &mut result.binding_reports,
        )?;
        if let Some(node_id) = range_action.captured_by {
            result.captured_by = Some(node_id);
            result.handled_by = Some(node_id);
            result.diagnostics.capture_started = true;
        }
        if let Some(node_id) = range_action.released_capture {
            result.released_capture = Some(node_id);
            result.handled_by = Some(node_id);
            result.diagnostics.capture_released = true;
        }
        if let Some(node_id) = range_action.handled_by {
            result.handled_by = Some(node_id);
        } else {
            let scrollbar_action = self.apply_default_scrollbar_pointer_action(
                &route,
                &mut result.component_events,
                &mut result.binding_reports,
            )?;
            if let Some(node_id) = scrollbar_action.captured_by {
                result.captured_by = Some(node_id);
                result.handled_by = Some(node_id);
                result.diagnostics.capture_started = true;
            }
            if let Some(node_id) = scrollbar_action.released_capture {
                result.released_capture = Some(node_id);
                result.handled_by = Some(node_id);
                result.diagnostics.capture_released = true;
            }
            if let Some(node_id) = scrollbar_action.handled_by {
                result.handled_by = Some(node_id);
                result.diagnostics.scroll_defaulted = true;
            } else {
                let table_action = self.apply_default_table_pointer_action(
                    &route,
                    &mut result.component_events,
                    &mut result.binding_reports,
                )?;
                if let Some(node_id) = table_action.captured_by {
                    result.captured_by = Some(node_id);
                    result.handled_by = Some(node_id);
                    result.diagnostics.capture_started = true;
                }
                if let Some(node_id) = table_action.released_capture {
                    result.released_capture = Some(node_id);
                    result.handled_by = Some(node_id);
                    result.diagnostics.capture_released = true;
                }
                if let Some(node_id) = table_action.handled_by {
                    result.handled_by = Some(node_id);
                } else {
                    let tree_action = self.apply_default_tree_view_virtual_scroll(
                        &route,
                        &mut result.component_events,
                        &mut result.binding_reports,
                    )?;
                    if let Some(node_id) = tree_action.handled_by {
                        result.handled_by = Some(node_id);
                        result.diagnostics.scroll_defaulted = true;
                    } else {
                        self.apply_default_pointer_component_actions(
                            &route,
                            event.click_count,
                            &mut result.component_events,
                            &mut result.binding_reports,
                        )?;
                    }
                    if let Some(node_id) = tree_action.damage_node {
                        self.push_damage_frame(&mut result, node_id);
                    }
                }
                if let Some(node_id) = table_action.damage_node {
                    self.push_damage_frame(&mut result, node_id);
                }
            }
            if let Some(node_id) = scrollbar_action.damage_node {
                self.push_damage_frame(&mut result, node_id);
            }
        }
        if let Some(node_id) = range_action.damage_node {
            self.push_damage_frame(&mut result, node_id);
        }
        self.push_focus_component_events(
            &mut result.component_events,
            focus_before_dispatch,
            self.focus.focused,
        )?;
        self.push_state_damage_frames(&mut result, &route, focus_before_dispatch);
        result.diagnostics.component_event_count = result.component_events.len();
        Ok(result)
    }

    fn route_pointer_event_with_details(
        &mut self,
        kind: UiPointerEventKind,
        query: UiHitTestQuery,
        button: Option<UiPointerButton>,
        modifiers: UiInputModifiers,
        scroll_delta: f32,
    ) -> Result<UiPointerRoute, UiTreeError> {
        let point = query.hit_point();
        let hit = self.hit_test_with_query(query);
        let previous_hovered = self.focus.hovered.clone();
        let captured = self.focus.captured;
        let previous_pressed = self.focus.pressed;
        let target = captured.or(hit.top_hit);
        let bubbled = match target {
            Some(node_id) => self.tree.bubble_route(node_id)?,
            None => Vec::new(),
        };

        self.focus.hovered = hit.stacked.clone();
        if matches!(kind, UiPointerEventKind::Down) {
            self.focus.pressed = target;
            if let Some(focus_target) = self
                .tree
                .first_focusable_in_route(&bubbled)?
                .filter(|focus_target| is_valid_input_owner(self, *focus_target))
            {
                self.focus_node_with_reason(
                    focus_target,
                    UiFocusChangeReason::Input,
                    UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction),
                )?;
            }
        }
        let click_target = if matches!(kind, UiPointerEventKind::Up)
            && button == Some(UiPointerButton::Primary)
            && previous_pressed.is_some_and(|node_id| hit.stacked.contains(&node_id))
        {
            previous_pressed
        } else {
            None
        };
        if matches!(kind, UiPointerEventKind::Up) {
            self.focus.pressed = None;
            self.focus.captured = None;
            if let Some(owner) = captured {
                self.input.clear_pointer_capture_for(owner);
            } else {
                self.input.clear_pointer_capture();
            }
        } else if matches!(kind, UiPointerEventKind::Cancel) {
            self.focus.pressed = None;
            self.release_pointer_capture();
        }
        let pressed = if matches!(kind, UiPointerEventKind::Down) {
            self.focus.pressed
        } else {
            previous_pressed
        };

        Ok(UiPointerRoute {
            kind,
            button,
            modifiers,
            activation_phase: activation_phase(kind, button),
            point,
            scroll_delta,
            target,
            hit_path: hit.path.clone(),
            bubbled,
            stacked: hit.stacked.clone(),
            entered: diff_nodes(&hit.stacked, &previous_hovered),
            left: diff_nodes(&previous_hovered, &hit.stacked),
            captured,
            pressed,
            click_target,
            release_inside_pressed: click_target.is_some(),
            focused: self.focus.focused,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }

    pub fn route_navigation_event(
        &self,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationRoute, UiTreeError> {
        let target = self.focus.focused.or(self.navigation.navigation_root);
        let bubbled = match target {
            Some(node_id) => self.tree.bubble_route(node_id)?,
            None => Vec::new(),
        };
        Ok(UiNavigationRoute {
            kind,
            target,
            bubbled,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }

    pub fn dispatch_navigation_event(
        &mut self,
        dispatcher: &UiNavigationDispatcher,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        let route = self.route_navigation_event(kind)?;
        if let Some(target) = route.target {
            if let Some(action) = default_interactions::range_navigation_action(kind) {
                let range_action = match action {
                    default_interactions::UiDefaultRangeNavigationAction::Step(direction) => {
                        self.mutate_default_range_step_value(target, direction)?
                    }
                    default_interactions::UiDefaultRangeNavigationAction::Minimum => {
                        self.mutate_default_range_endpoint_value(target, false)?
                    }
                    default_interactions::UiDefaultRangeNavigationAction::Maximum => {
                        self.mutate_default_range_endpoint_value(target, true)?
                    }
                };
                if let Some((report, _value)) = range_action {
                    let mut result = UiNavigationDispatchResult::new(route);
                    result.handled_by = Some(target);
                    result.record_binding_report(report.binding);
                    return Ok(result);
                }
            }
        }
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if result.focus_changed_to.is_none() {
            if let Some(node_id) = self.tree.next_navigation_target(route.target, route.kind)? {
                result.handled_by = Some(route.target.unwrap_or(node_id));
                result.focus_changed_to = Some(node_id);
            }
        }
        if let Some(node_id) = result.focus_changed_to {
            self.focus_node_with_reason(
                node_id,
                UiFocusChangeReason::Navigation,
                UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation),
            )?;
        }
        Ok(result)
    }
}

fn diff_nodes(current: &[UiNodeId], previous: &[UiNodeId]) -> Vec<UiNodeId> {
    current
        .iter()
        .filter(|node_id| !previous.contains(node_id))
        .copied()
        .collect()
}

fn activation_phase(
    kind: UiPointerEventKind,
    button: Option<UiPointerButton>,
) -> UiPointerActivationPhase {
    match (kind, button) {
        (UiPointerEventKind::Down, Some(UiPointerButton::Primary)) => {
            UiPointerActivationPhase::PrimaryPress
        }
        (UiPointerEventKind::Up, Some(UiPointerButton::Primary)) => {
            UiPointerActivationPhase::PrimaryRelease
        }
        (UiPointerEventKind::Down, Some(UiPointerButton::Secondary)) => {
            UiPointerActivationPhase::SecondaryPress
        }
        (UiPointerEventKind::Up, Some(UiPointerButton::Secondary)) => {
            UiPointerActivationPhase::SecondaryRelease
        }
        (UiPointerEventKind::Down, Some(UiPointerButton::Middle)) => {
            UiPointerActivationPhase::MiddlePress
        }
        (UiPointerEventKind::Up, Some(UiPointerButton::Middle)) => {
            UiPointerActivationPhase::MiddleRelease
        }
        (UiPointerEventKind::Move, _) => UiPointerActivationPhase::Hover,
        (UiPointerEventKind::Scroll, _) => UiPointerActivationPhase::Scroll,
        _ => UiPointerActivationPhase::None,
    }
}
