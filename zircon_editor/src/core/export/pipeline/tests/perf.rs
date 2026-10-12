use std::hint::black_box;
use std::time::Instant;

use super::{
    reject_duplicate_stages, reject_missing_dependencies, ExportPipelinePlan,
    ExportPipelinePlanError, ExportStage, ExportStageNode,
};

const SAMPLE_PAIRS: usize = 31;
const VALIDATIONS_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_r6_wave_editor633_preserves_error_precedence_and_stable_order() {
    let duplicate_before_missing = ExportPipelinePlan::new([
        ExportStageNode::new(ExportStage::Pack, [ExportStage::CookAssets]),
        ExportStageNode::new(ExportStage::Pack, []),
    ]);
    assert!(matches!(
        duplicate_before_missing,
        Err(ExportPipelinePlanError::DuplicateStage {
            stage: ExportStage::Pack
        })
    ));

    let plan = ExportPipelinePlan::new([
        ExportStageNode::new(ExportStage::Pack, [ExportStage::CookAssets]),
        ExportStageNode::new(ExportStage::Validate, []),
        ExportStageNode::new(ExportStage::CookAssets, [ExportStage::Validate]),
    ])
    .expect("valid out-of-order plan");
    let stages = plan
        .ordered_nodes()
        .iter()
        .map(|node| node.stage)
        .collect::<Vec<_>>();
    assert_eq!(
        stages,
        [
            ExportStage::Validate,
            ExportStage::CookAssets,
            ExportStage::Pack
        ]
    );
}

#[test]
fn optimization_batch_r6_wave_editor633_uses_stack_bitset_stage_validation() {
    let source = include_str!("../../pipeline.rs");
    assert!(source.contains("let mut seen_mask = 0_u8;"));
    assert!(source.contains("let declared_mask ="));
    assert!(source.contains("let mut completed_mask = 0_u8;"));
    assert!(!source.contains("let mut seen = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release helper microbenchmark; real export caller evidence is required"]
fn optimization_batch_r6_wave_editor633_export_stage_validation_bitset_p95() {
    let nodes = benchmark_nodes();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&nodes, false));
            optimized_samples.push(measure(&nodes, true));
        } else {
            optimized_samples.push(measure(&nodes, true));
            legacy_samples.push(measure(&nodes, false));
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let legacy_p99 = nearest_rank(&legacy_samples, 99);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let optimized_p99 = nearest_rank(&optimized_samples, 99);
    println!(
        "EDITOR633_BITSET_EXPORT_STAGE_VALIDATION_BENCH_V2 \
         sample_pairs={SAMPLE_PAIRS} validations_per_sample={VALIDATIONS_PER_SAMPLE} stages=8 \
         benchmark_scope=microbenchmark real_caller_required=true \
         percentile_method=nearest_rank legacy_p50_ns={legacy_p50} \
         legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} \
         optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} \
         optimized_p99_ns={optimized_p99} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(60),
        "stack bitset export-stage validation must be at least 40% faster at P95"
    );
}

fn benchmark_nodes() -> Vec<ExportStageNode> {
    vec![
        ExportStageNode::new(ExportStage::Validate, []),
        ExportStageNode::new(ExportStage::SourceTemplate, [ExportStage::Validate]),
        ExportStageNode::new(ExportStage::NativeDynamic, [ExportStage::SourceTemplate]),
        ExportStageNode::new(ExportStage::CompileHost, [ExportStage::NativeDynamic]),
        ExportStageNode::new(ExportStage::CookAssets, [ExportStage::Validate]),
        ExportStageNode::new(
            ExportStage::Pack,
            [ExportStage::CompileHost, ExportStage::CookAssets],
        ),
        ExportStageNode::new(ExportStage::PlatformBundle, [ExportStage::Pack]),
        ExportStageNode::new(ExportStage::Report, [ExportStage::PlatformBundle]),
    ]
}

fn measure(nodes: &[ExportStageNode], optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..VALIDATIONS_PER_SAMPLE {
        let nodes = black_box(nodes);
        if optimized {
            reject_duplicate_stages(nodes).unwrap();
            reject_missing_dependencies(nodes).unwrap();
        } else {
            legacy_reject_duplicate_stages(nodes).unwrap();
            legacy_reject_missing_dependencies(nodes).unwrap();
        }
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_reject_duplicate_stages(
    nodes: &[ExportStageNode],
) -> Result<(), ExportPipelinePlanError> {
    let mut seen = Vec::new();
    for node in nodes {
        if seen.contains(&node.stage) {
            return Err(ExportPipelinePlanError::DuplicateStage { stage: node.stage });
        }
        seen.push(node.stage);
    }
    Ok(())
}

fn legacy_reject_missing_dependencies(
    nodes: &[ExportStageNode],
) -> Result<(), ExportPipelinePlanError> {
    for node in nodes {
        for dependency in &node.dependencies {
            if !nodes.iter().any(|candidate| candidate.stage == *dependency) {
                return Err(ExportPipelinePlanError::MissingDependency {
                    stage: node.stage,
                    dependency: *dependency,
                });
            }
        }
    }
    Ok(())
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}
