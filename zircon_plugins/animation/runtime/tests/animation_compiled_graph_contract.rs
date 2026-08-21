use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_plugin_animation_runtime::{
    AnimationGraphCompileError, CompiledAnimationGraph, SkeletonTargetTable,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::framework::animation::{AnimationParameterMap, AnimationParameterValue};

#[test]
fn compiled_graph_keeps_dense_nodes_parameters_and_mask_rows_after_source_mutation() {
    let skeleton = skeleton(&[("Root", None), ("Spine", Some(0)), ("Hand", Some(1))]);
    let targets = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());
    let mut graph = AnimationGraphAsset {
        name: Some("masked".into()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "weight".into(),
            default_value: AnimationParameterValue::Scalar(0.25),
        }],
        nodes: vec![
            clip_node("idle", "res://animations/idle.zanim"),
            clip_node("run", "res://animations/run.zanim"),
            AnimationGraphNodeAsset::Blend {
                id: "locomotion".into(),
                inputs: vec!["idle".into(), "run".into()],
                weight_parameter: Some("weight".into()),
            },
            AnimationGraphNodeAsset::Mask {
                id: "upper".into(),
                input: "locomotion".into(),
                target_ids: vec!["Root/Spine/Hand".into()],
            },
            AnimationGraphNodeAsset::Output {
                source: "upper".into(),
            },
        ],
    };
    let compiled = CompiledAnimationGraph::compile(&graph, targets).unwrap();

    graph.parameters[0].name = "renamed".into();
    if let AnimationGraphNodeAsset::Blend { inputs, .. } = &mut graph.nodes[2] {
        inputs[0] = "missing".into();
    }
    if let AnimationGraphNodeAsset::Mask { target_ids, .. } = &mut graph.nodes[3] {
        target_ids[0] = "Missing/Bone".into();
    }

    let evaluation = compiled.evaluate(&AnimationParameterMap::from([(
        "weight".into(),
        AnimationParameterValue::Scalar(0.8),
    )]));
    assert_eq!(compiled.node_count(), 4);
    assert_eq!(compiled.parameter_count(), 1);
    assert_eq!(evaluation.clips().len(), 2);
    assert!((evaluation.clips()[0].weight() - 0.2).abs() < 0.0001);
    assert!((evaluation.clips()[1].weight() - 0.8).abs() < 0.0001);
    assert_eq!(evaluation.clips()[0].target_mask(), &[false, false, true]);
}

#[test]
fn compiled_graph_rejects_missing_edges_cycles_and_ambiguous_mask_leaves() {
    let targets = Arc::new(
        SkeletonTargetTable::compile(&skeleton(&[
            ("Root", None),
            ("Left", Some(0)),
            ("Hand", Some(1)),
            ("Right", Some(0)),
            ("Hand", Some(3)),
        ]))
        .unwrap(),
    );
    let missing = AnimationGraphAsset {
        name: None,
        parameters: vec![],
        nodes: vec![AnimationGraphNodeAsset::Output {
            source: "missing".into(),
        }],
    };
    assert!(matches!(
        CompiledAnimationGraph::compile(&missing, Arc::clone(&targets)),
        Err(AnimationGraphCompileError::MissingNode { .. })
    ));

    let cycle = AnimationGraphAsset {
        name: None,
        parameters: vec![],
        nodes: vec![
            AnimationGraphNodeAsset::Blend {
                id: "a".into(),
                inputs: vec!["b".into()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Blend {
                id: "b".into(),
                inputs: vec!["a".into()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Output { source: "a".into() },
        ],
    };
    assert!(matches!(
        CompiledAnimationGraph::compile(&cycle, Arc::clone(&targets)),
        Err(AnimationGraphCompileError::Cycle { .. })
    ));

    let ambiguous = AnimationGraphAsset {
        name: None,
        parameters: vec![],
        nodes: vec![
            clip_node("clip", "res://animations/clip.zanim"),
            AnimationGraphNodeAsset::Mask {
                id: "mask".into(),
                input: "clip".into(),
                target_ids: vec!["Hand".into()],
            },
            AnimationGraphNodeAsset::Output {
                source: "mask".into(),
            },
        ],
    };
    assert!(matches!(
        CompiledAnimationGraph::compile(&ambiguous, targets),
        Err(AnimationGraphCompileError::AmbiguousMaskTarget { .. })
    ));
}

#[test]
fn compiled_graph_falls_back_to_default_for_non_finite_parameter_override() {
    let targets = Arc::new(SkeletonTargetTable::compile(&skeleton(&[("Root", None)])).unwrap());
    let graph = weighted_graph(0.25, 1);
    let compiled = CompiledAnimationGraph::compile(&graph, targets).unwrap();
    let evaluation = compiled.evaluate(&AnimationParameterMap::from([(
        "weight".into(),
        AnimationParameterValue::Scalar(f32::NAN),
    )]));

    assert_eq!(evaluation.clips().len(), 2);
    assert!((evaluation.clips()[0].weight() - 0.75).abs() < 0.0001);
    assert!((evaluation.clips()[1].weight() - 0.25).abs() < 0.0001);
}

#[test]
fn compiled_graph_evaluation_borrows_parameter_slots_without_materializing_snapshot() {
    let source = include_str!("../src/evaluation/compiled_graph/evaluate.rs");

    assert!(source.contains("fn parameter_value<'a>("));
    assert!(!source.contains(".collect::<Vec<_>>()"));
    assert!(!source.contains(".cloned()"));
}

#[test]
#[ignore = "release performance evidence"]
fn compiled_graph_borrowed_parameter_slot_release_benchmark_evidence() {
    const PARAMETER_COUNT: usize = 8_192;
    const EVALUATIONS_PER_SAMPLE: usize = 64;
    const SAMPLE_PAIRS: usize = 21;
    const THRESHOLD_BPS: u64 = 2_500;

    let targets = Arc::new(SkeletonTargetTable::compile(&skeleton(&[("Root", None)])).unwrap());
    let graph = weighted_graph(0.25, PARAMETER_COUNT);
    let compiled = CompiledAnimationGraph::compile(&graph, targets).unwrap();
    let overrides =
        AnimationParameterMap::from([("weight".into(), AnimationParameterValue::Scalar(0.75))]);
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy_parameter_snapshot(
                &graph.parameters,
                &overrides,
                EVALUATIONS_PER_SAMPLE,
            ));
            borrowed_ns.push(measure_borrowed_parameter_slot(
                &compiled,
                &overrides,
                EVALUATIONS_PER_SAMPLE,
            ));
        } else {
            borrowed_ns.push(measure_borrowed_parameter_slot(
                &compiled,
                &overrides,
                EVALUATIONS_PER_SAMPLE,
            ));
            legacy_ns.push(measure_legacy_parameter_snapshot(
                &graph.parameters,
                &overrides,
                EVALUATIONS_PER_SAMPLE,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let borrowed_p95_ns = nearest_rank(&borrowed_ns, 95);
    let ratio_bps = borrowed_p95_ns.saturating_mul(10_000) / legacy_p95_ns.max(1);
    assert!(
        ratio_bps <= THRESHOLD_BPS,
        "borrowed parameter P95 {borrowed_p95_ns} ns exceeds {THRESHOLD_BPS} bps of legacy {legacy_p95_ns} ns"
    );

    println!(
        "PERF_RESULT plugins13_graph_parameter_slots parameters={PARAMETER_COUNT} \
         evaluations_per_sample={EVALUATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         order=alternating_legacy_first_even \
         legacy_parameter_value_clones_per_sample={} \
         borrowed_parameter_value_clones_per_sample=0 legacy_p95_ns={legacy_p95_ns} \
         borrowed_p95_ns={borrowed_p95_ns} ratio_bps={ratio_bps} \
         threshold_bps={THRESHOLD_BPS} legacy_ns={} borrowed_ns={}",
        PARAMETER_COUNT * EVALUATIONS_PER_SAMPLE,
        csv(&legacy_ns),
        csv(&borrowed_ns),
    );
}

fn clip_node(id: &str, uri: &str) -> AnimationGraphNodeAsset {
    AnimationGraphNodeAsset::Clip {
        id: id.into(),
        clip: AssetReference::from_locator(AssetUri::parse(uri).unwrap()),
        playback_speed: 1.0,
        looping: true,
    }
}

fn weighted_graph(default_weight: f32, parameter_count: usize) -> AnimationGraphAsset {
    let mut parameters = (0..parameter_count.saturating_sub(1))
        .map(|index| AnimationGraphParameterAsset {
            name: format!("unused_{index}"),
            default_value: AnimationParameterValue::Scalar(index as f32),
        })
        .collect::<Vec<_>>();
    parameters.push(AnimationGraphParameterAsset {
        name: "weight".into(),
        default_value: AnimationParameterValue::Scalar(default_weight),
    });
    AnimationGraphAsset {
        name: Some("weighted".into()),
        parameters,
        nodes: vec![
            clip_node("idle", "res://animations/idle.zanim"),
            clip_node("run", "res://animations/run.zanim"),
            AnimationGraphNodeAsset::Blend {
                id: "blend".into(),
                inputs: vec!["idle".into(), "run".into()],
                weight_parameter: Some("weight".into()),
            },
            AnimationGraphNodeAsset::Output {
                source: "blend".into(),
            },
        ],
    }
}

fn measure_legacy_parameter_snapshot(
    parameters: &[AnimationGraphParameterAsset],
    overrides: &AnimationParameterMap,
    evaluations: usize,
) -> u64 {
    let started = Instant::now();
    for _ in 0..evaluations {
        let snapshot = parameters
            .iter()
            .map(|parameter| {
                overrides
                    .get(&parameter.name)
                    .filter(|value| parameter_is_finite(value))
                    .cloned()
                    .unwrap_or_else(|| parameter.default_value.clone())
            })
            .collect::<Vec<_>>();
        black_box(snapshot);
    }
    u64::try_from(started.elapsed().as_nanos())
        .unwrap_or(u64::MAX)
        .max(1)
}

fn measure_borrowed_parameter_slot(
    compiled: &CompiledAnimationGraph,
    overrides: &AnimationParameterMap,
    evaluations: usize,
) -> u64 {
    let started = Instant::now();
    for _ in 0..evaluations {
        black_box(compiled.evaluate(overrides));
    }
    u64::try_from(started.elapsed().as_nanos())
        .unwrap_or(u64::MAX)
        .max(1)
}

fn parameter_is_finite(value: &AnimationParameterValue) -> bool {
    match value {
        AnimationParameterValue::Scalar(value) => value.is_finite(),
        AnimationParameterValue::Vec2(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Vec3(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Vec4(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Bool(_)
        | AnimationParameterValue::Integer(_)
        | AnimationParameterValue::Trigger => true,
    }
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn csv(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn skeleton(bones: &[(&str, Option<u32>)]) -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: None,
        bones: bones
            .iter()
            .map(|(name, parent_index)| AnimationSkeletonBoneAsset {
                name: (*name).into(),
                parent_index: *parent_index,
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            })
            .collect(),
    }
}
