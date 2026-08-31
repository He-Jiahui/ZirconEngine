use zircon_plugin_neural_runtime::{NnOpAttrs, NnOpCode};

use super::{OnnxAttribute, OnnxGraph, OnnxNode};
use crate::NnConversionDiagnostic;

const MAX_EXECUTABLE_V1_INPUTS: usize = 5;

pub(super) fn validate_executable_v1_shapes(
    node: &OnnxNode,
    graph: &OnnxGraph,
    code: NnOpCode,
    attrs: &NnOpAttrs,
) -> Result<(), NnConversionDiagnostic> {
    if code == NnOpCode::Reshape
        && node
            .outputs
            .iter()
            .any(|output| graph.outputs.contains(output))
    {
        return Err(node_diagnostic(
            node,
            graph,
            format!(
                "V1 {} cannot materialize a graph output on the GPU backend",
                node.op_type
            ),
        ));
    }
    if node.inputs.len() > MAX_EXECUTABLE_V1_INPUTS {
        return Err(contract_diagnostic(node, graph));
    }
    let mut input_shapes: [&[u32]; MAX_EXECUTABLE_V1_INPUTS] = [&[]; MAX_EXECUTABLE_V1_INPUTS];
    for (index, name) in node.inputs.iter().enumerate() {
        input_shapes[index] =
            tensor_shape(graph, name).ok_or_else(|| missing_tensor_diagnostic(node, graph))?;
    }
    let inputs = &input_shapes[..node.inputs.len()];
    let output = tensor_shape(graph, &node.outputs[0])
        .ok_or_else(|| missing_tensor_diagnostic(node, graph))?;
    let tensor_counts_fit = inputs
        .iter()
        .copied()
        .chain(std::iter::once(output))
        .all(|shape| {
            checked_element_count(shape).is_some_and(|count| count <= u64::from(u32::MAX))
        });
    let valid = tensor_counts_fit
        && match code {
            NnOpCode::Gemm => gemm_shapes_are_executable(inputs, output, attrs),
            NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d => {
                conv_shapes_are_executable(inputs, output, attrs, code)
            }
            NnOpCode::Relu | NnOpCode::Sigmoid | NnOpCode::Tanh => inputs[0] == output,
            NnOpCode::Add | NnOpCode::Mul | NnOpCode::Sub | NnOpCode::Div => {
                inputs.iter().all(|shape| *shape == output)
            }
            NnOpCode::MaxPool2d | NnOpCode::AvgPool2d => {
                pool_shapes_are_executable(inputs, output, attrs)
            }
            NnOpCode::Upsample2d => resize_shapes_are_executable(inputs, output, attrs),
            NnOpCode::BatchNorm => normalization_shapes_are_executable(inputs, output, false),
            NnOpCode::LayerNorm => {
                layer_norm_axis_is_last(node, inputs[0].len())
                    && normalization_shapes_are_executable(inputs, output, true)
            }
            NnOpCode::Reshape => reshape_shapes_are_executable(node, inputs, output),
            NnOpCode::Silu | NnOpCode::Concat | NnOpCode::Slice => false,
        };
    if valid {
        Ok(())
    } else {
        Err(contract_diagnostic(node, graph))
    }
}

fn gemm_shapes_are_executable(inputs: &[&[u32]], output: &[u32], attrs: &NnOpAttrs) -> bool {
    matches!(attrs, NnOpAttrs::Gemm(_))
        && inputs.len() == 2
        && inputs[0].len() == 2
        && inputs[1].len() == 2
        && output.len() == 2
        && inputs[0][1] == inputs[1][0]
        && output == [inputs[0][0], inputs[1][1]]
}

fn conv_shapes_are_executable(
    inputs: &[&[u32]],
    output: &[u32],
    attrs: &NnOpAttrs,
    code: NnOpCode,
) -> bool {
    let NnOpAttrs::Conv2d(attrs) = attrs else {
        return false;
    };
    if inputs.len() != 2 || inputs[0].len() != 4 || inputs[1].len() != 4 || output.len() != 4 {
        return false;
    }
    let input = inputs[0];
    let weights = inputs[1];
    if attrs.groups == 0
        || input[1] % attrs.groups != 0
        || weights[0] % attrs.groups != 0
        || weights[1] != input[1] / attrs.groups
        || (code == NnOpCode::DepthwiseConv2d && attrs.groups != input[1])
    {
        return false;
    }
    let Some(height) = convolution_output_dimension(
        input[2],
        weights[2],
        attrs.padding[0],
        attrs.padding[2],
        attrs.stride[0],
        attrs.dilation[0],
    ) else {
        return false;
    };
    let Some(width) = convolution_output_dimension(
        input[3],
        weights[3],
        attrs.padding[1],
        attrs.padding[3],
        attrs.stride[1],
        attrs.dilation[1],
    ) else {
        return false;
    };
    output == [input[0], weights[0], height, width]
}

fn pool_shapes_are_executable(inputs: &[&[u32]], output: &[u32], attrs: &NnOpAttrs) -> bool {
    let NnOpAttrs::Pool2d(attrs) = attrs else {
        return false;
    };
    if inputs.len() != 1 || inputs[0].len() != 4 || output.len() != 4 {
        return false;
    }
    let input = inputs[0];
    let Some(height) = convolution_output_dimension(
        input[2],
        attrs.kernel[0],
        attrs.padding[0],
        attrs.padding[2],
        attrs.stride[0],
        1,
    ) else {
        return false;
    };
    let Some(width) = convolution_output_dimension(
        input[3],
        attrs.kernel[1],
        attrs.padding[1],
        attrs.padding[3],
        attrs.stride[1],
        1,
    ) else {
        return false;
    };
    output == [input[0], input[1], height, width]
        && every_pool_window_overlaps(
            input[2],
            height,
            attrs.kernel[0],
            attrs.stride[0],
            attrs.padding[0],
        )
        && every_pool_window_overlaps(
            input[3],
            width,
            attrs.kernel[1],
            attrs.stride[1],
            attrs.padding[1],
        )
}

fn resize_shapes_are_executable(inputs: &[&[u32]], output: &[u32], attrs: &NnOpAttrs) -> bool {
    let NnOpAttrs::Upsample2d { scale } = attrs else {
        return false;
    };
    if inputs.len() != 1 || inputs[0].len() != 4 || output.len() != 4 {
        return false;
    }
    let input = inputs[0];
    let Some(height) = input[2].checked_mul(scale[0]) else {
        return false;
    };
    let Some(width) = input[3].checked_mul(scale[1]) else {
        return false;
    };
    output == [input[0], input[1], height, width]
}

fn normalization_shapes_are_executable(
    inputs: &[&[u32]],
    output: &[u32],
    layer_norm: bool,
) -> bool {
    let expected_inputs = if layer_norm { 3 } else { 5 };
    if inputs.len() != expected_inputs || inputs[0] != output {
        return false;
    }
    if !layer_norm && inputs[0].len() != 4 {
        return false;
    }
    let parameter_count = if layer_norm {
        u64::from(*inputs[0].last().unwrap_or(&0))
    } else {
        u64::from(inputs[0][1])
    };
    inputs[1..]
        .iter()
        .all(|shape| checked_element_count(shape) == Some(parameter_count))
}

fn reshape_shapes_are_executable(node: &OnnxNode, inputs: &[&[u32]], output: &[u32]) -> bool {
    if inputs.len() != 1 {
        return false;
    }
    if node.op_type != "Flatten" {
        return checked_element_count(inputs[0]) == checked_element_count(output);
    }
    let rank = inputs[0].len() as i64;
    let axis = int_attribute(node, "axis").unwrap_or(1);
    let axis = if axis < 0 { axis + rank } else { axis };
    if axis < 0 || axis > rank || output.len() != 2 {
        return false;
    }
    let axis = axis as usize;
    let leading =
        checked_element_count(&inputs[0][..axis]).and_then(|count| u32::try_from(count).ok());
    let trailing =
        checked_element_count(&inputs[0][axis..]).and_then(|count| u32::try_from(count).ok());
    matches!(
        (leading, trailing),
        (Some(leading), Some(trailing)) if output == [leading, trailing]
    )
}

fn layer_norm_axis_is_last(node: &OnnxNode, rank: usize) -> bool {
    let axis = int_attribute(node, "axis").unwrap_or(-1);
    axis == -1 || usize::try_from(axis).ok() == rank.checked_sub(1)
}

fn convolution_output_dimension(
    input: u32,
    kernel: u32,
    padding_before: u32,
    padding_after: u32,
    stride: u32,
    dilation: u32,
) -> Option<u32> {
    if kernel == 0 || stride == 0 || dilation == 0 {
        return None;
    }
    let effective_kernel = kernel
        .checked_sub(1)?
        .checked_mul(dilation)?
        .checked_add(1)?;
    input
        .checked_add(padding_before)?
        .checked_add(padding_after)?
        .checked_sub(effective_kernel)
        .map(|value| value / stride + 1)
}

fn every_pool_window_overlaps(
    input: u32,
    output: u32,
    kernel: u32,
    stride: u32,
    padding_before: u32,
) -> bool {
    if output == 0 || kernel == 0 || stride == 0 || padding_before >= kernel {
        return false;
    }
    let last_start = (output - 1).checked_mul(stride);
    let input_end = input.checked_add(padding_before);
    matches!((last_start, input_end), (Some(start), Some(end)) if start < end)
}

fn checked_element_count(shape: &[u32]) -> Option<u64> {
    shape.iter().try_fold(1_u64, |count, dimension| {
        count.checked_mul(u64::from(*dimension))
    })
}

fn tensor_shape<'a>(graph: &'a OnnxGraph, name: &str) -> Option<&'a [u32]> {
    graph
        .tensors
        .get(name)
        .map(|tensor| tensor.shape.as_slice())
}

fn int_attribute(node: &OnnxNode, name: &str) -> Option<i64> {
    match node.attributes.get(name) {
        Some(OnnxAttribute::Int(value)) => Some(*value),
        _ => None,
    }
}

fn contract_diagnostic(node: &OnnxNode, graph: &OnnxGraph) -> NnConversionDiagnostic {
    node_diagnostic(
        node,
        graph,
        format!(
            "V1 {} tensor shapes are not executable by both backends",
            node.op_type
        ),
    )
}

fn missing_tensor_diagnostic(node: &OnnxNode, graph: &OnnxGraph) -> NnConversionDiagnostic {
    node_diagnostic(
        node,
        graph,
        "node references a tensor without shape metadata".to_string(),
    )
}

fn node_diagnostic(node: &OnnxNode, graph: &OnnxGraph, reason: String) -> NnConversionDiagnostic {
    NnConversionDiagnostic {
        node: node.name.clone(),
        op_type: node.op_type.clone(),
        reason,
        input_shapes: node
            .inputs
            .iter()
            .filter_map(|input| graph.tensors.get(input).map(|tensor| tensor.shape.clone()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
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

        assert!(
            validate_executable_v1_shapes(&node, &graph, NnOpCode::Relu, &NnOpAttrs::None).is_ok()
        );
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
            shapes[index] =
                tensor_shape(graph, name).expect("benchmark tensors have shape metadata");
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
}
