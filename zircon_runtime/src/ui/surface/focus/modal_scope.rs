use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    focus::{UiFocusChangeEvent, UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
    navigation::UiNavigationGroupId,
    surface::UiModalFocusRestoreState,
    tree::{UiTemplateNodeMetadata, UiTreeError},
    widget::UiWidgetBehavior,
};

use crate::ui::tree::UiRuntimeTreeFocusExt;

use super::{UiSurface, bool_attribute_any, is_valid_input_owner};

impl UiSurface {
    pub(crate) fn apply_mui_modal_focus_transition(
        &mut self,
        node_id: UiNodeId,
        open: bool,
        restore_target: Option<UiNodeId>,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        let Some(root) = self.modal_focus_scope_root(node_id)? else {
            return Ok(None);
        };
        if open {
            return self.open_modal_focus_scope(node_id, root, restore_target);
        }
        self.close_modal_focus_scope(node_id, root)
    }

    pub(super) fn enforced_modal_focus_target(
        &self,
        requested: UiNodeId,
    ) -> Result<UiNodeId, UiTreeError> {
        let current = self.focus.focused.or(Some(requested));
        if self
            .tree
            .active_modal_focus_allows_target(current, requested)
        {
            return Ok(requested);
        }
        let Some(root) = self.tree.active_modal_focus_root(current) else {
            return Ok(requested);
        };
        if let Some(group_id) = self.tree.active_modal_navigation_group_id(current) {
            return Ok(self
                .first_valid_focusable_in_modal_group(&group_id)
                .unwrap_or(requested));
        }
        Ok(self
            .first_valid_focusable_in_subtree(root)?
            .unwrap_or(requested))
    }

    pub(super) fn is_open_modal_focus_root(&self, node_id: UiNodeId) -> bool {
        self.tree.node(node_id).is_some_and(|node| {
            let metadata_open = node.template_metadata.as_ref().is_some_and(|metadata| {
                (is_mui_modal_focus_metadata(metadata)
                    || node
                        .navigation
                        .group
                        .as_ref()
                        .is_some_and(|group| group.modal))
                    && (bool_attribute(metadata, "open") || bool_attribute(metadata, "popup_open"))
            });
            metadata_open && node.state_flags.enabled && node.is_render_visible()
        })
    }

    fn open_modal_focus_scope(
        &mut self,
        node_id: UiNodeId,
        root: UiNodeId,
        restore_target: Option<UiNodeId>,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        let restore = restore_target.or(self.focus.focused);
        let modal_path = self.tree.node(node_id).map(|node| node.node_path.clone());
        let restore_path = restore
            .and_then(|restore| self.tree.node(restore))
            .map(|node| node.node_path.clone());
        let state = UiModalFocusRestoreState {
            modal: node_id,
            modal_path,
            restore,
            restore_path,
        };
        if let Some(existing) = self.focus.modal_restore_stack.iter_mut().find(|entry| {
            state
                .modal_path
                .as_ref()
                .map_or(entry.modal == node_id, |path| {
                    entry.modal_path.as_ref() == Some(path)
                })
        }) {
            existing.modal = node_id;
            existing.modal_path = state.modal_path;
        } else {
            self.focus.modal_restore_stack.push(state);
        }

        if self.modal_bool_attribute(node_id, "disable_auto_focus")? {
            return Ok(None);
        }
        if self
            .focus
            .focused
            .is_some_and(|focused| self.modal_focus_scope_contains(node_id, root, focused))
        {
            return Ok(None);
        }
        let Some(target) = self.first_valid_focusable_in_scope(node_id, root)? else {
            return Ok(None);
        };
        self.focus_node_with_reason(
            target,
            UiFocusChangeReason::Autofocus,
            UiFocusVisible::hidden(UiFocusVisibleReason::Programmatic),
        )
    }

    fn close_modal_focus_scope(
        &mut self,
        node_id: UiNodeId,
        root: UiNodeId,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        let disable_restore_focus = self.modal_bool_attribute(node_id, "disable_restore_focus")?;
        let closing_scope_had_focus = self
            .focus
            .focused
            .is_some_and(|focused| self.modal_focus_scope_contains(node_id, root, focused));
        let restore = self.take_modal_restore_state(node_id, root, !disable_restore_focus);
        if !closing_scope_had_focus {
            return Ok(None);
        }
        if !disable_restore_focus {
            if let Some(restore) = restore.and_then(|restore| self.resolve_restore_target(&restore))
            {
                return self.focus_node_with_reason(
                    restore,
                    UiFocusChangeReason::Programmatic,
                    UiFocusVisible::hidden(UiFocusVisibleReason::Programmatic),
                );
            }
        }
        if let Some(target) = self.first_valid_focusable_in_active_modal_scope()? {
            return self.focus_node_with_reason(
                target,
                UiFocusChangeReason::Programmatic,
                UiFocusVisible::hidden(UiFocusVisibleReason::Programmatic),
            );
        }
        if self
            .focus
            .focused
            .is_some_and(|focused| self.modal_focus_scope_contains(node_id, root, focused))
        {
            return Ok(self.clear_focus_with_reason(UiFocusChangeReason::Clear));
        }
        Ok(None)
    }

    fn take_modal_restore_state(
        &mut self,
        node_id: UiNodeId,
        root: UiNodeId,
        preserve_restore: bool,
    ) -> Option<UiModalFocusRestoreState> {
        let node_path = self.tree.node(node_id).map(|node| &node.node_path);
        let index = self.focus.modal_restore_stack.iter().rposition(|entry| {
            entry
                .modal_path
                .as_ref()
                .map_or(entry.modal == node_id, |path| node_path == Some(path))
        })?;
        // Opening order and visual z-order can differ. Splice every restore edge that still
        // targets the closing scope instead of treating the Vec tail as the active modal.
        let dependent_indices = self
            .focus
            .modal_restore_stack
            .iter()
            .enumerate()
            .filter_map(|(candidate_index, candidate)| {
                (candidate_index != index
                    && self
                        .resolve_restore_target(candidate)
                        .is_some_and(|target| {
                            self.modal_focus_scope_contains(node_id, root, target)
                        }))
                .then_some(candidate_index)
            })
            .collect::<Vec<_>>();
        let state = self.focus.modal_restore_stack.remove(index);
        for dependent_index in dependent_indices {
            let adjusted_index = dependent_index - usize::from(dependent_index > index);
            let dependent = &mut self.focus.modal_restore_stack[adjusted_index];
            if preserve_restore {
                dependent.restore = state.restore;
                dependent.restore_path = state.restore_path.clone();
            } else {
                dependent.restore = None;
                dependent.restore_path = None;
            }
        }
        Some(state)
    }

    fn resolve_restore_target(&self, state: &UiModalFocusRestoreState) -> Option<UiNodeId> {
        if let Some(path) = state.restore_path.as_ref() {
            return self
                .tree
                .nodes
                .values()
                .find(|node| node.node_path == *path && self.is_focus_target(node.node_id))
                .map(|node| node.node_id);
        }
        state
            .restore
            .filter(|restore| self.is_focus_target(*restore))
    }

    fn modal_focus_scope_root(&self, node_id: UiNodeId) -> Result<Option<UiNodeId>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if node
            .template_metadata
            .as_ref()
            .is_some_and(is_mui_modal_focus_metadata)
        {
            return Ok(Some(node_id));
        }
        Ok(node
            .navigation
            .group
            .as_ref()
            .filter(|group| group.modal)
            .map(|group| group.root.unwrap_or(node_id)))
    }

    fn modal_focus_scope_contains(
        &self,
        node_id: UiNodeId,
        root: UiNodeId,
        candidate: UiNodeId,
    ) -> bool {
        self.modal_navigation_group_id(node_id).map_or_else(
            || self.tree.node_is_descendant_of(root, candidate),
            |group_id| {
                self.tree
                    .node_is_in_modal_navigation_group(candidate, group_id)
            },
        )
    }

    fn modal_navigation_group_id(&self, node_id: UiNodeId) -> Option<&UiNavigationGroupId> {
        self.tree
            .node(node_id)?
            .navigation
            .group
            .as_ref()
            .filter(|group| group.modal)
            .map(|group| &group.group_id)
    }

    fn first_valid_focusable_in_scope(
        &self,
        node_id: UiNodeId,
        root: UiNodeId,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        if let Some(group_id) = self.modal_navigation_group_id(node_id) {
            return Ok(self.first_valid_focusable_in_modal_group(group_id));
        }
        self.first_valid_focusable_in_subtree(root)
    }

    fn first_valid_focusable_in_active_modal_scope(&self) -> Result<Option<UiNodeId>, UiTreeError> {
        let Some(root) = self.tree.active_modal_focus_root(None) else {
            return Ok(None);
        };
        if let Some(group_id) = self.tree.active_modal_navigation_group_id(None) {
            return Ok(self.first_valid_focusable_in_modal_group(&group_id));
        }
        self.first_valid_focusable_in_subtree(root)
    }

    fn first_valid_focusable_in_modal_group(
        &self,
        group_id: &UiNavigationGroupId,
    ) -> Option<UiNodeId> {
        self.tree
            .nodes
            .values()
            .filter(|node| {
                node.is_focus_candidate()
                    && is_valid_input_owner(self, node.node_id)
                    && self
                        .tree
                        .node_is_in_modal_navigation_group(node.node_id, group_id)
            })
            .min_by_key(|node| (node.paint_order, node.node_id))
            .map(|node| node.node_id)
    }

    fn first_valid_focusable_in_subtree(
        &self,
        root: UiNodeId,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        let mut stack = vec![root];
        while let Some(node_id) = stack.pop() {
            let node = self
                .tree
                .node(node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if node.is_focus_candidate() && is_valid_input_owner(self, node_id) {
                return Ok(Some(node_id));
            }
            for child_id in node.children.iter().rev() {
                stack.push(*child_id);
            }
        }
        Ok(None)
    }

    fn modal_bool_attribute(&self, node_id: UiNodeId, key: &str) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        Ok(node.template_metadata.as_ref().is_some_and(|metadata| {
            bool_attribute_any(metadata, modal_bool_attribute_aliases(key))
        }))
    }
}

fn is_mui_modal_focus_metadata(metadata: &UiTemplateNodeMetadata) -> bool {
    is_mui_modal_focus_component(metadata.component.as_str())
        || metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::Popup
}

fn is_mui_modal_focus_component(component: &str) -> bool {
    matches!(
        component,
        "Dialog" | "ConfirmDialog" | "Modal" | "Popover" | "Menu"
    )
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> bool {
    metadata
        .attributes
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn modal_bool_attribute_aliases(key: &str) -> &[&str] {
    match key {
        "disable_auto_focus" => &["disable_auto_focus", "disableAutoFocus"],
        "disable_enforce_focus" => &["disable_enforce_focus", "disableEnforceFocus"],
        "disable_restore_focus" => &["disable_restore_focus", "disableRestoreFocus"],
        _ => &[],
    }
}
