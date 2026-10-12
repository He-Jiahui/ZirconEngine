//! V2 child surfaces share the existing surface owner; retired nodes are not pooled.
use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    focus::UiFocusChangeReason,
    layout::{UiAlignment, UiAlignment2D, UiSlot, UiSlotKind},
    tree::{UiTree, UiTreeError},
    v2::UiV2AssetError,
};

use crate::ui::tree::UiRuntimeTreeLayoutExt;

use super::{session_identity::UiSurfaceSessionIdentityHandle, UiInvalidationReason, UiSurface};

/// A commit proof for the original surface's value state, session and retired incarnations.
/// Private live leases remain with that owner until a successful commit.
pub struct UiSurfaceChildMountProof {
    source_session: UiSurfaceSessionIdentityHandle,
    tree_id: UiTreeId,
    source_state: Box<UiSurface>,
    retired_nodes: Vec<UiNodeId>,
    retired_incarnations: BTreeMap<UiNodeId, u64>,
}

impl std::fmt::Debug for UiSurfaceChildMountProof {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UiSurfaceChildMountProof")
            .field("tree_id", &self.tree_id)
            .field("retired_nodes", &self.retired_nodes)
            .finish_non_exhaustive()
    }
}

impl UiSurfaceChildMountProof {
    pub(crate) fn new(surface: &UiSurface, retired_nodes: Vec<UiNodeId>) -> Self {
        let retired_incarnations = retired_nodes
            .iter()
            .filter_map(|id| {
                surface
                    .tree
                    .node_incarnation(*id)
                    .map(|incarnation| (*id, incarnation))
            })
            .collect();
        Self {
            source_session: surface.session_identity.handle(),
            tree_id: surface.tree.tree_id.clone(),
            source_state: Box::new(surface.clone()),
            retired_nodes,
            retired_incarnations,
        }
    }

    pub fn retired_nodes(&self) -> &[UiNodeId] {
        &self.retired_nodes
    }
}

impl UiSurface {
    pub(crate) fn replace_v2_child_surface(
        &mut self,
        parent_id: UiNodeId,
        previous_root_id: Option<UiNodeId>,
        incoming: Option<Self>,
    ) -> Result<(Option<UiNodeId>, Vec<UiNodeId>), UiV2AssetError> {
        let asset_id = self.tree.tree_id.0.clone();
        let invalid = |detail: String| UiV2AssetError::InvalidDocument {
            asset_id: asset_id.clone(),
            detail,
        };
        let parent_path = self
            .tree
            .node(parent_id)
            .ok_or_else(|| invalid(format!("missing child-surface parent {parent_id:?}")))?
            .node_path
            .0
            .clone();
        let retired = if let Some(root_id) = previous_root_id {
            if self.tree.node(root_id).and_then(|node| node.parent) != Some(parent_id) {
                return Err(invalid(
                    "previous child surface belongs to a different parent".into(),
                ));
            }
            connected_subtree(&self.tree, root_id).map_err(|error| invalid(error.to_string()))?
        } else {
            Vec::new()
        };
        let retired_set = retired.iter().copied().collect::<BTreeSet<_>>();
        let incoming_nodes = if let Some(source) = incoming.as_ref() {
            if source.binding_program().node_count() != 0
                || source.binding_program().iter_bindings().len() != 0
                || source.tree.roots.len() != 1
                || source.runtime_style.has_mounted_child_indices()
            {
                return Err(invalid(
                    "child mount requires one fresh V2 root without compiled bindings".into(),
                ));
            }
            if source
                .tree
                .node(source.tree.roots[0])
                .is_some_and(|node| node.parent.is_some())
            {
                return Err(invalid("child surface root already has a parent".into()));
            }
            let nodes = connected_subtree(&source.tree, source.tree.roots[0])
                .map_err(|error| invalid(error.to_string()))?;
            if nodes.len() != source.tree.nodes.len() {
                return Err(invalid("child surface contains disconnected nodes".into()));
            }
            let mut controls = self
                .tree
                .nodes
                .iter()
                .filter(|(id, _)| !retired_set.contains(id))
                .filter_map(|(_, node)| node.template_metadata.as_ref()?.control_id.clone())
                .collect::<BTreeSet<_>>();
            for node_id in &nodes {
                let node = source.tree.node(*node_id).expect("validated incoming node");
                if let Some(control_id) = node
                    .template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_ref())
                {
                    if !controls.insert(control_id.clone()) {
                        return Err(invalid(format!("duplicate mounted control {control_id}")));
                    }
                }
            }
            for slot in source.tree.layout_slots() {
                if source.tree.node(slot.child_id).and_then(|node| node.parent)
                    != Some(slot.parent_id)
                {
                    return Err(invalid(
                        "child surface slot disagrees with its node parent".into(),
                    ));
                }
            }
            nodes
        } else {
            Vec::new()
        };

        // Retire before allocating so repeated switches reuse the same bounded numeric range.
        self.reset_detached_transient_state_for_nodes(&retired, UiFocusChangeReason::Despawned);
        self.component_states.clear_nodes(&retired);
        self.input.retire_node_input_records(&retired_set);
        if self
            .focus
            .previous
            .is_some_and(|id| retired_set.contains(&id))
        {
            self.focus.previous = None;
        }
        if self
            .focus
            .pending_autofocus
            .is_some_and(|id| retired_set.contains(&id))
        {
            self.focus.pending_autofocus = None;
        }
        self.focus.focused_inputs.retain(|input| {
            !retired_set.contains(&input.focused)
                && !input.route.iter().any(|id| retired_set.contains(id))
                && !input.handled_by.is_some_and(|id| retired_set.contains(&id))
        });
        self.focus
            .modal_restore_stack
            .retain(|state| !retired_set.contains(&state.modal));
        for state in &mut self.focus.modal_restore_stack {
            if state.restore.is_some_and(|id| retired_set.contains(&id)) {
                state.restore = None;
                state.restore_path = None;
            }
        }
        self.input.popup_stack.retain(|popup| {
            !popup.popup_node.is_some_and(|id| retired_set.contains(&id))
                && !popup.owner.is_some_and(|id| retired_set.contains(&id))
        });
        if let Some(root_id) = previous_root_id {
            self.runtime_style.retire_child_index(root_id);
            self.tree
                .nodes
                .get_mut(&parent_id)
                .expect("validated parent")
                .children
                .retain(|child| *child != root_id);
            self.tree.retain_layout_slots(|slot| {
                !retired_set.contains(&slot.parent_id) && !retired_set.contains(&slot.child_id)
            });
            for node_id in &retired {
                self.tree.nodes.remove(node_id);
                self.dirty_node_ids.remove(node_id);
            }
            self.prune_removed_virtual_list_materialization_owners();
        }
        self.tree.mark_layout_order_changed(parent_id);
        self.invalidation
            .record_reason(parent_id, UiInvalidationReason::Structure);
        self.tree
            .mark_layout_dirty(parent_id)
            .map_err(|error| invalid(error.to_string()))?;

        let Some(source) = incoming else {
            return Ok((None, retired));
        };
        let first_id = self
            .tree
            .nodes
            .keys()
            .map(|id| id.0)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| invalid("child surface node ID overflow".into()))?;
        let mut node_ids = BTreeMap::new();
        for (offset, source_id) in incoming_nodes.iter().enumerate() {
            let id = first_id
                .checked_add(offset as u64)
                .ok_or_else(|| invalid("child surface node ID overflow".into()))?;
            node_ids.insert(*source_id, UiNodeId::new(id));
        }
        let source_root = source.tree.roots[0];
        let root_id = node_ids[&source_root];
        let mut incoming_tree = UiTree::new(self.tree.tree_id.clone());
        for source_id in &incoming_nodes {
            let source_node = source
                .tree
                .node(*source_id)
                .expect("validated incoming node");
            let mut node = source_node.clone();
            node.node_id = node_ids[source_id];
            node.node_path =
                UiNodePath::new(format!("{parent_path}/mounted/{}", source_node.node_path.0));
            node.children.clear();
            if *source_id == source_root {
                incoming_tree.insert_root(node.clone());
                self.tree
                    .insert_child(parent_id, node)
                    .map_err(|error| invalid(error.to_string()))?;
            } else {
                let parent = node_ids[&source_node.parent.expect("validated incoming parent")];
                incoming_tree
                    .insert_child(parent, node.clone())
                    .map_err(|error| invalid(error.to_string()))?;
                self.tree
                    .insert_child(parent, node)
                    .map_err(|error| invalid(error.to_string()))?;
            }
            self.invalidation
                .record_reason(node_ids[source_id], UiInvalidationReason::Structure);
        }
        for source_slot in source.tree.layout_slots() {
            let mut slot = source_slot.clone();
            slot.parent_id = node_ids[&slot.parent_id];
            slot.child_id = node_ids[&slot.child_id];
            self.tree.push_layout_slot(slot);
        }
        let mut slot = UiSlot::new(parent_id, root_id, UiSlotKind::Overlay);
        slot.alignment = UiAlignment2D::new(UiAlignment::Fill, UiAlignment::Fill);
        self.tree.push_layout_slot(slot);
        self.runtime_style
            .mount_child_index(root_id, source.runtime_style, &node_ids);
        self.component_states
            .seed_from_tree_metadata(&incoming_tree);
        self.apply_runtime_state_style_all(true)
            .map_err(|error| invalid(error.to_string()))?;
        Ok((Some(root_id), retired))
    }

    pub(crate) fn adopt_live_child_mount_state(
        &mut self,
        current: &mut Self,
        proof: &UiSurfaceChildMountProof,
    ) -> Result<(), UiV2AssetError> {
        if proof.source_session != current.session_identity.handle()
            || proof.tree_id != current.tree.tree_id
            || self.tree.tree_id != current.tree.tree_id
            || proof.source_state.as_ref() != &*current
            || proof
                .retired_incarnations
                .iter()
                .any(|(id, incarnation)| current.tree.node_incarnation(*id) != Some(*incarnation))
        {
            return Err(UiV2AssetError::InvalidDocument {
                asset_id: proof.tree_id.0.clone(),
                detail: "child mount commit no longer matches the staged source state or session"
                    .into(),
            });
        }
        // UiSurface value equality covers tree, bindings, styles, focus, input and navigation.
        // Its private lease stores deliberately clone empty and compare equal; they are never
        // restored from the snapshot. Move the latest live stores only after every check passes.
        for node_id in &proof.retired_nodes {
            current.drop_clipboard_transfers_for(*node_id);
        }
        current
            .secure_text_values
            .retire_nodes(&proof.retired_nodes);
        self.session_identity = std::mem::take(&mut current.session_identity);
        self.clipboard_transfers = std::mem::take(&mut current.clipboard_transfers);
        self.secure_text_values = std::mem::take(&mut current.secure_text_values);
        self.reset_frame_publication_after_child_mount();
        Ok(())
    }
}

fn connected_subtree(tree: &UiTree, root_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
    let mut nodes = Vec::new();
    let mut seen = BTreeSet::new();
    let mut stack = vec![root_id];
    while let Some(node_id) = stack.pop() {
        if !seen.insert(node_id) {
            return Err(UiTreeError::DuplicateNode(node_id));
        }
        let node = tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        for child_id in &node.children {
            if tree.node(*child_id).and_then(|child| child.parent) != Some(node_id) {
                return Err(UiTreeError::MissingParent(node_id));
            }
        }
        nodes.push(node_id);
        stack.extend(node.children.iter().rev().copied());
    }
    Ok(nodes)
}
