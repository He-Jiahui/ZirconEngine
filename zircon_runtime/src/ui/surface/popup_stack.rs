use std::collections::{BTreeSet, HashMap, HashSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiPoint,
    tree::{UiDirtyFlags, UiTemplateNodeMetadata, UiTree, UiTreeError, UiTreeNode},
    widget::{UiPopupAnchor, UiWidgetBehavior},
};

use crate::ui::tree::UiRuntimeTreeFocusExt;

use super::UiSurface;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiPopupDependencyImpact {
    pub(crate) render_extract: bool,
    pub(crate) stack_reconciliation: bool,
}

impl UiSurface {
    /// Retargets a popup to an unambiguous, currently valid control without persisting geometry.
    ///
    /// Dynamic overlays such as hover tooltips use this when their trigger identity is known only
    /// at input time. The arranged tree remains the sole source of anchor geometry.
    pub fn set_popup_control_anchor(
        &mut self,
        popup_node_id: UiNodeId,
        control_id: impl Into<String>,
    ) -> Result<bool, UiTreeError> {
        let control_id = control_id.into();
        let Some(trigger_id) = self
            .control_index
            .unique_node_id_for_surface(&self.tree, control_id.as_str())
        else {
            return Ok(false);
        };
        if trigger_id == popup_node_id || !self.popup_trigger_is_valid(trigger_id) {
            return Ok(false);
        }

        let next_anchor = UiPopupAnchor::Control { control_id };
        let open = {
            let node = self
                .tree
                .nodes
                .get_mut(&popup_node_id)
                .ok_or(UiTreeError::MissingNode(popup_node_id))?;
            let Some(metadata) = node.template_metadata.as_mut() else {
                return Ok(false);
            };
            if !is_popup_stack_metadata(metadata) || metadata.widget.popup_anchor == next_anchor {
                return Ok(false);
            }
            metadata.widget.popup_anchor = next_anchor;
            tree_node_popup_open(metadata)
        };

        self.mark_node_dirty(
            popup_node_id,
            UiDirtyFlags {
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            },
        )?;
        if open {
            let _ = self.sync_popup_stack_for_node(popup_node_id, true);
        }
        Ok(true)
    }

    /// Captures the surface-space point used by a pointer-anchored popup.
    pub fn set_popup_pointer_anchor(
        &mut self,
        popup_node_id: UiNodeId,
        point: UiPoint,
    ) -> Result<bool, UiTreeError> {
        if !point.x.is_finite() || !point.y.is_finite() {
            return Ok(false);
        }
        let pointer_anchored = self
            .tree
            .nodes
            .get(&popup_node_id)
            .ok_or(UiTreeError::MissingNode(popup_node_id))?
            .template_metadata
            .as_ref()
            .is_some_and(|metadata| {
                is_popup_stack_metadata(metadata)
                    && matches!(&metadata.widget.popup_anchor, UiPopupAnchor::Pointer { .. })
            });
        if !pointer_anchored || self.input.popup_anchor_point(popup_node_id) == Some(point) {
            return Ok(false);
        }

        self.input.set_popup_anchor_point(popup_node_id, point);
        self.mark_node_dirty(
            popup_node_id,
            UiDirtyFlags {
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            },
        )?;
        if self
            .popup_state_for_node(popup_node_id)
            .is_some_and(|(_, open)| open)
        {
            let _ = self.sync_popup_stack_for_node(popup_node_id, true);
        }
        Ok(true)
    }

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
        let mut closures = Vec::new();
        let mut visited = BTreeSet::new();
        if let Some(stack_index) = self
            .input
            .popup_stack
            .iter()
            .position(|popup| popup.popup_node == Some(popup_node_id))
        {
            for node_id in self.input.popup_stack[stack_index + 1..]
                .iter()
                .rev()
                .filter_map(|popup| popup.popup_node)
            {
                append_popup_subtree_closures(
                    &self.tree,
                    node_id,
                    true,
                    &mut visited,
                    &mut closures,
                );
            }
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
                .filter(|_| popup_stack_id_matches(node, popup_id))
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
                    && popup_uses_runtime_anchor_metadata(metadata)
                    && (self.popup_anchor_owner(*node_id, metadata).is_none()
                        || (matches!(
                            &metadata.widget.popup_anchor,
                            UiPopupAnchor::Pointer { .. }
                        ) && self.input.popup_anchor_point(*node_id).is_none())))
                .then(|| (*node_id, popup_open_property(metadata)))
            })
            .collect::<Vec<_>>();
        for (node_id, property) in rejected_popups {
            let _ = self.reject_runtime_anchored_popup(node_id, property);
        }
        let open_popups = self
            .tree
            .nodes
            .iter()
            .filter_map(|(node_id, node)| self.popup_stack_record(*node_id, node))
            .collect::<Vec<_>>();
        let open_popup_by_node = open_popups
            .iter()
            .filter(|record| record.open)
            .map(|record| (record.popup_node, (record.popup_id.as_str(), record.owner)))
            .collect::<HashMap<_, _>>();

        self.input.popup_stack.retain(|popup| {
            let Some(popup_node) = popup.popup_node else {
                return true;
            };
            let Some((popup_id, owner)) = open_popup_by_node.get(&popup_node) else {
                return false;
            };
            popup.popup_id == *popup_id && popup.owner == Some(*owner)
        });
        drop(open_popup_by_node);

        let mut stacked_popup_nodes = self
            .input
            .popup_stack
            .iter()
            .filter_map(|popup| popup.popup_node)
            .collect::<HashSet<_>>();
        for record in open_popups.into_iter().filter(|record| record.open) {
            if stacked_popup_nodes.insert(record.popup_node) {
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
                let anchor = match &metadata.widget.popup_anchor {
                    UiPopupAnchor::Pointer { .. } => {
                        let Some(point) = self.input.popup_anchor_point(node_id) else {
                            self.input.close_popup_with_node(node_id, popup_id.as_str());
                            return None;
                        };
                        Some(point)
                    }
                    UiPopupAnchor::None
                    | UiPopupAnchor::Control { .. }
                    | UiPopupAnchor::Surface => None,
                };
                let first_open_descendant = self.input.popup_stack.iter().find_map(|popup| {
                    popup
                        .popup_node
                        .filter(|popup_node| self.tree.node_is_descendant_of(node_id, *popup_node))
                });
                self.input.synchronize_popup_with_node(
                    popup_id,
                    Some(owner),
                    node_id,
                    anchor,
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

    pub(crate) fn popup_uses_runtime_anchor(&self, popup_node_id: UiNodeId) -> bool {
        let Some(node) = self.tree.nodes.get(&popup_node_id) else {
            return false;
        };
        let Some(metadata) = node.template_metadata.as_ref() else {
            return false;
        };
        is_popup_stack_metadata(metadata) && popup_uses_runtime_anchor_metadata(metadata)
    }

    pub(crate) fn popup_dependency_impact(
        &self,
        changed_node_ids: &std::collections::BTreeSet<UiNodeId>,
    ) -> UiPopupDependencyImpact {
        if changed_node_ids.is_empty() {
            return UiPopupDependencyImpact::default();
        }

        let mut impact = UiPopupDependencyImpact::default();
        let mut changed_control_ids = HashSet::new();
        let mut changed_node_missing = false;
        for node_id in changed_node_ids.iter().copied() {
            let Some(node) = self.tree.nodes.get(&node_id) else {
                changed_node_missing = true;
                impact.render_extract = true;
                continue;
            };
            if let Some(metadata) = node.template_metadata.as_ref() {
                if is_popup_stack_metadata(metadata) {
                    impact.stack_reconciliation = true;
                }
                if let Some(control_id) = metadata.control_id.as_deref() {
                    changed_control_ids.insert(control_id);
                }
            }
            if is_open_runtime_anchored_popup(node) {
                impact.render_extract = true;
            }
        }
        if impact.render_extract && impact.stack_reconciliation {
            return impact;
        }

        // A popup only depends on its resolved trigger, its ancestor geometry and a
        // potential competing control id. Rebuilding every open popup after an
        // unrelated status/text repaint turns one local update into an O(N) extract.
        for popup in &self.input.popup_stack {
            if popup.popup_node.is_some_and(|popup_node| {
                self.changed_node_is_ancestor_of(changed_node_ids, popup_node)
            }) {
                return UiPopupDependencyImpact {
                    render_extract: true,
                    stack_reconciliation: true,
                };
            }
            let Some(control_id) = self.popup_control_anchor_id(popup.popup_node) else {
                continue;
            };
            let Some(owner) = popup.owner else {
                return UiPopupDependencyImpact {
                    render_extract: true,
                    stack_reconciliation: true,
                };
            };
            let Some(owner_node) = self.tree.nodes.get(&owner) else {
                return UiPopupDependencyImpact {
                    render_extract: true,
                    stack_reconciliation: true,
                };
            };
            if owner_node
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                != Some(control_id)
            {
                return UiPopupDependencyImpact {
                    render_extract: true,
                    stack_reconciliation: true,
                };
            }
            if changed_node_missing
                || changed_control_ids.contains(control_id)
                || self.changed_node_is_ancestor_of(changed_node_ids, owner)
            {
                return UiPopupDependencyImpact {
                    render_extract: true,
                    stack_reconciliation: true,
                };
            }
        }

        impact
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
        let control_id = match &metadata.widget.popup_anchor {
            UiPopupAnchor::None | UiPopupAnchor::Surface => return Some(popup_node_id),
            UiPopupAnchor::Control { control_id } => control_id.as_str(),
            UiPopupAnchor::Pointer { owner_property } => metadata
                .attributes
                .get(owner_property)
                .and_then(toml::Value::as_str)?,
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

fn is_open_runtime_anchored_popup(node: &UiTreeNode) -> bool {
    let Some(metadata) = node.template_metadata.as_ref() else {
        return false;
    };
    is_popup_stack_metadata(metadata)
        && tree_node_popup_open(metadata)
        && node.state_flags.enabled
        && node.is_render_visible()
        && popup_uses_runtime_anchor_metadata(metadata)
}

fn popup_uses_runtime_anchor_metadata(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        &metadata.widget.popup_anchor,
        UiPopupAnchor::Control { .. } | UiPopupAnchor::Surface | UiPopupAnchor::Pointer { .. }
    )
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

fn popup_stack_id_matches(node: &UiTreeNode, popup_id: &str) -> bool {
    if !node.node_path.0.is_empty() {
        return node.node_path.0.as_str() == popup_id;
    }
    let Some(encoded_node_id) = popup_id.strip_prefix("node:") else {
        return false;
    };
    if encoded_node_id.is_empty()
        || (encoded_node_id.len() > 1 && encoded_node_id.starts_with('0'))
        || !encoded_node_id.as_bytes().iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    encoded_node_id.parse::<u64>().ok() == Some(node.node_id.0)
}

#[cfg(test)]
#[path = "tests/popup_stack.rs"]
mod tests;
