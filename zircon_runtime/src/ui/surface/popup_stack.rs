use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
    widget::{UiPopupAnchor, UiWidgetBehavior},
};

use crate::ui::tree::UiRuntimeTreeFocusExt;

use super::UiSurface;

impl UiSurface {
    pub(crate) fn is_popup_stack_node(&self, node_id: UiNodeId) -> bool {
        self.tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .is_some_and(is_popup_stack_metadata)
    }

    pub(crate) fn popup_branch_closures(&self, popup_node_id: UiNodeId) -> Vec<(UiNodeId, String)> {
        if !self.tree.nodes.contains_key(&popup_node_id) {
            return Vec::new();
        }
        if !self.is_popup_stack_node(popup_node_id) {
            return Vec::new();
        }

        // Portal-backed nested menus can be siblings in the retained tree while the
        // Runtime popup stack still establishes their parent-child close order. Preserve
        // that LIFO order while collecting the complete branch in one pass.
        let stack_tail = self
            .input
            .popup_stack
            .iter()
            .position(|popup| popup.popup_node == Some(popup_node_id))
            .map(|stack_index| {
                self.input.popup_stack[stack_index + 1..]
                    .iter()
                    .rev()
                    .filter_map(|popup| popup.popup_node)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut closures = Vec::new();
        let mut visited = BTreeSet::new();
        for node_id in stack_tail {
            append_popup_subtree_closures(&self.tree, node_id, true, &mut visited, &mut closures);
        }

        // A stale input stack must not let an authored descendant reopen during the
        // next rebuild. Traverse only the closing popup branch and add unstacked nodes
        // after the LIFO stack tail, preserving the stack as the focus authority.
        append_popup_subtree_closures(
            &self.tree,
            popup_node_id,
            false,
            &mut visited,
            &mut closures,
        );
        closures
    }

    pub(crate) fn declarative_popup_closures(&self) -> Vec<(UiNodeId, String)> {
        let mut closures = Vec::new();
        let mut visited = BTreeSet::new();

        // The input stack owns interaction order. Close its topmost declarative
        // popup first, then let post-order traversal include any retained child
        // popup which missed stack reconciliation.
        for node_id in self
            .input
            .popup_stack
            .iter()
            .rev()
            .filter_map(|popup| popup.popup_node)
        {
            append_popup_subtree_closures(&self.tree, node_id, true, &mut visited, &mut closures);
        }

        // Programmatic dismissal must also clear declarative popups that were
        // authored open while their transient stack entry is stale or absent.
        for root_id in self.tree.roots.iter().rev().copied() {
            append_popup_subtree_closures(&self.tree, root_id, true, &mut visited, &mut closures);
        }
        closures
    }

    pub(crate) fn unique_popup_state_for_id(
        &self,
        popup_id: &str,
    ) -> Option<(UiNodeId, &'static str, bool)> {
        let mut matches = self.tree.nodes.iter().filter_map(|(node_id, node)| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| is_popup_stack_metadata(metadata))
                .filter(|_| popup_stack_id_for_node(node) == popup_id)
                .map(|_| *node_id)
        });
        let node_id = matches.next()?;
        matches
            .next()
            .is_none()
            .then(|| self.popup_state_for_node(node_id))
            .flatten()
            .map(|(property, open)| (node_id, property, open))
    }

    pub(crate) fn popup_state_for_node(&self, node_id: UiNodeId) -> Option<(&'static str, bool)> {
        let metadata = self
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())?;
        is_popup_stack_metadata(metadata).then(|| {
            (
                popup_open_property(metadata),
                tree_node_popup_open(metadata),
            )
        })
    }

    pub(crate) fn popup_route_owner_for_node(&self, node_id: UiNodeId) -> Option<UiNodeId> {
        if let Some(metadata) = self
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .filter(|metadata| is_popup_stack_metadata(metadata))
        {
            return self.popup_anchor_owner(node_id, metadata);
        }

        self.input
            .popup_stack
            .iter()
            .rev()
            .find(|popup| popup.popup_node == Some(node_id))
            .and_then(|popup| popup.owner)
    }

    pub(crate) fn seed_popup_stack_from_tree_metadata(&mut self) {
        let rejected_popups = self
            .tree
            .nodes
            .iter()
            .filter_map(|(node_id, node)| {
                let metadata = node.template_metadata.as_ref()?;
                (is_popup_stack_metadata(metadata)
                    && tree_node_popup_open(metadata)
                    && matches!(&metadata.widget.popup_anchor, UiPopupAnchor::Control { .. })
                    && self.popup_anchor_owner(*node_id, metadata).is_none())
                .then(|| (*node_id, popup_open_property(metadata)))
            })
            .collect::<Vec<_>>();
        for (node_id, property) in rejected_popups {
            let _ = self.reject_control_anchored_popup(node_id, property);
        }
        let open_popups = self
            .tree
            .nodes
            .iter()
            .filter_map(|(node_id, node)| self.popup_stack_record(*node_id, node))
            .collect::<Vec<_>>();

        self.input.popup_stack.retain(|popup| {
            let Some(popup_node) = popup.popup_node else {
                return true;
            };
            open_popups.iter().any(|record| {
                record.open
                    && record.popup_node == popup_node
                    && record.popup_id == popup.popup_id
                    && popup.owner == Some(record.owner)
            })
        });

        for record in open_popups.into_iter().filter(|record| record.open) {
            let already_open = self.input.popup_stack.iter().any(|popup| {
                popup.popup_node == Some(record.popup_node)
                    && popup.popup_id == record.popup_id
                    && popup.owner == Some(record.owner)
            });
            if !already_open {
                let _ = self.sync_popup_stack_for_node(record.popup_node, true);
            }
        }
    }

    pub(crate) fn sync_popup_stack_for_node(
        &mut self,
        node_id: UiNodeId,
        open: bool,
    ) -> Option<UiNodeId> {
        let Some(node) = self.tree.nodes.get(&node_id) else {
            return None;
        };
        let Some(metadata) = node.template_metadata.as_ref() else {
            return None;
        };
        if !is_popup_stack_metadata(metadata) {
            return None;
        }
        let popup_id = popup_stack_id_for_node(node);
        if open && node.state_flags.enabled && node.is_render_visible() {
            if let Some(owner) = self.popup_anchor_owner(node_id, metadata) {
                let first_open_descendant = self.input.popup_stack.iter().find_map(|popup| {
                    popup
                        .popup_node
                        .filter(|popup_node| self.tree.node_is_descendant_of(node_id, *popup_node))
                });
                self.input.synchronize_popup_with_node(
                    popup_id,
                    Some(owner),
                    node_id,
                    None,
                    first_open_descendant,
                );
                return Some(owner);
            } else {
                self.input.close_popup_with_node(node_id, popup_id.as_str());
            }
        } else {
            self.input.close_popup_with_node(node_id, popup_id.as_str());
        }
        None
    }

    pub(crate) fn popup_uses_control_anchor(&self, popup_node_id: UiNodeId) -> bool {
        let Some(node) = self.tree.nodes.get(&popup_node_id) else {
            return false;
        };
        let Some(metadata) = node.template_metadata.as_ref() else {
            return false;
        };
        is_popup_stack_metadata(metadata)
            && matches!(&metadata.widget.popup_anchor, UiPopupAnchor::Control { .. })
    }

    pub(crate) fn popup_trigger_requires_full_render_extract(
        &self,
        changed_node_ids: &std::collections::BTreeSet<UiNodeId>,
    ) -> bool {
        if changed_node_ids.is_empty() {
            return false;
        }

        // A popup only depends on its resolved trigger, its ancestor geometry and a
        // potential competing control id. Rebuilding every open popup after an
        // unrelated status/text repaint turns one local update into an O(N) extract.
        for popup in &self.input.popup_stack {
            if popup.popup_node.is_some_and(|popup_node| {
                self.changed_node_is_ancestor_of(changed_node_ids, popup_node)
            }) {
                return true;
            }
            let Some(control_id) = self.popup_control_anchor_id(popup.popup_node) else {
                continue;
            };
            let Some(owner) = popup.owner else {
                return true;
            };
            if changed_node_ids.contains(&owner) {
                return true;
            }
            let Some(owner_node) = self.tree.nodes.get(&owner) else {
                return true;
            };
            if owner_node
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                != Some(control_id)
            {
                return true;
            }
            if changed_node_ids
                .iter()
                .copied()
                .any(|node_id| self.changed_node_affects_popup_owner(node_id, owner, control_id))
            {
                return true;
            }
        }

        changed_node_ids.iter().copied().any(|node_id| {
            self.tree
                .nodes
                .get(&node_id)
                .is_none_or(|node| is_open_control_anchored_popup(node))
        })
    }

    pub(crate) fn popup_stack_requires_reconciliation(
        &self,
        changed_node_ids: &std::collections::BTreeSet<UiNodeId>,
    ) -> bool {
        if changed_node_ids.is_empty() {
            return false;
        }

        for popup in &self.input.popup_stack {
            if popup.popup_node.is_some_and(|popup_node| {
                self.changed_node_is_ancestor_of(changed_node_ids, popup_node)
            }) {
                return true;
            }
            let Some(control_id) = self.popup_control_anchor_id(popup.popup_node) else {
                continue;
            };
            let Some(owner) = popup.owner else {
                return true;
            };
            if changed_node_ids.contains(&owner) {
                return true;
            }
            if self
                .tree
                .nodes
                .get(&owner)
                .and_then(|node| node.template_metadata.as_ref())
                .and_then(|metadata| metadata.control_id.as_deref())
                != Some(control_id)
            {
                return true;
            }
            if changed_node_ids
                .iter()
                .copied()
                .any(|node_id| self.changed_node_affects_popup_owner(node_id, owner, control_id))
            {
                return true;
            }
        }

        changed_node_ids.iter().copied().any(|node_id| {
            self.tree
                .nodes
                .get(&node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .is_some_and(is_popup_stack_metadata)
        })
    }

    fn popup_control_anchor_id(&self, popup_node_id: Option<UiNodeId>) -> Option<&str> {
        let metadata = self
            .tree
            .nodes
            .get(&popup_node_id?)
            .and_then(|node| node.template_metadata.as_ref())?;
        let UiPopupAnchor::Control { control_id } = &metadata.widget.popup_anchor else {
            return None;
        };
        Some(control_id)
    }

    fn changed_node_affects_popup_owner(
        &self,
        changed_node_id: UiNodeId,
        owner: UiNodeId,
        control_id: &str,
    ) -> bool {
        let Some(changed_node) = self.tree.nodes.get(&changed_node_id) else {
            return true;
        };
        if changed_node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some(control_id)
        {
            return true;
        }

        let mut current = Some(owner);
        while let Some(node_id) = current {
            if node_id == changed_node_id {
                return true;
            }
            current = self.tree.nodes.get(&node_id).and_then(|node| node.parent);
        }
        false
    }

    fn changed_node_is_ancestor_of(
        &self,
        changed_node_ids: &std::collections::BTreeSet<UiNodeId>,
        descendant: UiNodeId,
    ) -> bool {
        let mut current = Some(descendant);
        while let Some(node_id) = current {
            if changed_node_ids.contains(&node_id) {
                return true;
            }
            let Some(node) = self.tree.nodes.get(&node_id) else {
                return true;
            };
            current = node.parent;
        }
        false
    }

    pub(crate) fn popup_anchor_owner(
        &self,
        popup_node_id: UiNodeId,
        metadata: &UiTemplateNodeMetadata,
    ) -> Option<UiNodeId> {
        let UiPopupAnchor::Control { control_id } = &metadata.widget.popup_anchor else {
            return Some(popup_node_id);
        };
        let trigger_id = self
            .control_index
            .unique_node_id_for_surface(&self.tree, control_id)?;
        self.popup_trigger_is_valid(trigger_id)
            .then_some(trigger_id)
    }

    fn popup_stack_record(&self, node_id: UiNodeId, node: &UiTreeNode) -> Option<PopupStackRecord> {
        let metadata = node.template_metadata.as_ref()?;
        let (_, (popup_id, open)) = tree_popup_stack_record(node_id, node)?;
        Some(PopupStackRecord {
            popup_node: node_id,
            popup_id,
            owner: self.popup_anchor_owner(node_id, metadata)?,
            open,
        })
    }

    fn popup_trigger_is_valid(&self, node_id: UiNodeId) -> bool {
        let mut current = Some(node_id);
        while let Some(current_id) = current {
            let Some(node) = self.tree.nodes.get(&current_id) else {
                return false;
            };
            if !node.state_flags.enabled || !node.is_render_visible() {
                return false;
            }
            current = node.parent;
        }
        true
    }
}

fn append_popup_subtree_closures(
    tree: &UiTree,
    root_id: UiNodeId,
    include_root: bool,
    visited: &mut BTreeSet<UiNodeId>,
    closures: &mut Vec<(UiNodeId, String)>,
) {
    let mut traversal = vec![(root_id, false)];
    while let Some((node_id, visiting_children)) = traversal.pop() {
        let Some(node) = tree.nodes.get(&node_id) else {
            continue;
        };
        if visiting_children {
            if (include_root || node_id != root_id)
                && node.template_metadata.as_ref().is_some_and(|metadata| {
                    is_popup_stack_metadata(metadata) && tree_node_popup_open(metadata)
                })
            {
                let metadata = node.template_metadata.as_ref().expect("checked above");
                closures.push((node_id, popup_open_property(metadata).to_string()));
            }
            continue;
        }
        if !visited.insert(node_id) {
            continue;
        }
        traversal.push((node_id, true));
        traversal.extend(
            node.children
                .iter()
                .rev()
                .copied()
                .map(|child| (child, false)),
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PopupStackRecord {
    popup_node: UiNodeId,
    popup_id: String,
    owner: UiNodeId,
    open: bool,
}

fn tree_popup_stack_record(
    node_id: UiNodeId,
    node: &UiTreeNode,
) -> Option<(UiNodeId, (String, bool))> {
    let metadata = node.template_metadata.as_ref()?;
    is_popup_stack_metadata(metadata).then_some((
        node_id,
        (
            popup_stack_id_for_node(node),
            tree_node_popup_open(metadata) && node.state_flags.enabled && node.is_render_visible(),
        ),
    ))
}

pub(super) fn is_popup_stack_metadata(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::Popup
        || matches!(
            metadata.component.as_str(),
            "Dialog" | "ConfirmDialog" | "Modal" | "Popover"
        )
}

fn is_open_control_anchored_popup(node: &UiTreeNode) -> bool {
    let Some(metadata) = node.template_metadata.as_ref() else {
        return false;
    };
    is_popup_stack_metadata(metadata)
        && tree_node_popup_open(metadata)
        && node.state_flags.enabled
        && node.is_render_visible()
        && matches!(&metadata.widget.popup_anchor, UiPopupAnchor::Control { .. })
}

fn tree_node_popup_open(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "popup_open") || bool_attribute(metadata, "open")
}

fn popup_open_property(metadata: &UiTemplateNodeMetadata) -> &'static str {
    if metadata.attributes.contains_key("popup_open")
        || metadata.widget.open_property.as_deref() == Some("popup_open")
    {
        "popup_open"
    } else {
        "open"
    }
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> bool {
    metadata
        .attributes
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn popup_stack_id_for_node(node: &UiTreeNode) -> String {
    if node.node_path.0.is_empty() {
        format!("node:{}", node.node_id.0)
    } else {
        node.node_path.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath},
        tree::{UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
    };

    use super::tree_popup_stack_record;

    fn open_popup_node(node_id: UiNodeId) -> UiTreeNode {
        let mut metadata = UiTemplateNodeMetadata {
            component: "Popover".to_string(),
            ..UiTemplateNodeMetadata::default()
        };
        metadata
            .attributes
            .insert("open".to_string(), toml::Value::Boolean(true));
        UiTreeNode::new(node_id, UiNodePath::new("root/popup")).with_template_metadata(metadata)
    }

    #[test]
    fn disabled_popup_owner_is_not_seeded_into_runtime_stack() {
        let mut node = open_popup_node(UiNodeId::new(7));
        node.state_flags.enabled = false;

        assert_eq!(
            tree_popup_stack_record(node.node_id, &node),
            Some((node.node_id, ("root/popup".to_string(), false)))
        );
    }

    #[test]
    fn collapsed_popup_owner_is_not_seeded_into_runtime_stack() {
        let node = open_popup_node(UiNodeId::new(7)).with_visibility(UiVisibility::Collapsed);

        assert_eq!(
            tree_popup_stack_record(node.node_id, &node),
            Some((node.node_id, ("root/popup".to_string(), false)))
        );
    }
}
