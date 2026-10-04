use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Range;

use zircon_runtime::core::framework::ai::{
    AiBehaviorAbortPolicy, AiBehaviorNodeKind, AiBehaviorNodeParameter,
    AiBehaviorNodeParameterValue, AiBehaviorTreeDescriptor,
};

use super::{
    standard_node_catalog, BehaviorNodeCatalogError, BehaviorNodeCategory, BehaviorNodeFactory,
    BehaviorNodeSemantics, BehaviorNodeSlot, FrozenBehaviorNodeCatalog, SelectorRecheckPolicy,
    SUBTREE_TARGET_PARAMETER_KEY,
};

use super::random_selector_weights::CompiledRandomSelectorWeights;

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledBehaviorNode {
    id: String,
    kind: AiBehaviorNodeKind,
    implementation: BehaviorNodeSlot,
    semantics: BehaviorNodeSemantics,
    factory: Option<BehaviorNodeFactory>,
    selector_recheck: SelectorRecheckPolicy,
    parameters: Box<[AiBehaviorNodeParameter]>,
    children: Range<u32>,
    abort_policy: AiBehaviorAbortPolicy,
    subtree_end: u32,
    random_selector_weights: Option<CompiledRandomSelectorWeights>,
}

impl CompiledBehaviorNode {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn kind(&self) -> AiBehaviorNodeKind {
        self.kind
    }

    pub fn parameters(&self) -> &[AiBehaviorNodeParameter] {
        &self.parameters
    }

    pub fn implementation(&self) -> BehaviorNodeSlot {
        self.implementation
    }

    pub fn semantics(&self) -> BehaviorNodeSemantics {
        self.semantics
    }

    pub fn factory(&self) -> Option<BehaviorNodeFactory> {
        self.factory
    }

    pub fn selector_recheck_policy(&self) -> SelectorRecheckPolicy {
        self.selector_recheck
    }

    pub fn children(&self) -> Range<u32> {
        self.children.clone()
    }

    pub fn abort_policy(&self) -> AiBehaviorAbortPolicy {
        self.abort_policy
    }

    pub(super) fn random_selector_weights(&self) -> Option<&CompiledRandomSelectorWeights> {
        self.random_selector_weights.as_ref()
    }

    pub(crate) fn subtree_range(&self, node_index: u32) -> Range<u32> {
        node_index..self.subtree_end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledBehaviorTree {
    id: String,
    nodes: Box<[CompiledBehaviorNode]>,
    child_indices: Box<[u32]>,
    parent_indices: Box<[Option<u32>]>,
    has_abort_observers: bool,
    subtree_targets: Box<[String]>,
    implementation_slots: Box<[BehaviorNodeSlot]>,
}

impl CompiledBehaviorTree {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn root(&self) -> &CompiledBehaviorNode {
        &self.nodes[0]
    }

    pub fn node(&self, index: usize) -> &CompiledBehaviorNode {
        &self.nodes[index]
    }

    pub fn nodes(&self) -> &[CompiledBehaviorNode] {
        &self.nodes
    }

    pub fn node_ids(&self) -> Vec<&str> {
        self.nodes.iter().map(CompiledBehaviorNode::id).collect()
    }

    pub fn child_indices(&self, node: &CompiledBehaviorNode) -> &[u32] {
        &self.child_indices[node.children.start as usize..node.children.end as usize]
    }

    pub(crate) fn parent_index(&self, node_index: u32) -> Option<u32> {
        self.parent_indices
            .get(node_index as usize)
            .copied()
            .flatten()
    }

    pub(crate) fn uses_any_implementation(&self, slots: &[BehaviorNodeSlot]) -> bool {
        self.implementation_slots
            .iter()
            .any(|implementation| slots.contains(implementation))
    }

    pub(crate) fn implementation_slots(&self) -> impl Iterator<Item = BehaviorNodeSlot> + '_ {
        self.implementation_slots.iter().copied()
    }

    pub(crate) fn has_abort_observers(&self) -> bool {
        self.has_abort_observers
    }

    pub(crate) fn subtree_targets(&self) -> impl Iterator<Item = &str> {
        self.subtree_targets.iter().map(String::as_str)
    }

    pub(crate) fn reachable_tree_has_abort_observers(
        &self,
        registered_trees: &[CompiledBehaviorTree],
    ) -> bool {
        let mut pending = vec![self];
        let mut visited = HashSet::new();
        while let Some(tree) = pending.pop() {
            if !visited.insert(tree.id()) {
                continue;
            }
            if tree.has_abort_observers() {
                return true;
            }
            for target in tree.subtree_targets() {
                if let Some(target_tree) = registered_trees.iter().find(|tree| tree.id() == target)
                {
                    pending.push(target_tree);
                }
            }
        }
        false
    }
}

pub(crate) fn reachable_behavior_trees<'a>(
    root: &'a CompiledBehaviorTree,
    registered_trees: &'a [CompiledBehaviorTree],
) -> Vec<&'a CompiledBehaviorTree> {
    let mut reachable = Vec::new();
    let mut pending = vec![root];
    let mut visited = HashSet::new();
    while let Some(tree) = pending.pop() {
        if !visited.insert(tree.id()) {
            continue;
        }
        reachable.push(tree);
        for target in tree.subtree_targets() {
            if let Some(target_tree) = registered_trees.iter().find(|tree| tree.id() == target) {
                pending.push(target_tree);
            }
        }
    }
    reachable
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BehaviorTreeCompileError {
    EmptyTreeId,
    MissingRoot {
        node_id: String,
    },
    DuplicateNodeId {
        node_id: String,
    },
    MissingChild {
        node_id: String,
        child_id: String,
    },
    MultipleParents {
        node_id: String,
    },
    Cycle {
        node_id: String,
    },
    UnreachableNode {
        node_id: String,
    },
    UnknownImplementation {
        node_id: String,
        implementation: String,
    },
    MissingCatalogDescriptor {
        node_id: String,
        implementation: String,
    },
    ImplementationCategoryMismatch {
        node_id: String,
        implementation: String,
        expected: BehaviorNodeCategory,
        actual: BehaviorNodeCategory,
    },
    StandardCatalog(BehaviorNodeCatalogError),
}

#[derive(Debug)]
pub enum BehaviorTreeAssetError {
    Parse(toml::de::Error),
    Validation(zircon_runtime::core::framework::ai::AiManagerError),
    Compile(BehaviorTreeCompileError),
}

impl fmt::Display for BehaviorTreeAssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(formatter, "behavior-tree TOML is invalid: {error}"),
            Self::Validation(error) => {
                write!(formatter, "behavior-tree descriptor is invalid: {error}")
            }
            Self::Compile(error) => write!(formatter, "behavior-tree compilation failed: {error}"),
        }
    }
}

impl std::error::Error for BehaviorTreeAssetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Validation(error) => Some(error),
            Self::Compile(error) => Some(error),
        }
    }
}

impl fmt::Display for BehaviorTreeCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTreeId => formatter.write_str("tree id is empty"),
            Self::MissingRoot { node_id } => write!(formatter, "root node `{node_id}` is missing"),
            Self::DuplicateNodeId { node_id } => {
                write!(formatter, "node id `{node_id}` is duplicated")
            }
            Self::MissingChild { node_id, child_id } => write!(
                formatter,
                "node `{node_id}` references missing child `{child_id}`"
            ),
            Self::MultipleParents { node_id } => {
                write!(formatter, "node `{node_id}` has multiple parents")
            }
            Self::Cycle { node_id } => {
                write!(formatter, "node `{node_id}` participates in a cycle")
            }
            Self::UnreachableNode { node_id } => {
                write!(formatter, "node `{node_id}` is unreachable from the root")
            }
            Self::UnknownImplementation {
                node_id,
                implementation,
            } => write!(
                formatter,
                "node `{node_id}` references unknown implementation `{implementation}`"
            ),
            Self::MissingCatalogDescriptor {
                node_id,
                implementation,
            } => write!(
                formatter,
                "node `{node_id}` resolved implementation `{implementation}` without a catalog descriptor"
            ),
            Self::ImplementationCategoryMismatch {
                node_id,
                implementation,
                expected,
                actual,
            } => write!(
                formatter,
                "node `{node_id}` implementation `{implementation}` has category {actual:?}, expected {expected:?} for its DTO kind"
            ),
            Self::StandardCatalog(error) => {
                write!(formatter, "standard node catalog is invalid: {error}")
            }
        }
    }
}

impl std::error::Error for BehaviorTreeCompileError {}

pub fn compile_behavior_tree(
    descriptor: &AiBehaviorTreeDescriptor,
) -> Result<CompiledBehaviorTree, BehaviorTreeCompileError> {
    let catalog = standard_node_catalog().map_err(BehaviorTreeCompileError::StandardCatalog)?;
    compile_behavior_tree_with_catalog(descriptor, &catalog)
}

pub fn compile_behavior_tree_toml(
    source: &str,
) -> Result<CompiledBehaviorTree, BehaviorTreeAssetError> {
    let descriptor = toml::from_str::<AiBehaviorTreeDescriptor>(source)
        .map_err(BehaviorTreeAssetError::Parse)?;
    crate::manager::validation::validate_behavior_tree_descriptor_for_compile(&descriptor)
        .map_err(BehaviorTreeAssetError::Validation)?;
    compile_behavior_tree(&descriptor).map_err(BehaviorTreeAssetError::Compile)
}

pub fn compile_behavior_tree_with_catalog(
    descriptor: &AiBehaviorTreeDescriptor,
    catalog: &FrozenBehaviorNodeCatalog,
) -> Result<CompiledBehaviorTree, BehaviorTreeCompileError> {
    if descriptor.id.trim().is_empty() {
        return Err(BehaviorTreeCompileError::EmptyTreeId);
    }
    let mut descriptors = HashMap::with_capacity(descriptor.nodes.len());
    for node in &descriptor.nodes {
        if descriptors.insert(node.id.as_str(), node).is_some() {
            return Err(BehaviorTreeCompileError::DuplicateNodeId {
                node_id: node.id.clone(),
            });
        }
    }
    if !descriptors.contains_key(descriptor.root_node.as_str()) {
        return Err(BehaviorTreeCompileError::MissingRoot {
            node_id: descriptor.root_node.clone(),
        });
    }

    let mut parent_counts = descriptors
        .keys()
        .copied()
        .map(|id| (id, 0_usize))
        .collect::<HashMap<_, _>>();
    for node in descriptors.values() {
        for child in &node.children {
            let Some(count) = parent_counts.get_mut(child.as_str()) else {
                return Err(BehaviorTreeCompileError::MissingChild {
                    node_id: node.id.clone(),
                    child_id: child.clone(),
                });
            };
            *count += 1;
            if *count > 1 {
                return Err(BehaviorTreeCompileError::MultipleParents {
                    node_id: child.clone(),
                });
            }
        }
    }

    let mut nodes = Vec::with_capacity(descriptor.nodes.len());
    let mut child_indices = Vec::with_capacity(descriptor.nodes.len().saturating_sub(1));
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    compile_subtree(
        descriptor.root_node.as_str(),
        &descriptors,
        &mut visiting,
        &mut visited,
        &mut nodes,
        &mut child_indices,
        catalog,
    )?;
    if let Some(node) = descriptor
        .nodes
        .iter()
        .find(|node| !visited.contains(node.id.as_str()))
    {
        return Err(BehaviorTreeCompileError::UnreachableNode {
            node_id: node.id.clone(),
        });
    }

    let mut parent_indices = vec![None; nodes.len()];
    for (parent_index, node) in nodes.iter().enumerate() {
        for child_index in &child_indices[node.children.start as usize..node.children.end as usize]
        {
            parent_indices[*child_index as usize] = Some(parent_index as u32);
        }
    }
    let has_abort_observers = nodes
        .iter()
        .any(|node| node.abort_policy != AiBehaviorAbortPolicy::None);
    let subtree_targets = nodes
        .iter()
        .filter(|node| node.semantics == BehaviorNodeSemantics::RunSubtree)
        .filter_map(|node| {
            node.parameters.iter().find_map(|parameter| {
                (parameter.key == SUBTREE_TARGET_PARAMETER_KEY)
                    .then_some(&parameter.value)
                    .and_then(AiBehaviorNodeParameterValue::as_string)
            })
        })
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let mut implementation_slots = Vec::new();
    for node in &nodes {
        if !implementation_slots.contains(&node.implementation) {
            implementation_slots.push(node.implementation);
        }
    }

    Ok(CompiledBehaviorTree {
        id: descriptor.id.clone(),
        nodes: nodes.into_boxed_slice(),
        child_indices: child_indices.into_boxed_slice(),
        parent_indices: parent_indices.into_boxed_slice(),
        has_abort_observers,
        subtree_targets,
        implementation_slots: implementation_slots.into_boxed_slice(),
    })
}

fn compile_subtree<'a>(
    node_id: &'a str,
    descriptors: &HashMap<
        &'a str,
        &'a zircon_runtime::core::framework::ai::AiBehaviorNodeDescriptor,
    >,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
    nodes: &mut Vec<CompiledBehaviorNode>,
    child_indices: &mut Vec<u32>,
    catalog: &FrozenBehaviorNodeCatalog,
) -> Result<u32, BehaviorTreeCompileError> {
    if !visiting.insert(node_id) {
        return Err(BehaviorTreeCompileError::Cycle {
            node_id: node_id.to_string(),
        });
    }
    let descriptor = descriptors[node_id];
    let implementation = catalog
        .resolve(descriptor.implementation.as_str())
        .ok_or_else(|| BehaviorTreeCompileError::UnknownImplementation {
            node_id: descriptor.id.clone(),
            implementation: descriptor.implementation.clone(),
        })?;
    let catalog_descriptor = catalog.get(implementation).ok_or_else(|| {
        BehaviorTreeCompileError::MissingCatalogDescriptor {
            node_id: descriptor.id.clone(),
            implementation: descriptor.implementation.clone(),
        }
    })?;
    let expected_category = category_for_kind(descriptor.kind);
    if catalog_descriptor.category() != expected_category {
        return Err(BehaviorTreeCompileError::ImplementationCategoryMismatch {
            node_id: descriptor.id.clone(),
            implementation: descriptor.implementation.clone(),
            expected: expected_category,
            actual: catalog_descriptor.category(),
        });
    }
    let index = nodes.len() as u32;
    nodes.push(CompiledBehaviorNode {
        id: descriptor.id.clone(),
        kind: descriptor.kind,
        implementation,
        semantics: catalog_descriptor.semantics(),
        factory: catalog_descriptor.factory(),
        selector_recheck: catalog_descriptor.selector_recheck_policy(),
        parameters: descriptor.parameters.clone().into_boxed_slice(),
        children: 0..0,
        abort_policy: descriptor.abort_policy,
        subtree_end: index + 1,
        random_selector_weights: None,
    });
    let mut compiled_children = Vec::with_capacity(descriptor.children.len());
    for child in &descriptor.children {
        compiled_children.push(compile_subtree(
            child.as_str(),
            descriptors,
            visiting,
            visited,
            nodes,
            child_indices,
            catalog,
        )?);
    }
    let random_selector_weights =
        (catalog_descriptor.semantics() == BehaviorNodeSemantics::RandomSelector).then(|| {
            CompiledRandomSelectorWeights::compile(
                &nodes[index as usize].parameters,
                compiled_children
                    .iter()
                    .map(|child_index| nodes[*child_index as usize].id.as_str()),
            )
        });
    let first_child = child_indices.len() as u32;
    child_indices.extend(compiled_children);
    let last_child = child_indices.len() as u32;
    nodes[index as usize].children = first_child..last_child;
    nodes[index as usize].subtree_end = nodes.len() as u32;
    nodes[index as usize].random_selector_weights = random_selector_weights;
    visiting.remove(node_id);
    visited.insert(node_id);
    Ok(index)
}

fn category_for_kind(kind: AiBehaviorNodeKind) -> BehaviorNodeCategory {
    match kind {
        AiBehaviorNodeKind::Selector
        | AiBehaviorNodeKind::Sequence
        | AiBehaviorNodeKind::Parallel => BehaviorNodeCategory::Composite,
        AiBehaviorNodeKind::Decorator => BehaviorNodeCategory::Decorator,
        AiBehaviorNodeKind::Service => BehaviorNodeCategory::Service,
        AiBehaviorNodeKind::Task | AiBehaviorNodeKind::Subtree => BehaviorNodeCategory::Task,
    }
}

#[cfg(test)]
#[path = "compile/tests/implementation_slots_tests.rs"]
mod implementation_slots_tests;

#[cfg(test)]
#[path = "compile/tests/reachable_abort_tests.rs"]
mod reachable_abort_tests;

#[cfg(test)]
#[path = "compile/tests/random_selector_weights_tests.rs"]
mod random_selector_weights_tests;

#[cfg(test)]
#[path = "tests/compile_parent_index_performance_tests.rs"]
mod parent_index_performance_tests;
