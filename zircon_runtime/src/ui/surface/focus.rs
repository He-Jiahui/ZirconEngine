use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    focus::{
        UiFocusChangeEvent, UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason,
        UiFocusedInput, UiFocusedInputKind,
    },
    tree::UiTreeError,
};

use crate::ui::tree::{UiRuntimeTreeAccessExt, UiRuntimeTreeFocusExt};

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
        let node_id = self.enforced_mui_modal_focus_target(node_id)?;
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !(node.is_focus_candidate() || self.is_open_mui_modal_focus_root(node_id))
            || !is_valid_input_owner(self, node_id)
        {
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

        if let Some(previous_id) = previous.filter(|previous_id| *previous_id != node_id) {
            if self.component_states.set_focused(previous_id, false) {
                mark_component_focus_render_dirty(self, previous_id);
            }
        }
        if self.component_states.set_focused(node_id, true) {
            mark_component_focus_render_dirty(self, node_id);
        }

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
        if self.component_states.set_focused(previous, false) {
            mark_component_focus_render_dirty(self, previous);
        }
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
        self.focus.focused_inputs.push(event.clone());
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
            (node.is_focus_candidate() || self.is_open_mui_modal_focus_root(node_id))
                && is_valid_input_owner(self, node_id)
        })
    }

    pub(crate) fn apply_mui_modal_focus_transition(
        &mut self,
        node_id: UiNodeId,
        open: bool,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        if !self.is_mui_modal_focus_component(node_id)? {
            return Ok(None);
        }
        if open {
            return self.open_mui_modal_focus_scope(node_id);
        }
        self.close_mui_modal_focus_scope(node_id)
    }

    fn open_mui_modal_focus_scope(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        let restore = self.focus.focused;
        if let Some(existing) = self
            .focus
            .modal_restore_stack
            .iter_mut()
            .find(|entry| entry.modal == node_id)
        {
            existing.restore = restore;
        } else {
            self.focus.modal_restore_stack.push(
                zircon_runtime_interface::ui::surface::UiModalFocusRestoreState {
                    modal: node_id,
                    restore,
                },
            );
        }

        if self.mui_modal_bool_attribute(node_id, "disable_auto_focus")? {
            return Ok(None);
        }
        if self
            .focus
            .focused
            .is_some_and(|focused| self.tree.node_is_descendant_of(node_id, focused))
        {
            return Ok(None);
        }
        let target = self
            .tree
            .first_focusable_in_subtree(node_id)?
            .unwrap_or(node_id);
        self.focus_node_with_reason(
            target,
            UiFocusChangeReason::Autofocus,
            UiFocusVisible::visible(UiFocusVisibleReason::Programmatic),
        )
    }

    fn close_mui_modal_focus_scope(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        let restore = self.take_mui_modal_restore_target(node_id);
        if !self.mui_modal_bool_attribute(node_id, "disable_restore_focus")? {
            if let Some(restore) = restore.filter(|restore| self.is_focus_target(*restore)) {
                return self.focus_node_with_reason(
                    restore,
                    UiFocusChangeReason::Programmatic,
                    UiFocusVisible::visible(UiFocusVisibleReason::Programmatic),
                );
            }
        }
        if self
            .focus
            .focused
            .is_some_and(|focused| self.tree.node_is_descendant_of(node_id, focused))
        {
            return Ok(self.clear_focus_with_reason(UiFocusChangeReason::Clear));
        }
        Ok(None)
    }

    fn take_mui_modal_restore_target(&mut self, node_id: UiNodeId) -> Option<UiNodeId> {
        let index = self
            .focus
            .modal_restore_stack
            .iter()
            .rposition(|entry| entry.modal == node_id)?;
        self.focus.modal_restore_stack.remove(index).restore
    }

    fn enforced_mui_modal_focus_target(
        &self,
        requested: UiNodeId,
    ) -> Result<UiNodeId, UiTreeError> {
        let Some(root) = self.tree.active_mui_modal_root(Some(requested)) else {
            return Ok(requested);
        };
        if self.tree.node_is_descendant_of(root, requested) {
            return Ok(requested);
        }
        Ok(self.tree.first_focusable_in_subtree(root)?.unwrap_or(root))
    }

    fn is_open_mui_modal_focus_root(&self, node_id: UiNodeId) -> bool {
        self.tree.node(node_id).is_some_and(|node| {
            node.template_metadata.as_ref().is_some_and(|metadata| {
                is_mui_modal_focus_component(metadata.component.as_str())
                    && node.state_flags.enabled
                    && node.is_render_visible()
                    && (bool_attribute(metadata, "open") || bool_attribute(metadata, "popup_open"))
            })
        })
    }

    fn is_mui_modal_focus_component(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        Ok(node
            .template_metadata
            .as_ref()
            .is_some_and(|metadata| is_mui_modal_focus_component(metadata.component.as_str())))
    }

    fn mui_modal_bool_attribute(&self, node_id: UiNodeId, key: &str) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        Ok(node.template_metadata.as_ref().is_some_and(|metadata| {
            bool_attribute_any(metadata, mui_modal_bool_attribute_aliases(key))
        }))
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

fn is_mui_modal_focus_component(component: &str) -> bool {
    matches!(component, "Dialog" | "Modal" | "Popover" | "Menu")
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

fn mui_modal_bool_attribute_aliases(key: &str) -> &[&str] {
    match key {
        "disable_auto_focus" => &["disable_auto_focus", "disableAutoFocus"],
        "disable_enforce_focus" => &["disable_enforce_focus", "disableEnforceFocus"],
        "disable_restore_focus" => &["disable_restore_focus", "disableRestoreFocus"],
        _ => &[],
    }
}
