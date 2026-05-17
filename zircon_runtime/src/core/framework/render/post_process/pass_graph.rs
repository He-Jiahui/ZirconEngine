use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::{
    PostProcessEffectKind, PostProcessGraphValidationError, PostProcessPassNode,
    PostProcessStackDescriptor,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PostProcessPassGraph {
    pub nodes: Vec<PostProcessPassNode>,
    pub skipped_nodes: Vec<PostProcessPassNode>,
    pub final_composite_node: Option<String>,
}

impl PostProcessPassGraph {
    pub fn validate_stack(
        stack: &PostProcessStackDescriptor,
    ) -> Result<Self, PostProcessGraphValidationError> {
        let enabled_nodes = stack
            .effects
            .iter()
            .filter(|effect| effect.enabled)
            .map(PostProcessPassNode::from_settings)
            .collect::<Vec<_>>();
        let skipped_nodes = stack
            .effects
            .iter()
            .filter(|effect| !effect.enabled)
            .map(PostProcessPassNode::from_settings)
            .collect::<Vec<_>>();
        let order = ordered_node_indices(&enabled_nodes)?;
        let mut available = stack
            .initial_resources
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let initial_resources = available.clone();
        let mut produced = BTreeSet::new();
        let mut ordered_nodes = Vec::with_capacity(enabled_nodes.len());

        for index in order {
            let node = enabled_nodes[index].clone();
            for resource in &node.required_inputs {
                if !available.contains(resource) {
                    return Err(PostProcessGraphValidationError::MissingRequiredInput {
                        node: node.name.clone(),
                        resource: resource.clone(),
                    });
                }
            }
            for resource in &node.produced_outputs {
                if initial_resources.contains(resource) {
                    return Err(PostProcessGraphValidationError::DuplicateOutputResource {
                        node: node.name.clone(),
                        resource: resource.clone(),
                    });
                }
                if !produced.insert(resource.clone()) {
                    return Err(PostProcessGraphValidationError::DuplicateOutputResource {
                        node: node.name.clone(),
                        resource: resource.clone(),
                    });
                }
                available.insert(resource.clone());
            }
            ordered_nodes.push(node);
        }

        let final_composite_node = ordered_nodes
            .iter()
            .find(|node| node.kind == PostProcessEffectKind::FinalComposite)
            .map(|node| node.name.clone());

        Ok(Self {
            nodes: ordered_nodes,
            skipped_nodes,
            final_composite_node,
        })
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn skipped_node_count(&self) -> usize {
        self.skipped_nodes.len()
    }
}

fn ordered_node_indices(
    nodes: &[PostProcessPassNode],
) -> Result<Vec<usize>, PostProcessGraphValidationError> {
    let indices_by_kind = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.kind, index))
        .collect::<BTreeMap<_, _>>();
    let mut dependencies = vec![BTreeSet::<usize>::new(); nodes.len()];
    let mut dependents = vec![Vec::<usize>::new(); nodes.len()];

    for (index, node) in nodes.iter().enumerate() {
        for dependency in &node.after {
            let Some(dependency_index) = indices_by_kind.get(dependency).copied() else {
                return Err(PostProcessGraphValidationError::MissingDependency {
                    node: node.name.clone(),
                    dependency: *dependency,
                });
            };
            dependencies[index].insert(dependency_index);
            dependents[dependency_index].push(index);
        }
    }

    let mut ready = dependencies
        .iter()
        .enumerate()
        .filter_map(|(index, dependencies)| dependencies.is_empty().then_some(index))
        .collect::<VecDeque<_>>();
    let mut ordered = Vec::with_capacity(nodes.len());

    while let Some(index) = ready.pop_front() {
        ordered.push(index);
        for dependent in &dependents[index] {
            dependencies[*dependent].remove(&index);
            if dependencies[*dependent].is_empty() {
                ready.push_back(*dependent);
            }
        }
    }

    if ordered.len() != nodes.len() {
        return Err(PostProcessGraphValidationError::CycleDetected);
    }

    Ok(ordered)
}
