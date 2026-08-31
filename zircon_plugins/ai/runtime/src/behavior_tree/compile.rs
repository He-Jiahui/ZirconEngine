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
    let first_child = child_indices.len() as u32;
    child_indices.extend(compiled_children);
    let last_child = child_indices.len() as u32;
    nodes[index as usize].children = first_child..last_child;
    nodes[index as usize].subtree_end = nodes.len() as u32;
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
#[path = "compile/implementation_slots_tests.rs"]
mod implementation_slots_tests;

#[cfg(test)]
#[path = "compile/reachable_abort_tests.rs"]
mod reachable_abort_tests;

#[cfg(test)]
mod parent_index_performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::ai::{
        AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind,
        AiBehaviorTreeDescriptor,
    };

    use super::{compile_behavior_tree, CompiledBehaviorTree};

    const BENCHMARK_NODE_COUNT: usize = 4_096;
    const BENCHMARK_ABORT_PROBE_COUNT: usize = 1_024;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn compiled_parent_index_preserves_root_child_and_grandchild_relationships() {
        let descriptor = AiBehaviorTreeDescriptor::new("nested", "Nested", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                    .with_child("branch"),
            )
            .with_node(
                AiBehaviorNodeDescriptor::new("branch", AiBehaviorNodeKind::Sequence, "Branch")
                    .with_child("leaf"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "leaf",
                AiBehaviorNodeKind::Task,
                "Leaf",
            ));
        let tree = compile_behavior_tree(&descriptor).expect("valid tree");

        assert_eq!(tree.parent_index(0), None);
        assert_eq!(tree.parent_index(1), Some(0));
        assert_eq!(tree.parent_index(2), Some(1));
        assert_eq!(tree.parent_index(3), None, "out of range stays absent");
    }

    #[test]
    fn compiled_tree_owns_a_dense_parent_index() {
        let source = include_str!("compile.rs");
        let fields = source
            .split("pub struct CompiledBehaviorTree {")
            .nth(1)
            .and_then(|body| body.split("impl CompiledBehaviorTree").next())
            .expect("compiled tree fields");

        assert!(fields.contains("parent_indices: Box<[Option<u32>]>"));
        assert!(source.contains("pub(crate) fn parent_index("));
    }

    #[test]
    fn compiled_tree_caches_whether_any_node_observes_aborts() {
        let without_abort = wide_tree(4);
        assert!(!without_abort.has_abort_observers());

        let descriptor = AiBehaviorTreeDescriptor::new("abort", "Abort", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                .with_abort_policy(AiBehaviorAbortPolicy::Self_),
        );
        let with_abort = compile_behavior_tree(&descriptor).expect("valid abort tree");
        assert!(with_abort.has_abort_observers());
    }

    #[test]
    fn abort_observer_probe_reads_a_compiled_flag_without_scanning_nodes() {
        let source = include_str!("compile.rs");
        let fields = source
            .split("pub struct CompiledBehaviorTree {")
            .nth(1)
            .and_then(|body| body.split("impl CompiledBehaviorTree").next())
            .expect("compiled tree fields");
        let probe = source
            .split("pub(crate) fn has_abort_observers(")
            .nth(1)
            .and_then(|body| {
                body.split("pub(crate) fn reachable_tree_has_abort_observers")
                    .next()
            })
            .expect("has_abort_observers body");

        assert!(fields.contains("has_abort_observers: bool"));
        assert!(probe.contains("self.has_abort_observers"));
        assert!(!probe.contains("self.nodes"));
    }

    #[test]
    fn compiled_subtree_targets_preserve_node_order_and_duplicates() {
        let descriptor = AiBehaviorTreeDescriptor::new("subtrees", "Subtrees", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                    .with_child("first")
                    .with_child("second")
                    .with_child("third"),
            )
            .with_node(
                AiBehaviorNodeDescriptor::new("first", AiBehaviorNodeKind::Subtree, "First")
                    .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, "target_b"),
            )
            .with_node(
                AiBehaviorNodeDescriptor::new("second", AiBehaviorNodeKind::Subtree, "Second")
                    .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, "target_a"),
            )
            .with_node(
                AiBehaviorNodeDescriptor::new("third", AiBehaviorNodeKind::Subtree, "Third")
                    .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, "target_b"),
            );
        let tree = compile_behavior_tree(&descriptor).expect("valid subtree tree");

        assert_eq!(
            tree.subtree_targets().collect::<Vec<_>>(),
            ["target_b", "target_a", "target_b"]
        );
    }

    #[test]
    fn reachable_tree_traversal_uses_compiled_subtree_targets() {
        let source = include_str!("compile.rs");
        let fields = source
            .split("pub struct CompiledBehaviorTree {")
            .nth(1)
            .and_then(|body| body.split("impl CompiledBehaviorTree").next())
            .expect("compiled tree fields");
        let reachable = source
            .split("pub(crate) fn reachable_behavior_trees")
            .nth(1)
            .and_then(|body| body.split("#[derive(Debug)]").next())
            .expect("reachable tree traversal");

        assert!(fields.contains("subtree_targets: Box<[String]>"));
        assert!(reachable.contains("tree.subtree_targets()"));
        assert!(!reachable.contains("node.parameters()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn indexed_behavior_parent_lookup_release_benchmark_evidence() {
        let tree = wide_tree(BENCHMARK_NODE_COUNT);
        let expected_checksum = (BENCHMARK_NODE_COUNT - 1) as u64;
        assert_eq!(legacy_parent_checksum(&tree), expected_checksum);
        assert_eq!(indexed_parent_checksum(&tree), expected_checksum);

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_parent_checksum(black_box(&tree)),
            || indexed_parent_checksum(black_box(&tree)),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);
        let legacy_child_comparisons = BENCHMARK_NODE_COUNT * (BENCHMARK_NODE_COUNT - 1) / 2;
        let optimized_parent_lookups = BENCHMARK_NODE_COUNT - 1;

        println!(
            "PERF_RESULT plugins15_indexed_behavior_parent_lookup nodes={BENCHMARK_NODE_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_child_comparisons_per_sample={legacy_child_comparisons} optimized_parent_lookups_per_sample={optimized_parent_lookups} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 10 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn compiled_abort_observer_probe_release_benchmark_evidence() {
        let tree = wide_tree(BENCHMARK_NODE_COUNT);
        assert_eq!(legacy_abort_probe_checksum(&tree), 0);
        assert_eq!(compiled_abort_probe_checksum(&tree), 0);

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_abort_probe_checksum(black_box(&tree)),
            || compiled_abort_probe_checksum(black_box(&tree)),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);
        let legacy_node_visits = BENCHMARK_NODE_COUNT * BENCHMARK_ABORT_PROBE_COUNT;

        println!(
            "PERF_RESULT plugins15_compiled_abort_observer_probe nodes={BENCHMARK_NODE_COUNT} probes_per_sample={BENCHMARK_ABORT_PROBE_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_node_visits_per_sample={legacy_node_visits} optimized_flag_reads_per_sample={BENCHMARK_ABORT_PROBE_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 10 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn compiled_subtree_target_scan_release_benchmark_evidence() {
        let tree = wide_subtree_tree(BENCHMARK_NODE_COUNT);
        let expected_checksum = (BENCHMARK_NODE_COUNT - 1) as u64 * "target".len() as u64;
        assert_eq!(legacy_subtree_target_checksum(&tree), expected_checksum);
        assert_eq!(compiled_subtree_target_checksum(&tree), expected_checksum);

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_subtree_target_checksum(black_box(&tree)),
            || compiled_subtree_target_checksum(black_box(&tree)),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_compiled_subtree_target_scan nodes={BENCHMARK_NODE_COUNT} targets={} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_node_visits_per_sample={BENCHMARK_NODE_COUNT} legacy_parameter_probes_per_sample={} optimized_compiled_target_visits_per_sample={} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            BENCHMARK_NODE_COUNT - 1,
            BENCHMARK_NODE_COUNT - 1,
            BENCHMARK_NODE_COUNT - 1,
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95 * 4,
            "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
        );
    }

    fn wide_tree(node_count: usize) -> CompiledBehaviorTree {
        assert!(node_count >= 2);
        let mut root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root");
        for index in 1..node_count {
            root = root.with_child(format!("leaf_{index:04}"));
        }
        let mut descriptor = AiBehaviorTreeDescriptor::new("wide", "Wide", "root").with_node(root);
        for index in 1..node_count {
            descriptor = descriptor.with_node(AiBehaviorNodeDescriptor::new(
                format!("leaf_{index:04}"),
                AiBehaviorNodeKind::Task,
                format!("Leaf {index}"),
            ));
        }
        compile_behavior_tree(&descriptor).expect("valid wide tree")
    }

    fn wide_subtree_tree(node_count: usize) -> CompiledBehaviorTree {
        assert!(node_count >= 2);
        let mut root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root");
        for index in 1..node_count {
            root = root.with_child(format!("subtree_{index:04}"));
        }
        let mut descriptor =
            AiBehaviorTreeDescriptor::new("wide_subtrees", "Wide subtrees", "root").with_node(root);
        for index in 1..node_count {
            descriptor = descriptor.with_node(
                AiBehaviorNodeDescriptor::new(
                    format!("subtree_{index:04}"),
                    AiBehaviorNodeKind::Subtree,
                    format!("Subtree {index}"),
                )
                .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, "target"),
            );
        }
        compile_behavior_tree(&descriptor).expect("valid wide subtree tree")
    }

    fn legacy_parent_checksum(tree: &CompiledBehaviorTree) -> u64 {
        (1..tree.nodes().len() as u32)
            .map(|node_index| {
                legacy_parent_of(tree, node_index)
                    .map(|parent| u64::from(parent) + 1)
                    .unwrap_or_default()
            })
            .sum()
    }

    fn legacy_parent_of(tree: &CompiledBehaviorTree, node_index: u32) -> Option<u32> {
        tree.nodes().iter().enumerate().find_map(|(parent, node)| {
            tree.child_indices(node)
                .contains(&node_index)
                .then_some(parent as u32)
        })
    }

    fn indexed_parent_checksum(tree: &CompiledBehaviorTree) -> u64 {
        (1..tree.nodes().len() as u32)
            .map(|node_index| {
                tree.parent_index(node_index)
                    .map(|parent| u64::from(parent) + 1)
                    .unwrap_or_default()
            })
            .sum()
    }

    fn legacy_abort_probe_checksum(tree: &CompiledBehaviorTree) -> u64 {
        (0..BENCHMARK_ABORT_PROBE_COUNT)
            .map(|_| {
                black_box(tree)
                    .nodes()
                    .iter()
                    .any(|node| node.abort_policy() != AiBehaviorAbortPolicy::None)
                    as u64
            })
            .sum()
    }

    fn compiled_abort_probe_checksum(tree: &CompiledBehaviorTree) -> u64 {
        (0..BENCHMARK_ABORT_PROBE_COUNT)
            .map(|_| black_box(tree).has_abort_observers() as u64)
            .sum()
    }

    fn legacy_subtree_target_checksum(tree: &CompiledBehaviorTree) -> u64 {
        tree.nodes()
            .iter()
            .filter(|node| node.semantics() == BehaviorNodeSemantics::RunSubtree)
            .filter_map(|node| {
                node.parameters().iter().find_map(|parameter| {
                    (parameter.key == SUBTREE_TARGET_PARAMETER_KEY)
                        .then_some(&parameter.value)
                        .and_then(AiBehaviorNodeParameterValue::as_string)
                })
            })
            .map(|target| target.len() as u64)
            .sum()
    }

    fn compiled_subtree_target_checksum(tree: &CompiledBehaviorTree) -> u64 {
        tree.subtree_targets()
            .map(|target| target.len() as u64)
            .sum()
    }

    fn benchmark_paired_samples(
        mut legacy: impl FnMut() -> u64,
        mut optimized: impl FnMut() -> u64,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(operation: &mut impl FnMut() -> u64) -> u128 {
        let started = Instant::now();
        black_box(operation());
        started.elapsed().as_nanos()
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }
}
