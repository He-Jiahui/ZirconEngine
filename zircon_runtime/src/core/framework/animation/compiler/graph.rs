//! Semantic graph validation and index-based graph IR construction.

use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::animation::{AnimationGraphAsset, AnimationGraphNodeAsset};
use crate::core::math::Real;
use crate::core::resource::AssetReference;

use super::{
    parameter_kind, parameter_value_is_finite, AnimationCompileDiagnostic, AnimationCompileElement,
    AnimationCompileSeverity, AnimationCompiledParameter, AnimationCompiledParameterKind,
};

const EMPTY_NODE_ID: &str = "ZR-ANIM-COMP-GRAPH-001";
const DUPLICATE_NODE_ID: &str = "ZR-ANIM-COMP-GRAPH-002";
const MISSING_OUTPUT: &str = "ZR-ANIM-COMP-GRAPH-003";
const MULTIPLE_OUTPUTS: &str = "ZR-ANIM-COMP-GRAPH-004";
const EMPTY_OUTPUT_SOURCE: &str = "ZR-ANIM-COMP-GRAPH-005";
const UNKNOWN_NODE_REFERENCE: &str = "ZR-ANIM-COMP-GRAPH-006";
const EMPTY_BLEND_INPUTS: &str = "ZR-ANIM-COMP-GRAPH-007";
const INVALID_WEIGHT_PARAMETER: &str = "ZR-ANIM-COMP-GRAPH-008";
const CYCLE: &str = "ZR-ANIM-COMP-GRAPH-009";
const UNREACHABLE_NODE: &str = "ZR-ANIM-COMP-GRAPH-010";
const EMPTY_PARAMETER_NAME: &str = "ZR-ANIM-COMP-GRAPH-011";
const DUPLICATE_PARAMETER_NAME: &str = "ZR-ANIM-COMP-GRAPH-012";
const NON_FINITE_PARAMETER_VALUE: &str = "ZR-ANIM-COMP-GRAPH-013";
const NON_FINITE_PLAYBACK_SPEED: &str = "ZR-ANIM-COMP-GRAPH-014";

/// A graph node with all intra-graph links resolved to stable artifact indexes.
#[derive(Clone, Debug, PartialEq)]
pub enum AnimationCompiledGraphNode {
    Clip {
        clip: AssetReference,
        playback_speed: Real,
        looping: bool,
    },
    Blend {
        inputs: Vec<usize>,
        weight_parameter: Option<usize>,
    },
    Additive {
        base: usize,
        additive: usize,
        weight_parameter: Option<usize>,
    },
    Mask {
        input: usize,
        target_ids: Vec<String>,
    },
}

/// A validated, deterministic graph IR for runtime evaluation and editor preview.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationCompiledGraph {
    parameters: Vec<AnimationCompiledParameter>,
    node_ids: Vec<String>,
    nodes: Vec<AnimationCompiledGraphNode>,
    output_node: usize,
    evaluation_order: Vec<usize>,
}

impl AnimationCompiledGraph {
    pub fn parameters(&self) -> &[AnimationCompiledParameter] {
        &self.parameters
    }

    pub fn node_ids(&self) -> &[String] {
        &self.node_ids
    }

    pub fn nodes(&self) -> &[AnimationCompiledGraphNode] {
        &self.nodes
    }

    pub fn output_node(&self) -> usize {
        self.output_node
    }

    /// Dependency-first indexes for the nodes reachable from the graph output.
    pub fn evaluation_order(&self) -> &[usize] {
        &self.evaluation_order
    }
}

/// Result of compiling one graph asset without loading external resources.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationGraphCompilation {
    artifact: Option<AnimationCompiledGraph>,
    diagnostics: Vec<AnimationCompileDiagnostic>,
}

impl AnimationGraphCompilation {
    pub fn artifact(&self) -> Option<&AnimationCompiledGraph> {
        self.artifact.as_ref()
    }

    pub fn diagnostics(&self) -> &[AnimationCompileDiagnostic] {
        &self.diagnostics
    }
}

/// Validates an animation graph and resolves its string links to a stable index-based IR.
///
/// The function neither loads clip resources nor mutates the authoring asset. Callers can retain
/// a successful artifact as a last-known-good preview/runtime topology while presenting the
/// diagnostics from a later failed recompilation.
pub fn compile_animation_graph(asset: &AnimationGraphAsset) -> AnimationGraphCompilation {
    let mut diagnostics = Vec::new();
    let parameter_kinds = collect_parameters(asset, &mut diagnostics);
    let (source_nodes, node_indexes, output_sources) = collect_nodes(asset, &mut diagnostics);

    let output_index = match output_sources.as_slice() {
        [] => {
            push_error(
                &mut diagnostics,
                MISSING_OUTPUT,
                AnimationCompileElement::Asset,
                "graph must declare exactly one output node",
            );
            None
        }
        [source] => resolve_output_source(source, &node_indexes, &mut diagnostics),
        sources => {
            push_error(
                &mut diagnostics,
                MULTIPLE_OUTPUTS,
                AnimationCompileElement::GraphOutput,
                "graph must declare exactly one output node",
            );
            for source in sources {
                resolve_output_source(source, &node_indexes, &mut diagnostics);
            }
            None
        }
    };

    let dependencies = collect_dependencies(
        &source_nodes,
        &node_indexes,
        &parameter_kinds,
        &mut diagnostics,
    );
    let (topological_order, unresolved_cycle_nodes) = dependency_first_topology(&dependencies);
    for index in unresolved_cycle_nodes {
        push_error(
            &mut diagnostics,
            CYCLE,
            AnimationCompileElement::GraphNode(node_id(source_nodes[index]).to_string()),
            "graph dependencies must not contain a cycle",
        );
    }

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity() == AnimationCompileSeverity::Error)
    {
        return AnimationGraphCompilation {
            artifact: None,
            diagnostics,
        };
    }

    let output_index = output_index.expect("an error-free graph has an output index");
    let reachable = output_reachable_nodes(output_index, &dependencies);
    let evaluation_order: Vec<usize> = topological_order
        .into_iter()
        .filter(|index| reachable.contains(index))
        .collect();
    for (index, node) in source_nodes.iter().enumerate() {
        if !reachable.contains(&index) {
            diagnostics.push(AnimationCompileDiagnostic::new(
                UNREACHABLE_NODE,
                AnimationCompileSeverity::Warning,
                AnimationCompileElement::GraphNode(node_id(node).to_string()),
                "node is not reachable from the graph output",
            ));
        }
    }

    let nodes = source_nodes
        .iter()
        .map(|node| compile_node(node, &node_indexes, &parameter_kinds))
        .collect();
    let parameters = asset
        .parameters
        .iter()
        .map(|parameter| {
            AnimationCompiledParameter::with_default(
                parameter.name.clone(),
                parameter.default_value.clone(),
            )
        })
        .collect();

    AnimationGraphCompilation {
        artifact: Some(AnimationCompiledGraph {
            parameters,
            node_ids: source_nodes
                .iter()
                .map(|node| node_id(node).to_string())
                .collect(),
            nodes,
            output_node: output_index,
            evaluation_order,
        }),
        diagnostics,
    }
}

fn collect_parameters(
    asset: &AnimationGraphAsset,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> BTreeMap<String, (usize, AnimationCompiledParameterKind)> {
    let mut parameters = BTreeMap::new();
    for (index, parameter) in asset.parameters.iter().enumerate() {
        if parameter.name.is_empty() {
            push_error(
                diagnostics,
                EMPTY_PARAMETER_NAME,
                AnimationCompileElement::GraphParameter(String::new()),
                "graph parameter name must not be empty",
            );
            continue;
        }
        if parameters.contains_key(&parameter.name) {
            push_error(
                diagnostics,
                DUPLICATE_PARAMETER_NAME,
                AnimationCompileElement::GraphParameter(parameter.name.clone()),
                "graph parameter names must be unique",
            );
            continue;
        }
        if !parameter_value_is_finite(&parameter.default_value) {
            push_error(
                diagnostics,
                NON_FINITE_PARAMETER_VALUE,
                AnimationCompileElement::GraphParameter(parameter.name.clone()),
                "graph parameter defaults must be finite",
            );
        }
        parameters.insert(
            parameter.name.clone(),
            (index, parameter_kind(&parameter.default_value)),
        );
    }
    parameters
}

fn collect_nodes<'a>(
    asset: &'a AnimationGraphAsset,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> (
    Vec<&'a AnimationGraphNodeAsset>,
    BTreeMap<String, usize>,
    Vec<&'a str>,
) {
    let output_count = asset
        .nodes
        .iter()
        .filter(|node| matches!(node, AnimationGraphNodeAsset::Output { .. }))
        .count();
    let node_count = asset.nodes.len().saturating_sub(output_count);
    let mut nodes = Vec::with_capacity(node_count);
    let mut indexes = BTreeMap::new();
    let mut output_sources = Vec::with_capacity(output_count);

    for node in &asset.nodes {
        if let AnimationGraphNodeAsset::Output { source } = node {
            output_sources.push(source.as_str());
            continue;
        }
        let id = node_id(node);
        if id.is_empty() {
            push_error(
                diagnostics,
                EMPTY_NODE_ID,
                AnimationCompileElement::GraphNode(String::new()),
                "graph node id must not be empty",
            );
            continue;
        }
        if indexes.contains_key(id) {
            push_error(
                diagnostics,
                DUPLICATE_NODE_ID,
                AnimationCompileElement::GraphNode(id.to_string()),
                "graph node ids must be unique",
            );
            continue;
        }
        indexes.insert(id.to_string(), nodes.len());
        nodes.push(node);
    }
    (nodes, indexes, output_sources)
}

fn collect_dependencies(
    nodes: &[&AnimationGraphNodeAsset],
    indexes: &BTreeMap<String, usize>,
    parameter_kinds: &BTreeMap<String, (usize, AnimationCompiledParameterKind)>,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> Vec<Vec<usize>> {
    nodes
        .iter()
        .map(|node| {
            let element = AnimationCompileElement::GraphNode(node_id(node).to_string());
            match node {
                AnimationGraphNodeAsset::Clip { playback_speed, .. } => {
                    if !playback_speed.is_finite() {
                        push_error(
                            diagnostics,
                            NON_FINITE_PLAYBACK_SPEED,
                            element,
                            "clip playback speed must be finite",
                        );
                    }
                    Vec::new()
                }
                AnimationGraphNodeAsset::Blend {
                    inputs,
                    weight_parameter,
                    ..
                } => {
                    if inputs.is_empty() {
                        push_error(
                            diagnostics,
                            EMPTY_BLEND_INPUTS,
                            element.clone(),
                            "blend nodes must reference at least one input",
                        );
                    }
                    validate_weight_parameter(
                        weight_parameter.as_deref(),
                        parameter_kinds,
                        diagnostics,
                        element.clone(),
                    );
                    resolve_references(
                        inputs.iter().map(String::as_str),
                        indexes,
                        diagnostics,
                        element,
                    )
                }
                AnimationGraphNodeAsset::Additive {
                    base,
                    additive,
                    weight_parameter,
                    ..
                } => {
                    validate_weight_parameter(
                        weight_parameter.as_deref(),
                        parameter_kinds,
                        diagnostics,
                        element.clone(),
                    );
                    resolve_references(
                        [base.as_str(), additive.as_str()],
                        indexes,
                        diagnostics,
                        element,
                    )
                }
                AnimationGraphNodeAsset::Mask { input, .. } => {
                    resolve_references([input.as_str()], indexes, diagnostics, element)
                }
                AnimationGraphNodeAsset::Output { .. } => {
                    unreachable!("outputs are excluded before validation")
                }
            }
        })
        .collect()
}

fn resolve_references<'a>(
    references: impl IntoIterator<Item = &'a str>,
    indexes: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    element: AnimationCompileElement,
) -> Vec<usize> {
    references
        .into_iter()
        .filter_map(|reference| resolve_reference(reference, indexes, diagnostics, &element))
        .collect()
}

fn resolve_reference(
    reference: &str,
    indexes: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    element: &AnimationCompileElement,
) -> Option<usize> {
    if reference.is_empty() {
        push_error(
            diagnostics,
            UNKNOWN_NODE_REFERENCE,
            element.clone(),
            "graph node reference must not be empty",
        );
        return None;
    }
    match indexes.get(reference) {
        Some(index) => Some(*index),
        None => {
            push_error(
                diagnostics,
                UNKNOWN_NODE_REFERENCE,
                element.clone(),
                format!("graph node reference `{reference}` does not exist"),
            );
            None
        }
    }
}

fn resolve_output_source(
    source: &str,
    indexes: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
) -> Option<usize> {
    if source.is_empty() {
        push_error(
            diagnostics,
            EMPTY_OUTPUT_SOURCE,
            AnimationCompileElement::GraphOutput,
            "graph output source must not be empty",
        );
        return None;
    }
    resolve_reference(
        source,
        indexes,
        diagnostics,
        &AnimationCompileElement::GraphOutput,
    )
}

fn validate_weight_parameter(
    parameter: Option<&str>,
    parameters: &BTreeMap<String, (usize, AnimationCompiledParameterKind)>,
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    element: AnimationCompileElement,
) {
    let Some(parameter) = parameter else {
        return;
    };
    let valid = parameters
        .get(parameter)
        .is_some_and(|(_, kind)| *kind == AnimationCompiledParameterKind::Scalar);
    if !valid {
        push_error(
            diagnostics,
            INVALID_WEIGHT_PARAMETER,
            element,
            format!("weight parameter `{parameter}` must resolve to a scalar graph parameter"),
        );
    }
}

/// Produces source-order-stable dependency-first topology without consuming call-stack depth.
///
/// Nodes that remain after Kahn's pass have a cyclic dependency or depend on one. Reporting every
/// remaining node is deliberate: an author can repair any listed source element without relying
/// on an arbitrary single cycle witness.
fn dependency_first_topology(dependencies: &[Vec<usize>]) -> (Vec<usize>, Vec<usize>) {
    let mut unresolved_dependencies: Vec<usize> = dependencies.iter().map(Vec::len).collect();
    let mut dependents = vec![Vec::new(); dependencies.len()];
    for (node_index, node_dependencies) in dependencies.iter().enumerate() {
        for dependency in node_dependencies {
            dependents[*dependency].push(node_index);
        }
    }

    let mut ready: BTreeSet<usize> = unresolved_dependencies
        .iter()
        .enumerate()
        .filter_map(|(index, count)| (*count == 0).then_some(index))
        .collect();
    let mut topology = Vec::with_capacity(dependencies.len());
    while let Some(index) = ready.pop_first() {
        topology.push(index);
        for dependent in &dependents[index] {
            unresolved_dependencies[*dependent] -= 1;
            if unresolved_dependencies[*dependent] == 0 {
                ready.insert(*dependent);
            }
        }
    }

    let unresolved = unresolved_dependencies
        .iter()
        .enumerate()
        .filter_map(|(index, count)| (*count != 0).then_some(index))
        .collect();
    (topology, unresolved)
}

fn output_reachable_nodes(output_index: usize, dependencies: &[Vec<usize>]) -> BTreeSet<usize> {
    let mut reachable = BTreeSet::new();
    let mut pending = vec![output_index];
    while let Some(index) = pending.pop() {
        if !reachable.insert(index) {
            continue;
        }
        pending.extend(dependencies[index].iter().copied());
    }
    reachable
}

fn compile_node(
    node: &AnimationGraphNodeAsset,
    indexes: &BTreeMap<String, usize>,
    parameters: &BTreeMap<String, (usize, AnimationCompiledParameterKind)>,
) -> AnimationCompiledGraphNode {
    match node {
        AnimationGraphNodeAsset::Clip {
            clip,
            playback_speed,
            looping,
            ..
        } => AnimationCompiledGraphNode::Clip {
            clip: clip.clone(),
            playback_speed: *playback_speed,
            looping: *looping,
        },
        AnimationGraphNodeAsset::Blend {
            inputs,
            weight_parameter,
            ..
        } => AnimationCompiledGraphNode::Blend {
            inputs: inputs.iter().map(|input| indexes[input]).collect(),
            weight_parameter: compiled_parameter_slot(weight_parameter.as_deref(), parameters),
        },
        AnimationGraphNodeAsset::Additive {
            base,
            additive,
            weight_parameter,
            ..
        } => AnimationCompiledGraphNode::Additive {
            base: indexes[base],
            additive: indexes[additive],
            weight_parameter: compiled_parameter_slot(weight_parameter.as_deref(), parameters),
        },
        AnimationGraphNodeAsset::Mask {
            input, target_ids, ..
        } => AnimationCompiledGraphNode::Mask {
            input: indexes[input],
            target_ids: target_ids.clone(),
        },
        AnimationGraphNodeAsset::Output { .. } => {
            unreachable!("outputs are excluded before compilation")
        }
    }
}

fn compiled_parameter_slot(
    parameter: Option<&str>,
    parameters: &BTreeMap<String, (usize, AnimationCompiledParameterKind)>,
) -> Option<usize> {
    parameter.map(|parameter| parameters[parameter].0)
}

fn node_id(node: &AnimationGraphNodeAsset) -> &str {
    match node {
        AnimationGraphNodeAsset::Clip { id, .. }
        | AnimationGraphNodeAsset::Blend { id, .. }
        | AnimationGraphNodeAsset::Additive { id, .. }
        | AnimationGraphNodeAsset::Mask { id, .. } => id,
        AnimationGraphNodeAsset::Output { .. } => unreachable!("outputs do not have an id"),
    }
}

fn push_error(
    diagnostics: &mut Vec<AnimationCompileDiagnostic>,
    code: &'static str,
    element: AnimationCompileElement,
    message: impl Into<String>,
) {
    diagnostics.push(AnimationCompileDiagnostic::new(
        code,
        AnimationCompileSeverity::Error,
        element,
        message,
    ));
}

#[cfg(test)]
mod optimization_batch_20260830cn_runtime_tests {
    use super::*;

    const SYNTHETIC_NODE_COUNT: usize = 32_768;

    #[test]
    fn optimization_batch_20260830cn_runtime_collect_nodes_reserves_exact_partitions() {
        let asset = AnimationGraphAsset {
            name: Some("capacity-contract".to_owned()),
            parameters: Vec::new(),
            nodes: vec![
                AnimationGraphNodeAsset::Output {
                    source: "blend-a".to_owned(),
                },
                AnimationGraphNodeAsset::Blend {
                    id: "blend-a".to_owned(),
                    inputs: vec!["mask-b".to_owned()],
                    weight_parameter: None,
                },
                AnimationGraphNodeAsset::Output {
                    source: "mask-b".to_owned(),
                },
                AnimationGraphNodeAsset::Mask {
                    id: "mask-b".to_owned(),
                    input: "blend-a".to_owned(),
                    target_ids: Vec::new(),
                },
            ],
        };
        let mut diagnostics = Vec::new();

        let (nodes, indexes, outputs) = collect_nodes(&asset, &mut diagnostics);

        assert!(diagnostics.is_empty());
        assert_eq!(
            nodes.iter().map(|node| node_id(node)).collect::<Vec<_>>(),
            ["blend-a", "mask-b"]
        );
        assert_eq!(indexes.get("blend-a"), Some(&0));
        assert_eq!(indexes.get("mask-b"), Some(&1));
        assert_eq!(outputs, ["blend-a", "mask-b"]);
        assert_eq!(nodes.capacity(), nodes.len());
        assert_eq!(outputs.capacity(), outputs.len());
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cn_runtime_collect_nodes_capacity_evidence() {
        let output_flags = (0..SYNTHETIC_NODE_COUNT)
            .map(|index| index % 8 == 0)
            .collect::<Vec<_>>();
        let legacy_growth_events = collect_partition_growth_events(&output_flags, false);
        let optimized_growth_events = collect_partition_growth_events(&output_flags, true);

        println!(
            "RUNTIME501_ANIMATION_GRAPH_NODE_CAPACITY_BENCH_V1 nodes={SYNTHETIC_NODE_COUNT} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
        );
        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
    }

    fn collect_partition_growth_events(output_flags: &[bool], reserve_exact: bool) -> usize {
        let output_count = reserve_exact
            .then(|| output_flags.iter().filter(|is_output| **is_output).count())
            .unwrap_or_default();
        let node_count = reserve_exact
            .then(|| output_flags.len().saturating_sub(output_count))
            .unwrap_or_default();
        let mut nodes = Vec::with_capacity(node_count);
        let mut outputs = Vec::with_capacity(output_count);
        let mut growth_events = 0;
        for (index, is_output) in output_flags.iter().copied().enumerate() {
            let target = if is_output { &mut outputs } else { &mut nodes };
            let capacity = target.capacity();
            target.push(index);
            growth_events += usize::from(target.capacity() != capacity);
        }
        std::hint::black_box((nodes, outputs));
        growth_events
    }
}

#[cfg(test)]
mod optimization_batch_hc_runtime584_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_hc_runtime584_valid_references_preserve_indexes_without_diagnostics() {
        let indexes = BTreeMap::from([("target".to_owned(), 7)]);
        let mut diagnostics = Vec::new();
        let resolved = resolve_references(
            ["target", "target"],
            &indexes,
            &mut diagnostics,
            AnimationCompileElement::GraphNode("blend".to_owned()),
        );

        assert_eq!(resolved, [7, 7]);
        assert!(diagnostics.is_empty());
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_hc_runtime584_animation_reference_borrow_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const REFERENCES: usize = 65_536;
        let indexes = BTreeMap::from([("target".to_owned(), 7)]);
        let element = AnimationCompileElement::GraphNode("animation-node/".repeat(128));
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &indexes, &element, REFERENCES));
                optimized.push(measure(true, &indexes, &element, REFERENCES));
            } else {
                optimized.push(measure(true, &indexes, &element, REFERENCES));
                legacy.push(measure(false, &indexes, &element, REFERENCES));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME584_ANIMATION_REFERENCE_BORROW_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
references={REFERENCES} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "borrowed valid-reference diagnostics must improve P95 by at least 50%"
        );
    }

    fn measure(
        optimized: bool,
        indexes: &BTreeMap<String, usize>,
        element: &AnimationCompileElement,
        references: usize,
    ) -> u128 {
        let started = Instant::now();
        let mut diagnostics = Vec::new();
        let mut resolved = 0_usize;
        for _ in 0..references {
            let index = if optimized {
                resolve_reference("target", indexes, &mut diagnostics, black_box(element))
            } else {
                resolve_reference_legacy("target", indexes, &mut diagnostics, black_box(element))
            };
            resolved ^= index.expect("fixture reference should resolve");
        }
        black_box((resolved, diagnostics));
        started.elapsed().as_nanos().max(1)
    }

    fn resolve_reference_legacy(
        reference: &str,
        indexes: &BTreeMap<String, usize>,
        diagnostics: &mut Vec<AnimationCompileDiagnostic>,
        element: &AnimationCompileElement,
    ) -> Option<usize> {
        let owned_element = element.clone();
        match indexes.get(reference) {
            Some(index) => Some(*index),
            None => {
                push_error(
                    diagnostics,
                    UNKNOWN_NODE_REFERENCE,
                    owned_element,
                    format!("graph node reference `{reference}` does not exist"),
                );
                None
            }
        }
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
