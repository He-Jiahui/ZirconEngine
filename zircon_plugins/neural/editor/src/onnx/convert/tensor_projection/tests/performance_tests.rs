use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_plugin_neural_runtime::NnTensorKind;

use crate::onnx::{OnnxGraph, OnnxTensor};

use super::TensorProjection;

const BENCH_TENSOR_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 2;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn borrowed_tensor_projection_preserves_ids_and_kind_precedence() {
    let mut graph = OnnxGraph::default();
    graph.inputs = vec!["input".to_owned(), "shared".to_owned()];
    graph.outputs = vec!["output".to_owned(), "shared".to_owned()];
    graph
        .tensors
        .insert("input".to_owned(), OnnxTensor::shape_only("input", [1]));
    graph.tensors.insert(
        "intermediate".to_owned(),
        OnnxTensor::shape_only("intermediate", [1]),
    );
    graph
        .tensors
        .insert("output".to_owned(), OnnxTensor::shape_only("output", [1]));
    graph
        .tensors
        .insert("shared".to_owned(), OnnxTensor::shape_only("shared", [1]));
    graph.tensors.insert(
        "weight".to_owned(),
        OnnxTensor::f32("weight", [1], vec![1.0]),
    );

    let projection = TensorProjection::new(&graph).unwrap();

    for (expected, name) in graph.tensors.keys().enumerate() {
        assert_eq!(projection.id(name), Some(expected as u16));
    }
    assert_eq!(
        projection.kind("input", &graph.tensors["input"]),
        NnTensorKind::Input
    );
    assert_eq!(
        projection.kind("output", &graph.tensors["output"]),
        NnTensorKind::Output
    );
    assert_eq!(
        projection.kind("shared", &graph.tensors["shared"]),
        NnTensorKind::Input
    );
    assert_eq!(
        projection.kind("intermediate", &graph.tensors["intermediate"]),
        NnTensorKind::Intermediate
    );
    assert_eq!(
        projection.kind("weight", &graph.tensors["weight"]),
        NnTensorKind::Weight
    );
}

#[test]
#[ignore = "release-only borrowed ONNX tensor projection benchmark"]
fn borrowed_tensor_projection_release_benchmark_evidence() {
    let graph = benchmark_graph();
    assert_eq!(legacy_checksum(&graph), optimized_checksum(&graph));

    let (legacy_samples, optimized_samples) =
        paired_samples(|| measure_legacy(&graph), || measure_optimized(&graph));
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins02 task=borrowed_onnx_tensor_projection \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
tensor_count={BENCH_TENSOR_COUNT} input_count={} output_count={} \
legacy_tensor_name_clones={BENCH_TENSOR_COUNT} optimized_tensor_name_clones=0 \
legacy_kind_lookup=linear_inputs_outputs optimized_kind_lookup=borrowed_btree_sets \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        graph.inputs.len(),
        graph.outputs.len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns,
        "borrowed tensor projection must reduce P95 by at least 90%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn benchmark_graph() -> OnnxGraph {
    let mut graph = OnnxGraph::default();
    for index in 0..BENCH_TENSOR_COUNT {
        let name = format!("tensor-{index:05}");
        if index < BENCH_TENSOR_COUNT / 4 {
            graph.inputs.push(name.clone());
        } else if index < BENCH_TENSOR_COUNT / 2 {
            graph.outputs.push(name.clone());
        }
        graph
            .tensors
            .insert(name.clone(), OnnxTensor::shape_only(name, [1]));
    }
    graph
}

fn legacy_checksum(graph: &OnnxGraph) -> usize {
    let tensor_ids = graph
        .tensors
        .keys()
        .enumerate()
        .map(|(index, name)| (name.clone(), index as u16))
        .collect::<BTreeMap<_, _>>();
    graph
        .tensors
        .iter()
        .map(|(name, tensor)| {
            usize::from(tensor_ids[name]) + legacy_kind(name, tensor, graph) as usize
        })
        .sum()
}

fn optimized_checksum(graph: &OnnxGraph) -> usize {
    let projection = TensorProjection::new(graph).unwrap();
    graph
        .tensors
        .iter()
        .map(|(name, tensor)| {
            usize::from(projection.id(name).unwrap()) + projection.kind(name, tensor) as usize
        })
        .sum()
}

fn legacy_kind(name: &str, tensor: &OnnxTensor, graph: &OnnxGraph) -> NnTensorKind {
    if tensor.values.is_some() {
        NnTensorKind::Weight
    } else if graph.inputs.iter().any(|input| input == name) {
        NnTensorKind::Input
    } else if graph.outputs.iter().any(|output| output == name) {
        NnTensorKind::Output
    } else {
        NnTensorKind::Intermediate
    }
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_legacy(graph: &OnnxGraph) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_checksum(black_box(graph)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(graph: &OnnxGraph) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(optimized_checksum(black_box(graph)));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
