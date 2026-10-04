use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::super::OnnxTensor;
use super::*;

const ADMISSIONS_PER_SAMPLE: usize = 65_536;
const WARMUP_PAIRS: usize = 4;
const SAMPLE_PAIRS: usize = 21;

struct CountingAllocator;

thread_local! {
    static ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation();
        unsafe { System.realloc(pointer, layout, size) }
    }
}

fn record_allocation() {
    let _ = COUNT_ALLOCATIONS.try_with(|enabled| {
        if enabled.get() {
            let _ = ALLOCATION_COUNT.try_with(|count| count.set(count.get() + 1));
        }
    });
}

#[test]
fn plugins02_stack_bounded_input_shapes_preserve_relu_admission() {
    let node = OnnxNode {
        name: "relu".to_string(),
        op_type: "Relu".to_string(),
        inputs: vec!["input".to_string()],
        outputs: vec!["output".to_string()],
        attributes: BTreeMap::new(),
    };
    let graph = OnnxGraph {
        tensors: BTreeMap::from([
            ("input".to_string(), OnnxTensor::shape_only("input", [1, 4])),
            (
                "output".to_string(),
                OnnxTensor::shape_only("output", [1, 4]),
            ),
        ]),
        ..OnnxGraph::default()
    };

    assert!(validate_executable_v1_shapes(&node, &graph, NnOpCode::Relu, &NnOpAttrs::None).is_ok());
}

struct StackInputShapes<'a> {
    shapes: [&'a [u32]; MAX_EXECUTABLE_V1_INPUTS],
    len: usize,
}

impl<'a> StackInputShapes<'a> {
    fn as_slice(&self) -> &[&'a [u32]] {
        &self.shapes[..self.len]
    }
}

struct AdmissionSample {
    elapsed_ns: u128,
    allocations: usize,
    checksum: u64,
}

#[inline(never)]
fn legacy_input_shapes<'a>(node: &OnnxNode, graph: &'a OnnxGraph) -> Vec<&'a [u32]> {
    node.inputs
        .iter()
        .map(|name| tensor_shape(graph, name))
        .collect::<Option<Vec<_>>>()
        .expect("benchmark tensors have shape metadata")
}

#[inline(never)]
fn stack_input_shapes<'a>(node: &OnnxNode, graph: &'a OnnxGraph) -> StackInputShapes<'a> {
    let mut shapes: [&[u32]; MAX_EXECUTABLE_V1_INPUTS] = [&[]; MAX_EXECUTABLE_V1_INPUTS];
    for (index, name) in node.inputs.iter().enumerate() {
        shapes[index] = tensor_shape(graph, name).expect("benchmark tensors have shape metadata");
    }
    StackInputShapes {
        shapes,
        len: node.inputs.len(),
    }
}

fn shape_checksum(mut checksum: u64, shapes: &[&[u32]], admission: usize) -> u64 {
    checksum = checksum
        .wrapping_mul(1_099_511_628_211)
        .wrapping_add(admission as u64);
    for shape in shapes {
        checksum ^= shape.len() as u64;
        for dimension in *shape {
            checksum = checksum.rotate_left(5) ^ u64::from(*dimension);
        }
    }
    checksum
}

fn legacy_admission_checksum(nodes: &[OnnxNode; 5], graph: &OnnxGraph) -> u64 {
    let mut checksum = 14_695_981_039_346_656_037_u64;
    for admission in 0..ADMISSIONS_PER_SAMPLE {
        let inputs = black_box(legacy_input_shapes(&nodes[admission % nodes.len()], graph));
        checksum = shape_checksum(checksum, &inputs, admission);
    }
    checksum
}

fn stack_admission_checksum(nodes: &[OnnxNode; 5], graph: &OnnxGraph) -> u64 {
    let mut checksum = 14_695_981_039_346_656_037_u64;
    for admission in 0..ADMISSIONS_PER_SAMPLE {
        let inputs = black_box(stack_input_shapes(&nodes[admission % nodes.len()], graph));
        checksum = shape_checksum(checksum, inputs.as_slice(), admission);
    }
    checksum
}

fn measure_admissions(operation: impl FnOnce() -> u64) -> AdmissionSample {
    ALLOCATION_COUNT.set(0);
    COUNT_ALLOCATIONS.set(true);
    let started = Instant::now();
    let checksum = operation();
    let elapsed_ns = started.elapsed().as_nanos();
    COUNT_ALLOCATIONS.set(false);
    AdmissionSample {
        elapsed_ns,
        allocations: ALLOCATION_COUNT.get(),
        checksum,
    }
}

fn performance_fixture() -> ([OnnxNode; 5], OnnxGraph) {
    let input_names = ["input_0", "input_1", "input_2", "input_3", "input_4"];
    let nodes = std::array::from_fn(|last_input| OnnxNode {
        name: format!("node_{last_input}"),
        op_type: "Benchmark".to_string(),
        inputs: input_names[..=last_input]
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        outputs: vec!["output".to_string()],
        attributes: BTreeMap::new(),
    });
    let graph = OnnxGraph {
        tensors: BTreeMap::from([
            ("input_0".into(), OnnxTensor::shape_only("input_0", [1])),
            ("input_1".into(), OnnxTensor::shape_only("input_1", [1, 4])),
            (
                "input_2".into(),
                OnnxTensor::shape_only("input_2", [1, 4, 8]),
            ),
            (
                "input_3".into(),
                OnnxTensor::shape_only("input_3", [1, 4, 8, 8]),
            ),
            ("input_4".into(), OnnxTensor::shape_only("input_4", [4])),
        ]),
        ..OnnxGraph::default()
    };
    (nodes, graph)
}

fn percentile(samples: &[u128; SAMPLE_PAIRS], percentile: usize) -> u128 {
    let mut sorted = *samples;
    sorted.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn reduction_percent(legacy_ns: u128, stack_ns: u128) -> f64 {
    legacy_ns.saturating_sub(stack_ns) as f64 * 100.0 / legacy_ns as f64
}

#[test]
#[ignore = "managed performance evidence"]
fn plugins02_stack_bounded_shape_admission_performance() {
    let (nodes, graph) = performance_fixture();
    for _ in 0..WARMUP_PAIRS {
        black_box(legacy_admission_checksum(&nodes, &graph));
        black_box(stack_admission_checksum(&nodes, &graph));
    }

    let mut legacy_ns_raw = [0_u128; SAMPLE_PAIRS];
    let mut stack_ns_raw = [0_u128; SAMPLE_PAIRS];
    let mut checksum = None;
    for sample_index in 0..SAMPLE_PAIRS {
        let (legacy, stack) = if sample_index % 2 == 0 {
            (
                measure_admissions(|| legacy_admission_checksum(&nodes, &graph)),
                measure_admissions(|| stack_admission_checksum(&nodes, &graph)),
            )
        } else {
            let stack = measure_admissions(|| stack_admission_checksum(&nodes, &graph));
            let legacy = measure_admissions(|| legacy_admission_checksum(&nodes, &graph));
            (legacy, stack)
        };
        assert_eq!(legacy.checksum, stack.checksum);
        let expected_checksum = *checksum.get_or_insert(legacy.checksum);
        assert_eq!(expected_checksum, legacy.checksum);
        assert_eq!(legacy.allocations, ADMISSIONS_PER_SAMPLE);
        assert_eq!(stack.allocations, 0);
        legacy_ns_raw[sample_index] = legacy.elapsed_ns;
        stack_ns_raw[sample_index] = stack.elapsed_ns;
    }

    let legacy_allocations = ADMISSIONS_PER_SAMPLE;
    let stack_allocations = 0;
    let p50_legacy_ns = percentile(&legacy_ns_raw, 50);
    let p50_stack_ns = percentile(&stack_ns_raw, 50);
    let p95_legacy_ns = percentile(&legacy_ns_raw, 95);
    let p95_stack_ns = percentile(&stack_ns_raw, 95);
    let p50_reduction_percent = reduction_percent(p50_legacy_ns, p50_stack_ns);
    let p95_reduction_percent = reduction_percent(p95_legacy_ns, p95_stack_ns);
    println!(
        "PERF_RESULT plugins02_stack_bounded_shape_admission \
             admissions={} legacy_allocations={} stack_allocations={} \
             p50_legacy_ns={} p50_stack_ns={} p50_reduction_percent={:.4} \
             p95_legacy_ns={} p95_stack_ns={} p95_reduction_percent={:.4} \
             checksum={} legacy_ns_raw={legacy_ns_raw:?} stack_ns_raw={stack_ns_raw:?}",
        ADMISSIONS_PER_SAMPLE,
        legacy_allocations,
        stack_allocations,
        p50_legacy_ns,
        p50_stack_ns,
        p50_reduction_percent,
        p95_legacy_ns,
        p95_stack_ns,
        p95_reduction_percent,
        checksum.expect("samples record a checksum"),
    );

    assert!(p50_reduction_percent >= 70.0);
    assert!(p95_reduction_percent >= 40.0);
}
