use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::{
    PostProcessEffectSettings, RenderPipelinePhase, RenderResolutionPolicy, RenderUpscalerKind,
    RenderViewFamilyPipeline,
};
use crate::core::math::UVec2;

use super::{
    ordered_node_indices, required_post_process_phases, PostProcessEffectKind,
    PostProcessGraphValidationError, PostProcessPassGraph, PostProcessPassNode,
    PostProcessStackDescriptor,
};

const PHASE_SORT_NODE_COUNT: usize = 1_024;
const VIEW_PHASE_NODE_COUNT: usize = 16_384;
const SAMPLE_PAIRS: usize = 15;

#[test]
fn graph_orders_independent_nodes_by_view_family_phase() {
    let nodes = vec![
        PostProcessPassNode::new("output", PostProcessEffectKind::OutputTransfer),
        PostProcessPassNode::new("taa", PostProcessEffectKind::TaaResolve),
        PostProcessPassNode::new("bloom", PostProcessEffectKind::Bloom),
        PostProcessPassNode::new("fxaa", PostProcessEffectKind::Fxaa),
        PostProcessPassNode::new("uber", PostProcessEffectKind::Uber),
        PostProcessPassNode::new("primary-upscale", PostProcessEffectKind::PrimaryUpscale),
    ];

    assert_eq!(
        ordered_node_indices(&nodes).expect("independent phases form a valid graph"),
        vec![1, 2, 4, 3, 5, 0]
    );
}

#[test]
fn graph_rejects_temporal_nodes_when_the_view_family_is_spatial() {
    let stack = PostProcessStackDescriptor {
        initial_resources: Vec::new(),
        effects: vec![PostProcessEffectSettings::new(
            PostProcessEffectKind::TaaResolve,
        )],
    };
    let spatial_pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::default(),
        RenderUpscalerKind::Spatial,
    );

    assert_eq!(
        PostProcessPassGraph::validate_stack_for_view_family(&stack, &spatial_pipeline),
        Err(
            PostProcessGraphValidationError::UnavailableViewFamilyPhase {
                node: "taa-resolve".to_string(),
                phase: RenderPipelinePhase::TemporalReconstruction,
            }
        )
    );
}

#[test]
fn graph_rejects_primary_upscale_when_temporal_reconstruction_owns_the_transition() {
    let stack = PostProcessStackDescriptor {
        initial_resources: Vec::new(),
        effects: vec![PostProcessEffectSettings::new(
            PostProcessEffectKind::PrimaryUpscale,
        )],
    };
    let temporal_pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.5, 1.0),
        RenderUpscalerKind::Temporal,
    );

    assert_eq!(
        PostProcessPassGraph::validate_stack_for_view_family(&stack, &temporal_pipeline),
        Err(
            PostProcessGraphValidationError::UnavailableViewFamilyPhase {
                node: "primary-upscale".to_string(),
                phase: RenderPipelinePhase::PrimarySpatialUpscale,
            }
        )
    );
}

#[test]
fn graph_requires_primary_upscale_when_the_resolved_view_family_needs_it() {
    let stack = PostProcessStackDescriptor {
        initial_resources: Vec::new(),
        effects: Vec::new(),
    };
    let spatial_pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.5, 1.0),
        RenderUpscalerKind::Spatial,
    );

    assert_eq!(
        PostProcessPassGraph::validate_stack_for_view_family(&stack, &spatial_pipeline),
        Err(
            PostProcessGraphValidationError::MissingRequiredViewFamilyPhase {
                phase: RenderPipelinePhase::PrimarySpatialUpscale,
            }
        )
    );
}

#[test]
fn graph_requires_secondary_upscale_when_the_resolved_view_family_needs_it() {
    let stack = PostProcessStackDescriptor {
        initial_resources: Vec::new(),
        effects: vec![PostProcessEffectSettings::new(
            PostProcessEffectKind::TaaResolve,
        )],
    };
    let temporal_pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.5, 0.5),
        RenderUpscalerKind::Temporal,
    );

    assert_eq!(
        PostProcessPassGraph::validate_stack_for_view_family(&stack, &temporal_pipeline),
        Err(
            PostProcessGraphValidationError::MissingRequiredViewFamilyPhase {
                phase: RenderPipelinePhase::SecondarySpatialUpscale,
            }
        )
    );
}

#[test]
fn graph_requires_temporal_reconstruction_when_the_resolved_view_family_needs_it() {
    let stack = PostProcessStackDescriptor {
        initial_resources: Vec::new(),
        effects: Vec::new(),
    };
    let temporal_pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.5, 1.0),
        RenderUpscalerKind::Temporal,
    );

    assert_eq!(
        PostProcessPassGraph::validate_stack_for_view_family(&stack, &temporal_pipeline),
        Err(
            PostProcessGraphValidationError::MissingRequiredViewFamilyPhase {
                phase: RenderPipelinePhase::TemporalReconstruction,
            }
        )
    );
}

#[test]
fn optimization_wave_20260825vw_runtime102_phase_buckets_preserve_legacy_order() {
    let nodes = vec![
        PostProcessPassNode::new("output", PostProcessEffectKind::OutputTransfer),
        PostProcessPassNode::new("taa", PostProcessEffectKind::TaaResolve),
        PostProcessPassNode::new("bloom", PostProcessEffectKind::Bloom),
        PostProcessPassNode::new("fxaa", PostProcessEffectKind::Fxaa),
        PostProcessPassNode::new("uber", PostProcessEffectKind::Uber),
        PostProcessPassNode::new("primary-upscale", PostProcessEffectKind::PrimaryUpscale),
    ];

    assert_eq!(
        ordered_node_indices(&nodes),
        legacy_ordered_node_indices(&nodes)
    );

    let same_phase_dependencies = vec![
        PostProcessPassNode::new("motion-blur", PostProcessEffectKind::MotionBlur)
            .with_after([PostProcessEffectKind::DepthOfField]),
        PostProcessPassNode::new("bloom", PostProcessEffectKind::Bloom)
            .with_after([PostProcessEffectKind::MotionBlur]),
        PostProcessPassNode::new("depth-of-field", PostProcessEffectKind::DepthOfField),
        PostProcessPassNode::new("output", PostProcessEffectKind::OutputTransfer)
            .with_after([PostProcessEffectKind::Bloom]),
    ];
    assert_eq!(
        ordered_node_indices(&same_phase_dependencies),
        legacy_ordered_node_indices(&same_phase_dependencies)
    );
}

#[test]
fn optimization_wave_20260825vw_runtime102_phase_buckets_preserve_error_precedence() {
    let nodes = vec![
        PostProcessPassNode::new("taa", PostProcessEffectKind::TaaResolve)
            .with_after([PostProcessEffectKind::Bloom]),
        PostProcessPassNode::new("output", PostProcessEffectKind::OutputTransfer)
            .with_after([PostProcessEffectKind::Blur]),
        PostProcessPassNode::new("bloom", PostProcessEffectKind::Bloom),
    ];

    assert_eq!(
        ordered_node_indices(&nodes),
        Err(PostProcessGraphValidationError::MissingDependency {
            node: "output".to_string(),
            dependency: PostProcessEffectKind::Blur,
        })
    );

    let reverse_phase_dependency = vec![
        PostProcessPassNode::new("taa", PostProcessEffectKind::TaaResolve)
            .with_after([PostProcessEffectKind::Bloom]),
        PostProcessPassNode::new("bloom", PostProcessEffectKind::Bloom),
    ];
    assert_eq!(
        ordered_node_indices(&reverse_phase_dependency),
        Err(PostProcessGraphValidationError::CycleDetected)
    );
}

#[test]
fn optimization_wave_20260825vw_runtime102_phase_buckets_use_linear_phase_admission() {
    let source = include_str!("../pass_graph.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    assert!(production.contains("phase_buckets"));
    assert!(production.contains("dependency_phase == node_phase"));
    assert!(!production.contains("for (later_index, later_node)"));
    assert!(!production.contains("for (earlier_index, earlier_node)"));
}

#[test]
fn optimization_wave_20260825vw_runtime102_view_phase_validation_is_single_pass() {
    let source = include_str!("../pass_graph.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes tests");
    assert!(production.contains("enabled_phase_mask"));
    assert!(production.contains("observed_phase_mask"));
    assert!(!production.contains(".any(|node| node.chain_slot.pipeline_phase() == phase)"));
}

#[test]
#[ignore = "release-only deterministic performance evidence"]
fn optimization_wave_20260825vw_runtime102_phase_bucket_p95_evidence() {
    const TARGET_P95_PERCENT: u128 = 10;
    const TARGET_P95_NS: u128 = 5_000_000;
    let phase_kinds = [
        PostProcessEffectKind::TaaResolve,
        PostProcessEffectKind::Bloom,
        PostProcessEffectKind::Uber,
        PostProcessEffectKind::Fxaa,
        PostProcessEffectKind::PrimaryUpscale,
        PostProcessEffectKind::OutputTransfer,
    ];
    let nodes = (0..PHASE_SORT_NODE_COUNT)
        .map(|index| {
            PostProcessPassNode::new(
                format!("phase-node-{index}"),
                phase_kinds[index % phase_kinds.len()],
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        ordered_node_indices(&nodes),
        legacy_ordered_node_indices(&nodes)
    );
    black_box(legacy_ordered_node_indices(black_box(&nodes)).unwrap());
    black_box(ordered_node_indices(black_box(&nodes)).unwrap());

    let (legacy_samples_ns, optimized_samples_ns) = alternating_samples(
        || black_box(legacy_ordered_node_indices(black_box(&nodes)).unwrap()),
        || black_box(ordered_node_indices(black_box(&nodes)).unwrap()),
    );
    let legacy_p50_ns = nearest_rank(&legacy_samples_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_samples_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_samples_ns, 95);
    let legacy_phase_comparisons = PHASE_SORT_NODE_COUNT * PHASE_SORT_NODE_COUNT;

    println!(
        "RUNTIME102_POST_PROCESS_PHASE_BUCKET_BENCH_V1 node_count={PHASE_SORT_NODE_COUNT} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even percentile_method=nearest_rank legacy_phase_comparisons={legacy_phase_comparisons} optimized_phase_admissions={PHASE_SORT_NODE_COUNT} target_p95_percent={TARGET_P95_PERCENT} target_p95_ns={TARGET_P95_NS} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        format_samples(&legacy_samples_ns),
        format_samples(&optimized_samples_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "phase buckets must reduce P95 to at most {TARGET_P95_PERCENT}% of legacy: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    assert!(
        optimized_p95_ns <= TARGET_P95_NS,
        "phase bucket P95 must stay within {TARGET_P95_NS}ns: {optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "release-only deterministic performance evidence"]
fn optimization_wave_20260825vw_runtime102_view_phase_single_pass_p95_evidence() {
    const TARGET_P95_PERCENT: u128 = 75;
    const TARGET_P95_NS: u128 = 5_000_000;
    let mut nodes = (0..VIEW_PHASE_NODE_COUNT)
        .map(|index| {
            PostProcessPassNode::new(format!("bloom-{index}"), PostProcessEffectKind::Bloom)
        })
        .collect::<Vec<_>>();
    nodes.push(PostProcessPassNode::new(
        "primary-upscale",
        PostProcessEffectKind::PrimaryUpscale,
    ));
    let graph = PostProcessPassGraph::from_ordered_nodes(nodes, Vec::new(), None);
    let pipeline = RenderViewFamilyPipeline::resolve(
        UVec2::new(1920, 1080),
        RenderResolutionPolicy::with_scales(0.5, 1.0),
        RenderUpscalerKind::Spatial,
    );

    assert_eq!(
        graph.validate_view_family_phases(&pipeline),
        legacy_validate_view_family_phases(&graph, &pipeline)
    );
    black_box(legacy_validate_view_family_phases(
        black_box(&graph),
        black_box(&pipeline),
    ))
    .unwrap();
    black_box(black_box(&graph).validate_view_family_phases(black_box(&pipeline))).unwrap();

    let (legacy_samples_ns, optimized_samples_ns) = alternating_samples(
        || {
            black_box(legacy_validate_view_family_phases(
                black_box(&graph),
                black_box(&pipeline),
            ))
            .unwrap()
        },
        || black_box(black_box(&graph).validate_view_family_phases(black_box(&pipeline))).unwrap(),
    );
    let legacy_p50_ns = nearest_rank(&legacy_samples_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_samples_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_samples_ns, 95);
    let node_count = graph.node_count();

    println!(
        "RUNTIME102_POST_PROCESS_VIEW_PHASE_SINGLE_PASS_BENCH_V1 node_count={node_count} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even percentile_method=nearest_rank legacy_node_passes=2 optimized_node_passes=1 target_p95_percent={TARGET_P95_PERCENT} target_p95_ns={TARGET_P95_NS} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        format_samples(&legacy_samples_ns),
        format_samples(&optimized_samples_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "single-pass phase validation must reduce P95 to at most {TARGET_P95_PERCENT}% of legacy: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    assert!(
        optimized_p95_ns <= TARGET_P95_NS,
        "single-pass phase validation P95 must stay within {TARGET_P95_NS}ns: {optimized_p95_ns}ns"
    );
}

fn legacy_ordered_node_indices(
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

    for (later_index, later_node) in nodes.iter().enumerate() {
        let later_phase = later_node.chain_slot.pipeline_phase().order();
        for (earlier_index, earlier_node) in nodes.iter().enumerate() {
            if earlier_node.chain_slot.pipeline_phase().order() < later_phase
                && dependencies[later_index].insert(earlier_index)
            {
                dependents[earlier_index].push(later_index);
            }
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

fn legacy_validate_view_family_phases(
    graph: &PostProcessPassGraph,
    pipeline: &RenderViewFamilyPipeline,
) -> Result<(), PostProcessGraphValidationError> {
    for node in &graph.nodes {
        let phase = node.chain_slot.pipeline_phase();
        if !pipeline.phases().contains(&phase) {
            return Err(
                PostProcessGraphValidationError::UnavailableViewFamilyPhase {
                    node: node.name.clone(),
                    phase,
                },
            );
        }
    }
    for phase in required_post_process_phases(pipeline) {
        if !graph
            .nodes
            .iter()
            .any(|node| node.chain_slot.pipeline_phase() == phase)
        {
            return Err(PostProcessGraphValidationError::MissingRequiredViewFamilyPhase { phase });
        }
    }
    Ok(())
}

fn alternating_samples<Legacy, Optimized, LegacyOutput, OptimizedOutput>(
    mut legacy: Legacy,
    mut optimized: Optimized,
) -> (Vec<u128>, Vec<u128>)
where
    Legacy: FnMut() -> LegacyOutput,
    Optimized: FnMut() -> OptimizedOutput,
{
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_ns(&mut legacy));
            optimized_samples.push(measure_ns(&mut optimized));
        } else {
            optimized_samples.push(measure_ns(&mut optimized));
            legacy_samples.push(measure_ns(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_ns<Action, Output>(action: &mut Action) -> u128
where
    Action: FnMut() -> Output,
{
    let started = Instant::now();
    black_box(action());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (percentile * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn format_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
