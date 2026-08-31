use zircon_runtime_interface::ui::{
    dispatch::{UiInputMethodRequest, UiInputMethodRequestKind},
    event_ui::UiNodeId,
    focus::{
        UiFocusChangeEvent, UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason,
        UiFocusedInput, UiFocusedInputKind,
    },
    tree::UiTreeError,
};

mod modal_scope;

use super::input::{
    cancel_editable_text_composition_for_input_method_loss, editable_text_input_is_secure,
    finish_editable_text_for_focus_loss, is_editable_text_input, is_valid_input_owner,
};
use super::surface::UiSurface;

const UI_FOCUS_DIAGNOSTIC_HISTORY_CAPACITY: usize = 64;

impl UiSurface {
    pub fn focus_node(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        self.focus_node_with_reason(
            node_id,
            UiFocusChangeReason::Programmatic,
            UiFocusVisible::hidden(UiFocusVisibleReason::Programmatic),
        )?;
        Ok(())
    }

    pub(crate) fn focus_node_with_reason(
        &mut self,
        node_id: UiNodeId,
        reason: UiFocusChangeReason,
        visible: UiFocusVisible,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        let node_id = self.enforced_modal_focus_target(node_id)?;
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !(node.is_focus_candidate() || self.is_open_modal_focus_root(node_id))
            || !is_valid_input_owner(self, node_id)
        {
            return Err(UiTreeError::MissingNode(node_id));
        }

        let previous = self.focus.focused;
        self.focus.previous = previous;
        self.focus.focused = Some(node_id);
        self.focus.pending_autofocus = None;
        self.focus.focus_visible = visible;
        self.navigation.navigation_root = Some(node_id);
        self.navigation.focus_visible = visible.visible;

        if let Some(previous_id) = previous.filter(|previous_id| *previous_id != node_id) {
            let focused_changed = self.component_states.set_focused(previous_id, false);
            let focus_visible_changed = self.component_states.set_focus_visible(previous_id, false);
            if focused_changed || focus_visible_changed {
                mark_component_focus_render_dirty(self, previous_id);
            }
        }
        let focused_changed = self.component_states.set_focused(node_id, true);
        let focus_visible_changed = self
            .component_states
            .set_focus_visible(node_id, visible.visible);
        if focused_changed || focus_visible_changed {
            mark_component_focus_render_dirty(self, node_id);
        }

        if previous == Some(node_id) {
            return Ok(None);
        }

        if let Some(previous) = previous {
            self.invalidate_clipboard_transfers_for(previous);
        }
        self.transition_focus_input_method(previous, Some(node_id));
        let event = UiFocusChangeEvent {
            previous,
            current: Some(node_id),
            reason,
            visible,
        };
        push_focus_diagnostic(&mut self.focus.changes, event);
        Ok(Some(event))
    }

    pub fn clear_focus(&mut self) {
        self.clear_focus_with_reason(UiFocusChangeReason::Clear);
    }

    pub(crate) fn clear_focus_with_reason(
        &mut self,
        reason: UiFocusChangeReason,
    ) -> Option<UiFocusChangeEvent> {
        let previous = self.focus.focused?;
        let visible = UiFocusVisible::hidden(clear_focus_visible_reason(reason));
        self.focus.previous = Some(previous);
        self.focus.focused = None;
        self.focus.focus_visible = visible;
        self.navigation.navigation_root = None;
        self.navigation.focus_visible = false;
        let focused_changed = self.component_states.set_focused(previous, false);
        let focus_visible_changed = self.component_states.set_focus_visible(previous, false);
        if focused_changed || focus_visible_changed {
            mark_component_focus_render_dirty(self, previous);
        }
        self.invalidate_clipboard_transfers_for(previous);
        self.transition_focus_input_method(Some(previous), None);
        let event = UiFocusChangeEvent {
            previous: Some(previous),
            current: None,
            reason,
            visible,
        };
        push_focus_diagnostic(&mut self.focus.changes, event);
        Some(event)
    }

    pub fn resolve_autofocus(&mut self) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        if self.focus.focused.is_some() {
            return Ok(None);
        }
        let target = self
            .focus
            .pending_autofocus
            .filter(|node_id| self.is_focus_target(*node_id))
            .or_else(|| self.first_autofocus_target());
        self.focus.pending_autofocus = target;
        let Some(target) = target else {
            return Ok(None);
        };
        self.focus_node_with_reason(
            target,
            UiFocusChangeReason::Autofocus,
            UiFocusVisible::hidden(UiFocusVisibleReason::Programmatic),
        )
    }

    pub(crate) fn reconcile_focus_after_tree_change(
        &mut self,
        reason: UiFocusChangeReason,
    ) -> Option<UiFocusChangeEvent> {
        let focus_change = if self
            .focus
            .focused
            .is_some_and(|node_id| !self.is_focus_target(node_id))
        {
            self.clear_focus_with_reason(reason)
        } else {
            None
        };
        self.clear_invalid_transient_input_owners();
        focus_change
    }

    pub(crate) fn reset_detached_transient_state_for_nodes(
        &mut self,
        node_ids: &[UiNodeId],
        reason: UiFocusChangeReason,
    ) -> Option<UiFocusChangeEvent> {
        for node_id in node_ids {
            self.drop_clipboard_transfers_for(*node_id);
            self.input.drop_text_document_epoch(*node_id);
        }
        let focus_change = if self
            .focus
            .focused
            .is_some_and(|focused| node_ids.contains(&focused))
        {
            self.clear_focus_with_reason(reason)
        } else {
            None
        };
        if self
            .focus
            .captured
            .is_some_and(|captured| node_ids.contains(&captured))
        {
            if let Some(captured) = self.focus.captured.take() {
                self.input.clear_pointer_capture_for(captured);
            }
        }
        self.input.clear_pointer_drags_for_nodes(node_ids);
        if self
            .focus
            .pressed
            .is_some_and(|pressed| node_ids.contains(&pressed))
        {
            self.focus.pressed = None;
        }
        self.focus
            .hovered
            .retain(|hovered| !node_ids.contains(hovered));
        if let Some(owner) = self
            .input
            .high_precision_owner
            .filter(|owner| node_ids.contains(owner))
        {
            self.input.clear_high_precision_for(owner);
        }
        if self
            .input
            .input_method_owner
            .is_some_and(|owner| node_ids.contains(&owner))
        {
            self.disable_input_method_for_focus_loss();
        }
        if self
            .input
            .pointer_lock_owner
            .is_some_and(|owner| node_ids.contains(&owner))
        {
            self.input.pointer_lock_owner = None;
            self.input.pointer_lock_policy = None;
        }
        if let Some(source) = self.input.drag_drop.as_ref().and_then(|drag| {
            (node_ids.contains(&drag.source) || node_ids.contains(&drag.target))
                .then_some(drag.source)
        }) {
            self.clear_drag_drop_session_for_source(source);
        }
        focus_change
    }

    pub(crate) fn record_focused_input(
        &mut self,
        kind: UiFocusedInputKind,
        focused: UiNodeId,
        route: Vec<UiNodeId>,
        handled_by: Option<UiNodeId>,
        accepted: bool,
    ) -> UiFocusedInput {
        let event = UiFocusedInput {
            focused,
            kind,
            route,
            handled_by,
            accepted,
        };
        push_focus_diagnostic(&mut self.focus.focused_inputs, event.clone());
        event
    }

    fn first_autofocus_target(&self) -> Option<UiNodeId> {
        self.tree
            .nodes
            .values()
            .filter(|node| {
                node.focus.autofocus
                    || node.template_metadata.as_ref().is_some_and(|metadata| {
                        bool_attribute_any(metadata, &["autofocus", "auto_focus", "autoFocus"])
                    })
            })
            .filter(|node| self.is_focus_target(node.node_id))
            .min_by_key(|node| node.paint_order)
            .map(|node| node.node_id)
    }

    fn is_focus_target(&self, node_id: UiNodeId) -> bool {
        self.tree.nodes.get(&node_id).is_some_and(|node| {
            (node.is_focus_candidate() || self.is_open_modal_focus_root(node_id))
                && is_valid_input_owner(self, node_id)
        })
    }

    fn transition_focus_input_method(
        &mut self,
        previous_focus: Option<UiNodeId>,
        next_focus: Option<UiNodeId>,
    ) {
        if previous_focus == next_focus {
            return;
        }

        if let Some(owner) = previous_focus.filter(|owner| is_editable_text_input(self, *owner)) {
            self.input.record_focus_loss(owner);
            let component_event = finish_editable_text_for_focus_loss(self, owner);
            self.input.queue_focus_component_event(component_event);
        }
        let disabled_previous_owner = self.disable_input_method_for_focus_loss();

        let Some(target) = next_focus else {
            return;
        };
        if !is_editable_text_input(self, target) {
            return;
        }
        if editable_text_input_is_secure(self, target) {
            if disabled_previous_owner {
                return;
            }
            self.input.queue_focus_input_lifecycle(
                None,
                input_method_request(UiInputMethodRequestKind::Disable, target),
            );
            return;
        }

        let request = input_method_request(UiInputMethodRequestKind::Enable, target);
        self.input.input_method_owner = Some(target);
        self.input.input_method_request = Some(request.clone());
        self.input.queue_focus_input_lifecycle(None, request);
    }

    pub(in crate::ui::surface) fn disable_input_method_for_focus_loss(&mut self) -> bool {
        let previous_input_method_owner = self.input.input_method_owner.take();
        self.input.input_method_request = None;
        let Some(owner) = previous_input_method_owner else {
            return false;
        };
        cancel_editable_text_composition_for_input_method_loss(self, owner);
        self.input.queue_focus_input_lifecycle(
            None,
            input_method_request(UiInputMethodRequestKind::Disable, owner),
        );
        true
    }

    fn clear_invalid_transient_input_owners(&mut self) {
        if self
            .focus
            .captured
            .is_some_and(|owner| !is_valid_input_owner(self, owner))
        {
            if let Some(owner) = self.focus.captured.take() {
                self.input.clear_pointer_capture_for(owner);
                self.input.clear_pointer_drag_for(owner);
            }
        }
        if self
            .focus
            .pressed
            .is_some_and(|owner| !is_valid_input_owner(self, owner))
        {
            self.focus.pressed = None;
        }
        self.focus.hovered = self
            .focus
            .hovered
            .iter()
            .copied()
            .filter(|owner| is_valid_input_owner(self, *owner))
            .collect();
        let invalid_pointer_drag_owners = self
            .input
            .pointer_drags
            .keys()
            .copied()
            .filter(|owner| !is_valid_input_owner(self, *owner))
            .collect::<Vec<_>>();
        for owner in invalid_pointer_drag_owners {
            self.input.clear_pointer_drag_for(owner);
        }
        if self
            .input
            .high_precision_owner
            .is_some_and(|owner| !is_valid_input_owner(self, owner))
        {
            self.input.high_precision_owner = None;
        }
        if self
            .input
            .input_method_owner
            .is_some_and(|owner| !is_valid_input_owner(self, owner))
        {
            self.disable_input_method_for_focus_loss();
        }
        if self
            .input
            .pointer_lock_owner
            .is_some_and(|owner| !is_valid_input_owner(self, owner))
        {
            self.input.pointer_lock_owner = None;
            self.input.pointer_lock_policy = None;
        }
        if let Some(source) = self.input.drag_drop.as_ref().and_then(|drag| {
            (!is_valid_input_owner(self, drag.source) || !is_valid_input_owner(self, drag.target))
                .then_some(drag.source)
        }) {
            self.clear_drag_drop_session_for_source(source);
        }
    }

    fn clear_drag_drop_session_for_source(&mut self, source: UiNodeId) {
        if self.focus.captured == Some(source) {
            self.focus.captured = None;
        }
        self.input.clear_pointer_capture_for(source);
        self.input.clear_pointer_drag_for(source);
        self.input.drag_drop = None;
    }
}

fn push_focus_diagnostic<T>(history: &mut Vec<T>, event: T) {
    let retained_before_append = UI_FOCUS_DIAGNOSTIC_HISTORY_CAPACITY.saturating_sub(1);
    if history.len() > retained_before_append {
        let expired_count = history.len() - retained_before_append;
        history.drain(..expired_count);
    }
    history.push(event);
}

fn input_method_request(kind: UiInputMethodRequestKind, owner: UiNodeId) -> UiInputMethodRequest {
    UiInputMethodRequest {
        kind,
        owner,
        cursor_rect: None,
        composition_rects: Vec::new(),
        surrounding_text: None,
    }
}

fn clear_focus_visible_reason(reason: UiFocusChangeReason) -> UiFocusVisibleReason {
    match reason {
        UiFocusChangeReason::Disabled
        | UiFocusChangeReason::Hidden
        | UiFocusChangeReason::Despawned => UiFocusVisibleReason::DisabledOrHidden,
        UiFocusChangeReason::Input => UiFocusVisibleReason::PointerInteraction,
        UiFocusChangeReason::Navigation => UiFocusVisibleReason::KeyboardNavigation,
        UiFocusChangeReason::Programmatic
        | UiFocusChangeReason::Autofocus
        | UiFocusChangeReason::Clear => UiFocusVisibleReason::Programmatic,
    }
}

fn mark_component_focus_render_dirty(surface: &mut UiSurface, node_id: UiNodeId) {
    let _ = surface.mark_component_state_render_dirty(node_id);
}

fn bool_attribute(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    key: &str,
) -> bool {
    metadata
        .attributes
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn bool_attribute_any(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    keys: &[&str],
) -> bool {
    keys.iter().any(|key| bool_attribute(metadata, key))
}
