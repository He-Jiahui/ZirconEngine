use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    focus::{
        UiFocusChangeEvent, UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason,
        UiFocusedInput, UiFocusedInputKind,
    },
    tree::UiTreeError,
};

use crate::ui::tree::UiRuntimeTreeAccessExt;

use super::input::is_valid_input_owner;
use super::surface::UiSurface;

impl UiSurface {
    pub fn focus_node(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        self.focus_node_with_reason(
            node_id,
            UiFocusChangeReason::Programmatic,
            UiFocusVisible::visible(UiFocusVisibleReason::Programmatic),
        )?;
        Ok(())
    }

    pub(crate) fn focus_node_with_reason(
        &mut self,
        node_id: UiNodeId,
        reason: UiFocusChangeReason,
        visible: UiFocusVisible,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !node.is_focus_candidate() || !is_valid_input_owner(self, node_id) {
            return Err(UiTreeError::MissingNode(node_id));
        }

        let previous = self.focus.focused;
        if self
            .input
            .input_method_owner
            .is_some_and(|owner| owner != node_id)
        {
            self.input.clear_input_method();
        }

        self.focus.previous = previous;
        self.focus.focused = Some(node_id);
        self.focus.pending_autofocus = None;
        self.focus.focus_visible = visible;
        self.navigation.navigation_root = Some(node_id);
        self.navigation.focus_visible = visible.visible;

        if previous == Some(node_id) {
            return Ok(None);
        }

        let event = UiFocusChangeEvent {
            previous,
            current: Some(node_id),
            reason,
            visible,
        };
        self.focus.changes.push(event);
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
        if self.input.input_method_owner == Some(previous) {
            self.input.clear_input_method();
        }
        let visible = UiFocusVisible::hidden(clear_focus_visible_reason(reason));
        self.focus.previous = Some(previous);
        self.focus.focused = None;
        self.focus.focus_visible = visible;
        self.navigation.navigation_root = None;
        self.navigation.focus_visible = false;
        let event = UiFocusChangeEvent {
            previous: Some(previous),
            current: None,
            reason,
            visible,
        };
        self.focus.changes.push(event);
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
            UiFocusVisible::visible(UiFocusVisibleReason::Programmatic),
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
            self.input.clear_input_method();
        }
        if self
            .input
            .pointer_lock_owner
            .is_some_and(|owner| node_ids.contains(&owner))
        {
            self.input.pointer_lock_owner = None;
            self.input.pointer_lock_policy = None;
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
        self.focus.focused_inputs.push(event.clone());
        event
    }

    fn first_autofocus_target(&self) -> Option<UiNodeId> {
        self.tree
            .nodes
            .values()
            .filter(|node| node.focus.autofocus)
            .filter(|node| self.is_focus_target(node.node_id))
            .min_by_key(|node| node.paint_order)
            .map(|node| node.node_id)
    }

    fn is_focus_target(&self, node_id: UiNodeId) -> bool {
        self.tree
            .nodes
            .get(&node_id)
            .is_some_and(|node| node.is_focus_candidate() && is_valid_input_owner(self, node_id))
    }

    fn clear_invalid_transient_input_owners(&mut self) {
        if self
            .focus
            .captured
            .is_some_and(|owner| !is_valid_input_owner(self, owner))
        {
            if let Some(owner) = self.focus.captured.take() {
                self.input.clear_pointer_capture_for(owner);
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
            self.input.clear_input_method();
        }
        if self
            .input
            .pointer_lock_owner
            .is_some_and(|owner| !is_valid_input_owner(self, owner))
        {
            self.input.pointer_lock_owner = None;
            self.input.pointer_lock_policy = None;
        }
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
