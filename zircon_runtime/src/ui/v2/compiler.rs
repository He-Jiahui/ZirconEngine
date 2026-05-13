use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::v2::{
    UiV2ArenaChild, UiV2ArenaNode, UiV2AssetDocument, UiV2AssetError, UiV2CompiledDocument,
    UiV2ComponentGraph, UiV2ComponentGraphNode, UiV2NodeArena, UiV2NodeHandle,
};

use super::{cache::UiV2PrototypeStore, component_instancer::UiV2ComponentInstancer};

#[derive(Default)]
pub struct UiV2DocumentCompiler;

impl UiV2DocumentCompiler {
    pub fn compile(document: &UiV2AssetDocument) -> Result<UiV2CompiledDocument, UiV2AssetError> {
        Self::compile_with_prototype_store(document, &UiV2PrototypeStore::new())
    }

    pub fn compile_with_prototype_store(
        document: &UiV2AssetDocument,
        store: &UiV2PrototypeStore,
    ) -> Result<UiV2CompiledDocument, UiV2AssetError> {
        let document = UiV2ComponentInstancer::instantiate_document(document, store)?;
        let node_handles = node_handles(&document)?;
        if let Some(root) = document.root_node_id() {
            validate_reachable_root(&document, &node_handles, root)?;
        }
        for (component_name, component) in &document.components {
            validate_reachable_root(&document, &node_handles, &component.root)
                .map_err(|error| with_component_context(error, component_name.as_str()))?;
        }
        let arena = arena_from_document(&document, &node_handles)?;
        let component_graph = component_graph_from_arena(&arena);
        Ok(UiV2CompiledDocument {
            asset_id: document.asset.id.clone(),
            arena,
            node_handles,
            component_graph,
        })
    }
}

fn node_handles(
    document: &UiV2AssetDocument,
) -> Result<BTreeMap<String, UiV2NodeHandle>, UiV2AssetError> {
    if document.nodes.len() > u32::MAX as usize {
        return Err(UiV2AssetError::InvalidDocument {
            asset_id: document.asset.id.clone(),
            detail: "node table exceeds u32 handle capacity".to_string(),
        });
    }

    Ok(document
        .nodes
        .keys()
        .enumerate()
        .map(|(index, node_id)| (node_id.clone(), UiV2NodeHandle::new(index as u32)))
        .collect())
}

fn validate_reachable_root(
    document: &UiV2AssetDocument,
    node_handles: &BTreeMap<String, UiV2NodeHandle>,
    root: &str,
) -> Result<(), UiV2AssetError> {
    if !node_handles.contains_key(root) {
        return Err(UiV2AssetError::MissingNode {
            asset_id: document.asset.id.clone(),
            node_id: root.to_string(),
        });
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut parent_of = BTreeMap::new();
    let mut stack = vec![VisitFrame::Enter(root.to_string())];
    while let Some(frame) = stack.pop() {
        match frame {
            VisitFrame::Enter(node_id) => {
                if visited.contains(&node_id) {
                    continue;
                }
                if !visiting.insert(node_id.clone()) {
                    return Err(UiV2AssetError::InvalidDocument {
                        asset_id: document.asset.id.clone(),
                        detail: format!("ui v2 graph contains a cycle at {node_id}"),
                    });
                }
                let node =
                    document
                        .nodes
                        .get(&node_id)
                        .ok_or_else(|| UiV2AssetError::MissingNode {
                            asset_id: document.asset.id.clone(),
                            node_id: node_id.clone(),
                        })?;
                if node.component.is_empty() {
                    return Err(UiV2AssetError::InvalidDocument {
                        asset_id: document.asset.id.clone(),
                        detail: format!("node {node_id} has an empty component"),
                    });
                }
                stack.push(VisitFrame::Exit(node_id.clone()));
                for child in node.children.iter().rev() {
                    if !node_handles.contains_key(&child.node) {
                        return Err(UiV2AssetError::MissingNode {
                            asset_id: document.asset.id.clone(),
                            node_id: child.node.clone(),
                        });
                    }
                    if let Some(existing_parent) =
                        parent_of.insert(child.node.clone(), node_id.clone())
                    {
                        if existing_parent != node_id {
                            return Err(UiV2AssetError::InvalidDocument {
                                asset_id: document.asset.id.clone(),
                                detail: format!(
                                    "node {} is mounted by both {} and {}",
                                    child.node, existing_parent, node_id
                                ),
                            });
                        }
                    }
                    stack.push(VisitFrame::Enter(child.node.clone()));
                }
            }
            VisitFrame::Exit(node_id) => {
                let _ = visiting.remove(&node_id);
                let _ = visited.insert(node_id);
            }
        }
    }
    Ok(())
}

fn arena_from_document(
    document: &UiV2AssetDocument,
    node_handles: &BTreeMap<String, UiV2NodeHandle>,
) -> Result<UiV2NodeArena, UiV2AssetError> {
    let mut nodes = vec![UiV2ArenaNode::default(); node_handles.len()];
    for (node_id, node) in &document.nodes {
        let handle = node_handles[node_id];
        let children = node
            .children
            .iter()
            .map(|child| {
                let child_handle = node_handles.get(&child.node).copied().ok_or_else(|| {
                    UiV2AssetError::MissingNode {
                        asset_id: document.asset.id.clone(),
                        node_id: child.node.clone(),
                    }
                })?;
                Ok(UiV2ArenaChild {
                    child: child_handle,
                    slot: child.slot.clone(),
                })
            })
            .collect::<Result<_, UiV2AssetError>>()?;
        nodes[handle.index()] = UiV2ArenaNode {
            source_id: node_id.clone(),
            component: node.component.clone(),
            control_id: node.control_id.clone(),
            classes: node.classes.clone(),
            props: node.props.clone(),
            state: node.state.clone(),
            layout: node.layout.clone(),
            style: node.style.clone(),
            slots: node.slots.clone(),
            events: node.events.clone(),
            children,
        };
    }

    Ok(UiV2NodeArena {
        root: document
            .root_node_id()
            .and_then(|root| node_handles.get(root).copied()),
        nodes,
    })
}

fn with_component_context(error: UiV2AssetError, component_name: &str) -> UiV2AssetError {
    match error {
        UiV2AssetError::InvalidDocument { asset_id, detail } => UiV2AssetError::InvalidDocument {
            asset_id,
            detail: format!("component {component_name}: {detail}"),
        },
        other => other,
    }
}

fn component_graph_from_arena(arena: &UiV2NodeArena) -> UiV2ComponentGraph {
    let mut parents = vec![None; arena.nodes.len()];
    for (parent_index, node) in arena.nodes.iter().enumerate() {
        let parent = UiV2NodeHandle::new(parent_index as u32);
        for child in &node.children {
            if child.child.index() < parents.len() {
                parents[child.child.index()] = Some(parent);
            }
        }
    }

    UiV2ComponentGraph {
        root: arena.root,
        nodes: arena
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| UiV2ComponentGraphNode {
                handle: UiV2NodeHandle::new(index as u32),
                source_id: node.source_id.clone(),
                component: node.component.clone(),
                parent: parents[index],
                children: node.children.iter().map(|child| child.child).collect(),
            })
            .collect(),
    }
}

enum VisitFrame {
    Enter(String),
    Exit(String),
}
