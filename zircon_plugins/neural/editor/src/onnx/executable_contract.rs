use zircon_plugin_neural_runtime::{NnOpAttrs, NnOpCode};

use super::{OnnxAttribute, OnnxGraph, OnnxNode};
use crate::NnConversionDiagnostic;

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
    let inputs = node
        .inputs
        .iter()
        .map(|name| tensor_shape(graph, name))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| missing_tensor_diagnostic(node, graph))?;
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
            NnOpCode::Gemm => gemm_shapes_are_executable(&inputs, output, attrs),
            NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d => {
                conv_shapes_are_executable(&inputs, output, attrs, code)
            }
            NnOpCode::Relu | NnOpCode::Sigmoid | NnOpCode::Tanh => inputs[0] == output,
            NnOpCode::Add | NnOpCode::Mul | NnOpCode::Sub | NnOpCode::Div => {
                inputs.iter().all(|shape| *shape == output)
            }
            NnOpCode::MaxPool2d | NnOpCode::AvgPool2d => {
                pool_shapes_are_executable(&inputs, output, attrs)
            }
            NnOpCode::Upsample2d => resize_shapes_are_executable(&inputs, output, attrs),
            NnOpCode::BatchNorm => normalization_shapes_are_executable(&inputs, output, false),
            NnOpCode::LayerNorm => {
                layer_norm_axis_is_last(node, inputs[0].len())
                    && normalization_shapes_are_executable(&inputs, output, true)
            }
            NnOpCode::Reshape => reshape_shapes_are_executable(node, &inputs, output),
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
