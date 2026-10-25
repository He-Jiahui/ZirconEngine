use std::collections::HashSet;

use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
};

use super::AiBehaviorTreeEditError as Error;

pub(super) fn node_mut<'a>(
    tree: &'a mut AiBehaviorTreeDescriptor,
    id: &str,
) -> Result<&'a mut AiBehaviorNodeDescriptor, Error> {
    tree.nodes
        .iter_mut()
        .find(|node| node.id == id)
        .ok_or_else(|| Error::MissingNode { id: id.to_owned() })
}

fn parent_slot(tree: &AiBehaviorTreeDescriptor, child: &str) -> Result<(String, usize), Error> {
    tree.nodes
        .iter()
        .find_map(|node| {
            node.children
                .iter()
                .position(|id| id == child)
                .map(|index| (node.id.clone(), index))
        })
        .ok_or_else(|| Error::RootEdit {
            id: child.to_owned(),
        })
}

fn new_node(tree: &AiBehaviorTreeDescriptor, node: &AiBehaviorNodeDescriptor) -> Result<(), Error> {
    if tree.nodes.iter().any(|old| old.id == node.id) {
        return Err(Error::DuplicateNode {
            id: node.id.clone(),
        });
    }
    if !node.children.is_empty() {
        return Err(Error::NewNodeHasChildren {
            id: node.id.clone(),
        });
    }
    Ok(())
}

pub(super) fn has_explicit_child_weights(parent: &AiBehaviorNodeDescriptor) -> bool {
    parent.implementation == "random_selector"
        && parent.parameters.iter().any(|parameter| {
            parameter.key.starts_with("weight.") || parameter.key.starts_with("weight_")
        })
}

fn editable_children(parent: &AiBehaviorNodeDescriptor) -> Result<(), Error> {
    // Runtime resolves ID weights before positional weights. Reordering or wrapping must
    // not silently attach an existing probability to a different node identity.
    if has_explicit_child_weights(parent) {
        return Err(Error::WeightedTopology {
            id: parent.id.clone(),
        });
    }
    Ok(())
}

fn insert_slot(
    parent: &mut AiBehaviorNodeDescriptor,
    index: usize,
    child: &str,
) -> Result<(), Error> {
    if index > parent.children.len() {
        return Err(Error::ChildIndex {
            parent: parent.id.clone(),
            index,
            count: parent.children.len(),
        });
    }
    parent.children.insert(index, child.to_owned());
    Ok(())
}

pub(super) fn insert(
    tree: &mut AiBehaviorTreeDescriptor,
    parent: &str,
    index: usize,
    node: &AiBehaviorNodeDescriptor,
) -> Result<(), Error> {
    new_node(tree, node)?;
    let parent = node_mut(tree, parent)?;
    editable_children(parent)?;
    insert_slot(parent, index, &node.id)?;
    tree.nodes.push(node.clone());
    Ok(())
}

pub(super) fn reparent(
    tree: &mut AiBehaviorTreeDescriptor,
    child: &str,
    destination: &str,
    index: usize,
) -> Result<(), Error> {
    node_mut(tree, child)?;
    let (old_parent, old_index) = parent_slot(tree, child)?;
    if old_parent == destination && old_index == index {
        return Ok(());
    }
    editable_children(node_mut(tree, destination)?)?;
    let old = node_mut(tree, &old_parent)?;
    editable_children(old)?;
    old.children.remove(old_index);
    // Only the candidate is changed here. Full runtime topology validation below rejects
    // cycles, invalid child counts and unreachable nodes before anything is returned.
    insert_slot(node_mut(tree, destination)?, index, child)
}

pub(super) fn wrap(
    tree: &mut AiBehaviorTreeDescriptor,
    child: &str,
    decorator: &AiBehaviorNodeDescriptor,
) -> Result<(), Error> {
    node_mut(tree, child)?;
    new_node(tree, decorator)?;
    if decorator.kind != AiBehaviorNodeKind::Decorator {
        return Err(Error::NotDecorator {
            id: decorator.id.clone(),
        });
    }
    if tree.root_node == child {
        tree.root_node.clone_from(&decorator.id);
    } else {
        let (parent, index) = parent_slot(tree, child)?;
        let parent = node_mut(tree, &parent)?;
        editable_children(parent)?;
        parent.children[index].clone_from(&decorator.id);
    }
    let mut wrapper = decorator.clone();
    wrapper.children.push(child.to_owned());
    tree.nodes.push(wrapper);
    Ok(())
}

pub(super) fn unwrap(tree: &mut AiBehaviorTreeDescriptor, id: &str) -> Result<(), Error> {
    let node = node_mut(tree, id)?;
    if node.kind != AiBehaviorNodeKind::Decorator || node.children.len() != 1 {
        return Err(Error::NotDecorator { id: id.to_owned() });
    }
    let child = node.children[0].clone();
    if tree.root_node == id {
        tree.root_node = child;
    } else {
        let (parent, index) = parent_slot(tree, id)?;
        let parent = node_mut(tree, &parent)?;
        editable_children(parent)?;
        parent.children[index] = child;
    }
    tree.nodes.retain(|node| node.id != id);
    Ok(())
}

pub(super) fn delete(tree: &mut AiBehaviorTreeDescriptor, id: &str) -> Result<(), Error> {
    node_mut(tree, id)?;
    let (parent, index) = parent_slot(tree, id)?;
    let mut deleted = HashSet::new();
    let mut pending = vec![id.to_owned()];
    while let Some(id) = pending.pop() {
        if deleted.insert(id.clone()) {
            pending.extend(node_mut(tree, &id)?.children.iter().cloned());
        }
    }
    let parent = node_mut(tree, &parent)?;
    editable_children(parent)?;
    parent.children.remove(index);
    tree.nodes.retain(|node| !deleted.contains(&node.id));
    Ok(())
}
