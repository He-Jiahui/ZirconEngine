use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_plugin_animation_runtime::{
    compile_animation_graph_runtime, AnimationGraphCompileError, CompiledAnimationGraph,
    SkeletonTargetTable,
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
    let compiled = compile_animation_graph_runtime(&graph, targets).unwrap();

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
    assert_source_diagnostic(
        compile_animation_graph_runtime(&missing, Arc::clone(&targets)),
        "ZR-ANIM-COMP-GRAPH-006",
    );

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
    assert_source_diagnostic(
        compile_animation_graph_runtime(&cycle, Arc::clone(&targets)),
        "ZR-ANIM-COMP-GRAPH-009",
    );

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
        compile_animation_graph_runtime(&ambiguous, targets),
        Err(AnimationGraphCompileError::AmbiguousMaskTarget { .. })
    ));
}

#[test]
fn compiled_graph_falls_back_to_default_for_non_finite_parameter_override() {
    let targets = Arc::new(SkeletonTargetTable::compile(&skeleton(&[("Root", None)])).unwrap());
    let graph = weighted_graph(0.25, 1);
    let compiled = compile_animation_graph_runtime(&graph, targets).unwrap();
    let evaluation = compiled.evaluate(&AnimationParameterMap::from([(
        "weight".into(),
        AnimationParameterValue::Scalar(f32::NAN),
    )]));

    assert_eq!(evaluation.clips().len(), 2);
    assert!((evaluation.clips()[0].weight() - 0.75).abs() < 0.0001);
    assert!((evaluation.clips()[1].weight() - 0.25).abs() < 0.0001);
}

#[test]
fn compiled_graph_diamond_aggregates_shared_clip_node_once() {
    let targets = Arc::new(SkeletonTargetTable::compile(&skeleton(&[("Root", None)])).unwrap());
    let graph = diamond_graph(12);
    let compiled = compile_animation_graph_runtime(&graph, targets).unwrap();

    let evaluation = compiled.evaluate(&AnimationParameterMap::new());

    assert_eq!(compiled.node_count(), 37);
    assert_eq!(evaluation.clips().len(), 1);
    assert!((evaluation.clips()[0].weight() - 1.0).abs() < 0.0001);
}

#[test]
fn compiled_graph_preserves_inner_mask_and_additive_context() {
    let targets = Arc::new(
        SkeletonTargetTable::compile(&skeleton(&[
            ("Root", None),
            ("Spine", Some(0)),
            ("Hand", Some(1)),
        ]))
        .unwrap(),
    );
    let graph = AnimationGraphAsset {
        name: Some("nested-mask-additive".into()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "additive_weight".into(),
            default_value: AnimationParameterValue::Scalar(0.25),
        }],
        nodes: vec![
            clip_node("base", "res://animations/base.zanim"),
            clip_node("overlay", "res://animations/overlay.zanim"),
            AnimationGraphNodeAsset::Additive {
                id: "additive".into(),
                base: "base".into(),
                additive: "overlay".into(),
                weight_parameter: Some("additive_weight".into()),
            },
            AnimationGraphNodeAsset::Mask {
                id: "inner-mask".into(),
                input: "additive".into(),
                target_ids: vec!["Root/Spine/Hand".into()],
            },
            AnimationGraphNodeAsset::Mask {
                id: "outer-mask".into(),
                input: "inner-mask".into(),
                target_ids: vec!["Root/Spine".into()],
            },
            AnimationGraphNodeAsset::Output {
                source: "outer-mask".into(),
            },
        ],
    };
    let compiled = compile_animation_graph_runtime(&graph, targets).unwrap();

    let evaluation = compiled.evaluate(&AnimationParameterMap::new());

    assert_eq!(evaluation.clips().len(), 2);
    assert_eq!(
        evaluation.clips()[0].blend_mode(),
        zircon_runtime::core::framework::animation::AnimationGraphBlendMode::Base
    );
    assert_eq!(evaluation.clips()[0].target_mask(), &[false, false, true]);
    assert_eq!(
        evaluation.clips()[1].blend_mode(),
        zircon_runtime::core::framework::animation::AnimationGraphBlendMode::Additive
    );
    assert_eq!(evaluation.clips()[1].target_mask(), &[false, false, true]);
    assert!((evaluation.clips()[1].weight() - 0.25).abs() < 0.0001);
}

#[test]
fn compiled_graph_emits_clip_nodes_in_stable_source_slot_order() {
    let targets = Arc::new(SkeletonTargetTable::compile(&skeleton(&[("Root", None)])).unwrap());
    let graph = AnimationGraphAsset {
        name: Some("stable-output-order".into()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "weight".into(),
            default_value: AnimationParameterValue::Scalar(0.25),
        }],
        nodes: vec![
            clip_node("a", "res://animations/a.zanim"),
            clip_node("b", "res://animations/b.zanim"),
            AnimationGraphNodeAsset::Blend {
                id: "reverse-input-order".into(),
                inputs: vec!["b".into(), "a".into()],
                weight_parameter: Some("weight".into()),
            },
            AnimationGraphNodeAsset::Output {
                source: "reverse-input-order".into(),
            },
        ],
    };
    let compiled = compile_animation_graph_runtime(&graph, targets).unwrap();

    let evaluation = compiled.evaluate(&AnimationParameterMap::new());

    assert_eq!(evaluation.clips().len(), 2);
    assert_eq!(
        evaluation.clips()[0].clip(),
        &AssetReference::from_locator(AssetUri::parse("res://animations/a.zanim").unwrap())
    );
    assert!((evaluation.clips()[0].weight() - 0.25).abs() < 0.0001);
    assert_eq!(
        evaluation.clips()[1].clip(),
        &AssetReference::from_locator(AssetUri::parse("res://animations/b.zanim").unwrap())
    );
    assert!((evaluation.clips()[1].weight() - 0.75).abs() < 0.0001);
}

#[test]
fn compiled_graph_evaluation_is_non_recursive_for_deep_chain() {
    const DEPTH: usize = 4_096;
    let targets = Arc::new(SkeletonTargetTable::compile(&skeleton(&[("Root", None)])).unwrap());
    let mut nodes = vec![clip_node("clip", "res://animations/deep.zanim")];
    let mut previous = "clip".to_string();
    for index in 0..DEPTH {
        let id = format!("node-{index}");
        nodes.push(AnimationGraphNodeAsset::Blend {
            id: id.clone(),
            inputs: vec![previous],
            weight_parameter: None,
        });
        previous = id;
    }
    nodes.push(AnimationGraphNodeAsset::Output { source: previous });
    let graph = AnimationGraphAsset {
        name: Some("deep-runtime-evaluation".into()),
        parameters: Vec::new(),
        nodes,
    };
    let compiled = compile_animation_graph_runtime(&graph, targets).unwrap();

    let evaluation = compiled.evaluate(&AnimationParameterMap::new());

    assert_eq!(evaluation.clips().len(), 1);
    assert_eq!(evaluation.clips()[0].weight(), 1.0);
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
    let compiled = compile_animation_graph_runtime(&graph, targets).unwrap();
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

fn diamond_graph(layers: usize) -> AnimationGraphAsset {
    let mut nodes = vec![clip_node("shared", "res://animations/shared.zanim")];
    let mut previous = "shared".to_string();
    for layer in 0..layers {
        let left = format!("left-{layer}");
        let right = format!("right-{layer}");
        let merged = format!("merged-{layer}");
        nodes.push(AnimationGraphNodeAsset::Blend {
            id: left.clone(),
            inputs: vec![previous.clone()],
            weight_parameter: None,
        });
        nodes.push(AnimationGraphNodeAsset::Blend {
            id: right.clone(),
            inputs: vec![previous],
            weight_parameter: None,
        });
        nodes.push(AnimationGraphNodeAsset::Blend {
            id: merged.clone(),
            inputs: vec![left, right],
            weight_parameter: Some("weight".into()),
        });
        previous = merged;
    }
    nodes.push(AnimationGraphNodeAsset::Output { source: previous });
    AnimationGraphAsset {
        name: Some("diamond".into()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "weight".into(),
            default_value: AnimationParameterValue::Scalar(0.5),
        }],
        nodes,
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

fn assert_source_diagnostic(
    result: Result<CompiledAnimationGraph, AnimationGraphCompileError>,
    code: &str,
) {
    let Err(AnimationGraphCompileError::SourceDiagnostics(diagnostics)) = result else {
        panic!("expected framework source diagnostics");
    };
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code() == code));
}
