use std::collections::BTreeMap;

use zircon_plugin_neural_runtime::{
    NnConv2dAttrs, NnDataType, NnGemmAttrs, NnModelAsset, NnOp, NnOpAttrs, NnOpCode, NnPool2dAttrs,
    NnTensorDesc, NnTensorKind, NN_WEIGHT_ALIGNMENT,
};

use super::{
    executable_contract::validate_executable_v1_shapes, OnnxAttribute, OnnxGraph, OnnxNode,
    OnnxTensor, OnnxTensorDataType,
};
use crate::NnConversionDiagnostic;

pub fn convert_graph(graph: &OnnxGraph) -> Result<NnModelAsset, Vec<NnConversionDiagnostic>> {
    let tensor_ids = graph
        .tensors
        .keys()
        .enumerate()
        .map(|(index, name)| u16::try_from(index).map(|id| (name.clone(), id)))
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(|_| {
            vec![NnConversionDiagnostic {
                node: "graph".to_string(),
                op_type: "TensorIdAllocation".to_string(),
                reason: "graph exceeds the V1 tensor id capacity".to_string(),
                input_shapes: Vec::new(),
            }]
        })?;
    let mut diagnostics = Vec::new();
    let mut tensors = Vec::with_capacity(graph.tensors.len());
    let mut weights = Vec::new();

    for (name, tensor) in &graph.tensors {
        match tensor.data_type {
            OnnxTensorDataType::F32 => {}
            _ => {
                diagnostics.push(diagnostic_for_tensor(
                    name,
                    tensor,
                    "only float32 tensors are supported",
                ));
                continue;
            }
        }
        let kind = tensor_kind(name, tensor, graph);
        let shape = match padded_shape(&tensor.shape) {
            Ok(shape) => shape,
            Err(reason) => {
                diagnostics.push(diagnostic_for_tensor(name, tensor, reason));
                continue;
            }
        };
        let element_count = shape.iter().try_fold(1_u64, |count, dimension| {
            count.checked_mul(u64::from(*dimension))
        });
        let Some(element_count) = element_count.filter(|count| *count <= u64::from(u32::MAX))
        else {
            let reason = if kind == NnTensorKind::Weight {
                "initializer element count exceeds the V1 tensor capacity"
            } else {
                "tensor element count exceeds the V1 backend index capacity"
            };
            diagnostics.push(diagnostic_for_tensor(name, tensor, reason));
            continue;
        };
        let mut descriptor =
            NnTensorDesc::new(NnDataType::F32, kind, tensor.shape.len() as u8, shape);
        if kind == NnTensorKind::Weight {
            let values = tensor
                .values
                .as_ref()
                .expect("weight tensors always have values");
            let Ok(expected_elements) = usize::try_from(element_count) else {
                diagnostics.push(diagnostic_for_tensor(
                    name,
                    tensor,
                    "initializer element count exceeds the V1 tensor capacity",
                ));
                continue;
            };
            if values.len() != expected_elements {
                diagnostics.push(diagnostic_for_tensor(
                    name,
                    tensor,
                    "initializer element count does not match its declared shape",
                ));
                continue;
            }
            let offset = align_weight_offset(weights.len());
            weights.resize(offset, 0);
            descriptor.weight_offset = offset as u64;
            for value in values {
                weights.extend_from_slice(&value.to_le_bytes());
            }
        }
        tensors.push((name.clone(), descriptor));
    }

    let mut ops = Vec::with_capacity(graph.nodes.len());
    for node in &graph.nodes {
        match convert_node(node, graph, &tensor_ids) {
            Ok(op) => ops.push(op),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    weights.resize(align_weight_offset(weights.len()), 0);
    let model = NnModelAsset {
        tensors: tensors.into_iter().map(|(_, tensor)| tensor).collect(),
        ops,
        weights,
    };
    if let Err(error) = model.validate() {
        return Err(vec![NnConversionDiagnostic {
            node: "graph".to_string(),
            op_type: "ModelValidation".to_string(),
            reason: error.to_string(),
            input_shapes: Vec::new(),
        }]);
    }
    Ok(model)
}

fn convert_node(
    node: &OnnxNode,
    graph: &OnnxGraph,
    tensor_ids: &BTreeMap<String, u16>,
) -> Result<NnOp, NnConversionDiagnostic> {
    let code = map_op_code(node, graph)?;
    validate_executable_v1_arity(node, graph, code)?;
    validate_executable_v1_attributes(node, graph)?;
    let inputs = node
        .inputs
        .iter()
        .map(|name| tensor_id(node, name, tensor_ids, graph))
        .collect::<Result<Vec<_>, _>>()?;
    let outputs = node
        .outputs
        .iter()
        .map(|name| tensor_id(node, name, tensor_ids, graph))
        .collect::<Result<Vec<_>, _>>()?;
    let attrs = convert_attrs(node, graph, code)?;
    validate_executable_v1_shapes(node, graph, code, &attrs)?;
    Ok(NnOp::new(code, inputs, outputs, attrs))
}

fn map_op_code(node: &OnnxNode, graph: &OnnxGraph) -> Result<NnOpCode, NnConversionDiagnostic> {
    let code = match node.op_type.as_str() {
        "Gemm" | "MatMul" => NnOpCode::Gemm,
        "Conv" => {
            let groups = positive_integer_attribute(node, graph, "group", 1)?;
            let input_channels = node
                .inputs
                .first()
                .and_then(|name| graph.tensors.get(name))
                .and_then(|tensor| tensor.shape.get(1))
                .copied()
                .unwrap_or(0);
            if groups > 1 && groups == input_channels {
                NnOpCode::DepthwiseConv2d
            } else {
                NnOpCode::Conv2d
            }
        }
        "Relu" => NnOpCode::Relu,
        "Sigmoid" => NnOpCode::Sigmoid,
        "Tanh" => NnOpCode::Tanh,
        "Add" => NnOpCode::Add,
        "Mul" => NnOpCode::Mul,
        "Sub" => NnOpCode::Sub,
        "Div" => NnOpCode::Div,
        "BatchNormalization" => NnOpCode::BatchNorm,
        "LayerNormalization" => NnOpCode::LayerNorm,
        "MaxPool" => NnOpCode::MaxPool2d,
        "AveragePool" => NnOpCode::AvgPool2d,
        "Resize" => NnOpCode::Upsample2d,
        "Concat" | "Slice" => {
            return Err(diagnostic_for_node(
                node,
                graph,
                "operator has no executable V1 backend",
            ));
        }
        "Reshape" | "Flatten" => NnOpCode::Reshape,
        _ => {
            return Err(diagnostic_for_node(
                node,
                graph,
                "operator is not in the V1 mapping table",
            ));
        }
    };
    Ok(code)
}

fn validate_executable_v1_arity(
    node: &OnnxNode,
    graph: &OnnxGraph,
    code: NnOpCode,
) -> Result<(), NnConversionDiagnostic> {
    let expected_inputs = match code {
        NnOpCode::Gemm | NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d => 2,
        NnOpCode::Add | NnOpCode::Mul | NnOpCode::Sub | NnOpCode::Div => 2,
        NnOpCode::Relu
        | NnOpCode::Sigmoid
        | NnOpCode::Tanh
        | NnOpCode::MaxPool2d
        | NnOpCode::AvgPool2d
        | NnOpCode::Upsample2d
        | NnOpCode::Reshape => 1,
        NnOpCode::BatchNorm => 5,
        NnOpCode::LayerNorm => 3,
        NnOpCode::Silu | NnOpCode::Concat | NnOpCode::Slice => {
            return Err(diagnostic_for_node(
                node,
                graph,
                "operator has no executable V1 import mapping",
            ));
        }
    };
    if node.inputs.len() == expected_inputs && node.outputs.len() == 1 {
        return Ok(());
    }
    let input_label = if expected_inputs == 1 {
        "input"
    } else {
        "inputs"
    };
    Err(diagnostic_for_node(
        node,
        graph,
        format!(
            "V1 {} requires exactly {expected_inputs} {input_label} and 1 output",
            node.op_type
        ),
    ))
}

fn validate_executable_v1_attributes(
    node: &OnnxNode,
    graph: &OnnxGraph,
) -> Result<(), NnConversionDiagnostic> {
    let allowed = match node.op_type.as_str() {
        "Gemm" => &["alpha", "beta", "transA", "transB"][..],
        "MatMul" => &[],
        "Conv" => &["strides", "pads", "dilations", "group"],
        "BatchNormalization" => &["epsilon"],
        "LayerNormalization" => &["epsilon", "axis", "stash_type"],
        "MaxPool" => &[
            "kernel_shape",
            "strides",
            "pads",
            "auto_pad",
            "ceil_mode",
            "dilations",
            "storage_order",
        ],
        "AveragePool" => &[
            "kernel_shape",
            "strides",
            "pads",
            "auto_pad",
            "ceil_mode",
            "count_include_pad",
        ],
        "Resize" => &[
            "scales",
            "mode",
            "nearest_mode",
            "coordinate_transformation_mode",
        ],
        "Flatten" => &["axis"],
        _ => &[],
    };
    if let Some(name) = node
        .attributes
        .keys()
        .find(|name| !allowed.contains(&name.as_str()))
    {
        return Err(diagnostic_for_node(
            node,
            graph,
            format!("V1 {} does not support attribute {name}", node.op_type),
        ));
    }

    match node.op_type.as_str() {
        "Gemm" => {
            let alpha = checked_float_attribute(node, graph, "alpha")?.unwrap_or(1.0);
            if !alpha.is_finite() {
                return Err(attribute_value_diagnostic(node, graph, "alpha"));
            }
            let beta = checked_float_attribute(node, graph, "beta")?.unwrap_or(0.0);
            if !beta.is_finite() || beta != 0.0 {
                return Err(diagnostic_for_node(node, graph, "V1 Gemm requires beta=0"));
            }
            for name in ["transA", "transB"] {
                if checked_int_attribute(node, graph, name)?.unwrap_or(0) != 0 {
                    return Err(diagnostic_for_node(
                        node,
                        graph,
                        format!("V1 Gemm requires {name}=0"),
                    ));
                }
            }
        }
        "Conv" => {
            for name in ["strides", "pads", "dilations"] {
                let _ = checked_ints_attribute(node, graph, name)?;
            }
            let _ = checked_int_attribute(node, graph, "group")?;
        }
        "BatchNormalization" | "LayerNormalization" => {
            let epsilon = checked_float_attribute(node, graph, "epsilon")?.unwrap_or(1.0e-5);
            if !epsilon.is_finite() || epsilon < 0.0 {
                return Err(diagnostic_for_node(
                    node,
                    graph,
                    format!("V1 {} requires a finite non-negative epsilon", node.op_type),
                ));
            }
            if node.op_type == "LayerNormalization" {
                let _ = checked_int_attribute(node, graph, "axis")?;
                if checked_int_attribute(node, graph, "stash_type")?.unwrap_or(1) != 1 {
                    return Err(diagnostic_for_node(
                        node,
                        graph,
                        "V1 LayerNormalization requires stash_type=1",
                    ));
                }
            }
        }
        "MaxPool" | "AveragePool" => {
            for name in ["kernel_shape", "strides", "pads"] {
                let _ = checked_ints_attribute(node, graph, name)?;
            }
            if checked_string_attribute(node, graph, "auto_pad")?
                .is_some_and(|value| value != "NOTSET")
            {
                return Err(diagnostic_for_node(
                    node,
                    graph,
                    format!("V1 {} requires auto_pad=NOTSET", node.op_type),
                ));
            }
            if checked_int_attribute(node, graph, "ceil_mode")?.unwrap_or(0) != 0 {
                return Err(diagnostic_for_node(
                    node,
                    graph,
                    format!("V1 {} requires ceil_mode=0", node.op_type),
                ));
            }
            if node.op_type == "MaxPool" {
                let _ = checked_ints_attribute(node, graph, "dilations")?;
                if checked_ints_attribute(node, graph, "dilations")?
                    .is_some_and(|values| values != [1, 1])
                {
                    return Err(diagnostic_for_node(
                        node,
                        graph,
                        "V1 MaxPool requires dilations=[1, 1]",
                    ));
                }
                if checked_int_attribute(node, graph, "storage_order")?.unwrap_or(0) != 0 {
                    return Err(diagnostic_for_node(
                        node,
                        graph,
                        "V1 MaxPool requires storage_order=0",
                    ));
                }
            } else if checked_int_attribute(node, graph, "count_include_pad")?.unwrap_or(0) != 0 {
                return Err(diagnostic_for_node(
                    node,
                    graph,
                    "V1 AveragePool requires count_include_pad=0",
                ));
            }
        }
        "Resize" => {
            let _ = checked_scales_attribute(node, graph, "scales")?;
            for name in ["mode", "nearest_mode", "coordinate_transformation_mode"] {
                let _ = checked_string_attribute(node, graph, name)?;
            }
        }
        "Flatten" => {
            let _ = checked_int_attribute(node, graph, "axis")?;
        }
        _ => {}
    }
    Ok(())
}

fn convert_attrs(
    node: &OnnxNode,
    graph: &OnnxGraph,
    code: NnOpCode,
) -> Result<NnOpAttrs, NnConversionDiagnostic> {
    let attrs = match code {
        NnOpCode::Gemm => NnOpAttrs::Gemm(NnGemmAttrs {
            alpha: float_attribute(&node.attributes, "alpha").unwrap_or(1.0),
            beta: float_attribute(&node.attributes, "beta").unwrap_or(0.0),
            transpose_a: int_attribute(&node.attributes, "transA").unwrap_or(0) != 0,
            transpose_b: int_attribute(&node.attributes, "transB").unwrap_or(0) != 0,
        }),
        NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d => NnOpAttrs::Conv2d(NnConv2dAttrs {
            stride: pair_attribute(node, graph, "strides", [1, 1])?,
            padding: quad_attribute(node, graph, "pads", [0, 0, 0, 0])?,
            dilation: pair_attribute(node, graph, "dilations", [1, 1])?,
            groups: positive_integer_attribute(node, graph, "group", 1)?,
        }),
        NnOpCode::MaxPool2d | NnOpCode::AvgPool2d => NnOpAttrs::Pool2d(NnPool2dAttrs {
            kernel: required_pair_attribute(node, graph, "kernel_shape")?,
            stride: pair_attribute(node, graph, "strides", [1, 1])?,
            padding: quad_attribute(node, graph, "pads", [0, 0, 0, 0])?,
        }),
        NnOpCode::BatchNorm => NnOpAttrs::BatchNorm {
            epsilon: float_attribute(&node.attributes, "epsilon").unwrap_or(1.0e-5),
        },
        NnOpCode::LayerNorm => NnOpAttrs::LayerNorm {
            epsilon: float_attribute(&node.attributes, "epsilon").unwrap_or(1.0e-5),
        },
        NnOpCode::Upsample2d => NnOpAttrs::Upsample2d {
            scale: upsample_scale_attribute(node, graph)?,
        },
        _ => NnOpAttrs::None,
    };
    Ok(attrs)
}

fn required_pair_attribute(
    node: &OnnxNode,
    graph: &OnnxGraph,
    name: &str,
) -> Result<[u32; 2], NnConversionDiagnostic> {
    if !node.attributes.contains_key(name) {
        return Err(diagnostic_for_node(
            node,
            graph,
            format!("V1 {} requires {name}", node.op_type),
        ));
    }
    pair_attribute(node, graph, name, [1, 1])
}

fn pair_attribute(
    node: &OnnxNode,
    graph: &OnnxGraph,
    name: &str,
    default: [u32; 2],
) -> Result<[u32; 2], NnConversionDiagnostic> {
    let Some(values) = ints_attribute(&node.attributes, name) else {
        return Ok(default);
    };
    if values.len() != 2 || values.iter().any(|value| *value <= 0) {
        Err(diagnostic_for_node(
            node,
            graph,
            "invalid pair-valued attribute",
        ))
    } else {
        let values = values
            .iter()
            .map(|value| u32::try_from(*value))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                diagnostic_for_node(node, graph, "pair-valued attribute does not fit into u32")
            })?;
        Ok([values[0], values[1]])
    }
}

fn quad_attribute(
    node: &OnnxNode,
    graph: &OnnxGraph,
    name: &str,
    default: [u32; 4],
) -> Result<[u32; 4], NnConversionDiagnostic> {
    let Some(values) = ints_attribute(&node.attributes, name) else {
        return Ok(default);
    };
    if values.len() != 4 || values.iter().any(|value| *value < 0) {
        Err(diagnostic_for_node(
            node,
            graph,
            "invalid quad-valued attribute",
        ))
    } else {
        let values = values
            .iter()
            .map(|value| u32::try_from(*value))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                diagnostic_for_node(node, graph, "quad-valued attribute does not fit into u32")
            })?;
        Ok([values[0], values[1], values[2], values[3]])
    }
}

fn upsample_scale_attribute(
    node: &OnnxNode,
    graph: &OnnxGraph,
) -> Result<[u32; 2], NnConversionDiagnostic> {
    if let Some(mode) = string_attribute(&node.attributes, "mode") {
        if mode != "nearest" {
            return Err(diagnostic_for_node(
                node,
                graph,
                "V1 Resize requires nearest mode",
            ));
        }
    }
    if string_attribute(&node.attributes, "nearest_mode") != Some("floor") {
        return Err(diagnostic_for_node(
            node,
            graph,
            "V1 Resize requires nearest_mode=floor",
        ));
    }
    if string_attribute(&node.attributes, "coordinate_transformation_mode") != Some("asymmetric") {
        return Err(diagnostic_for_node(
            node,
            graph,
            "V1 Resize requires coordinate_transformation_mode=asymmetric",
        ));
    }
    let values = match scale_attribute(&node.attributes, "scales") {
        Some(values) => values,
        None => {
            return Err(diagnostic_for_node(
                node,
                graph,
                "V1 Resize requires explicit scales",
            ));
        }
    };
    let spatial = match values.as_slice() {
        [height, width] => [*height, *width],
        [batch, channels, height, width] if *batch == 1.0 && *channels == 1.0 => [*height, *width],
        _ => {
            return Err(diagnostic_for_node(
                node,
                graph,
                "Resize scales must be [H, W] or NCHW [1, 1, H, W]",
            ));
        }
    };
    let to_integer = |value: f32| {
        (value.is_finite()
            && value > 0.0
            && value.fract() == 0.0
            && f64::from(value) <= f64::from(u32::MAX))
        .then_some(value as u32)
    };
    match (to_integer(spatial[0]), to_integer(spatial[1])) {
        (Some(height), Some(width)) => Ok([height, width]),
        _ => Err(diagnostic_for_node(
            node,
            graph,
            "V1 Resize requires positive integer spatial scales",
        )),
    }
}

fn positive_integer_attribute(
    node: &OnnxNode,
    graph: &OnnxGraph,
    name: &str,
    default: u32,
) -> Result<u32, NnConversionDiagnostic> {
    match int_attribute(&node.attributes, name) {
        None => Ok(default),
        Some(value) if value > 0 => u32::try_from(value).map_err(|_| {
            diagnostic_for_node(node, graph, "integer attribute does not fit into u32")
        }),
        Some(_) => Err(diagnostic_for_node(
            node,
            graph,
            "integer attribute must be positive",
        )),
    }
}

fn tensor_id(
    node: &OnnxNode,
    name: &str,
    tensor_ids: &BTreeMap<String, u16>,
    graph: &OnnxGraph,
) -> Result<u16, NnConversionDiagnostic> {
    tensor_ids.get(name).copied().ok_or_else(|| {
        diagnostic_for_node(
            node,
            graph,
            "node references a tensor without shape metadata",
        )
    })
}

fn tensor_kind(name: &str, tensor: &OnnxTensor, graph: &OnnxGraph) -> NnTensorKind {
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

fn padded_shape(shape: &[u32]) -> Result<[u32; 4], &'static str> {
    if shape.is_empty() || shape.len() > 4 || shape.contains(&0) {
        return Err("V1 tensors must have between one and four non-zero dimensions");
    }
    let mut padded = [1; 4];
    let start = padded.len() - shape.len();
    padded[start..].copy_from_slice(shape);
    Ok(padded)
}

fn align_weight_offset(offset: usize) -> usize {
    (offset + NN_WEIGHT_ALIGNMENT as usize - 1) & !(NN_WEIGHT_ALIGNMENT as usize - 1)
}

fn diagnostic_for_tensor(
    name: &str,
    tensor: &OnnxTensor,
    reason: impl Into<String>,
) -> NnConversionDiagnostic {
    NnConversionDiagnostic {
        node: name.to_string(),
        op_type: "Tensor".to_string(),
        reason: reason.into(),
        input_shapes: vec![tensor.shape.clone()],
    }
}

fn diagnostic_for_node(
    node: &OnnxNode,
    graph: &OnnxGraph,
    reason: impl Into<String>,
) -> NnConversionDiagnostic {
    NnConversionDiagnostic {
        node: node.name.clone(),
        op_type: node.op_type.clone(),
        reason: reason.into(),
        input_shapes: node
            .inputs
            .iter()
            .filter_map(|input| graph.tensors.get(input).map(|tensor| tensor.shape.clone()))
            .collect(),
    }
}

fn attribute_value_diagnostic(
    node: &OnnxNode,
    graph: &OnnxGraph,
    name: &str,
) -> NnConversionDiagnostic {
    diagnostic_for_node(
        node,
        graph,
        format!(
            "V1 {} attribute {name} has an unsupported value type",
            node.op_type
        ),
    )
}

fn checked_int_attribute(
    node: &OnnxNode,
    graph: &OnnxGraph,
    name: &str,
) -> Result<Option<i64>, NnConversionDiagnostic> {
    match node.attributes.get(name) {
        None => Ok(None),
        Some(OnnxAttribute::Int(value)) => Ok(Some(*value)),
        Some(_) => Err(attribute_value_diagnostic(node, graph, name)),
    }
}

fn checked_float_attribute(
    node: &OnnxNode,
    graph: &OnnxGraph,
    name: &str,
) -> Result<Option<f32>, NnConversionDiagnostic> {
    match node.attributes.get(name) {
        None => Ok(None),
        Some(OnnxAttribute::Float(value)) => Ok(Some(*value)),
        Some(_) => Err(attribute_value_diagnostic(node, graph, name)),
    }
}

fn checked_ints_attribute<'a>(
    node: &'a OnnxNode,
    graph: &OnnxGraph,
    name: &str,
) -> Result<Option<&'a [i64]>, NnConversionDiagnostic> {
    match node.attributes.get(name) {
        None => Ok(None),
        Some(OnnxAttribute::Ints(values)) => Ok(Some(values)),
        Some(_) => Err(attribute_value_diagnostic(node, graph, name)),
    }
}

fn checked_scales_attribute<'a>(
    node: &'a OnnxNode,
    graph: &OnnxGraph,
    name: &str,
) -> Result<Option<&'a [f32]>, NnConversionDiagnostic> {
    match node.attributes.get(name) {
        None => Ok(None),
        Some(OnnxAttribute::Floats(values)) => Ok(Some(values)),
        Some(_) => Err(attribute_value_diagnostic(node, graph, name)),
    }
}

fn checked_string_attribute<'a>(
    node: &'a OnnxNode,
    graph: &OnnxGraph,
    name: &str,
) -> Result<Option<&'a str>, NnConversionDiagnostic> {
    match node.attributes.get(name) {
        None => Ok(None),
        Some(OnnxAttribute::String(value)) => Ok(Some(value)),
        Some(_) => Err(attribute_value_diagnostic(node, graph, name)),
    }
}

fn int_attribute(attributes: &BTreeMap<String, OnnxAttribute>, name: &str) -> Option<i64> {
    match attributes.get(name) {
        Some(OnnxAttribute::Int(value)) => Some(*value),
        _ => None,
    }
}

fn float_attribute(attributes: &BTreeMap<String, OnnxAttribute>, name: &str) -> Option<f32> {
    match attributes.get(name) {
        Some(OnnxAttribute::Float(value)) => Some(*value),
        _ => None,
    }
}

fn ints_attribute<'a>(
    attributes: &'a BTreeMap<String, OnnxAttribute>,
    name: &str,
) -> Option<&'a [i64]> {
    match attributes.get(name) {
        Some(OnnxAttribute::Ints(values)) => Some(values),
        _ => None,
    }
}

fn scale_attribute(attributes: &BTreeMap<String, OnnxAttribute>, name: &str) -> Option<Vec<f32>> {
    match attributes.get(name) {
        Some(OnnxAttribute::Floats(values)) => Some(values.clone()),
        Some(OnnxAttribute::Ints(values)) => {
            Some(values.iter().map(|value| *value as f32).collect())
        }
        _ => None,
    }
}

fn string_attribute<'a>(
    attributes: &'a BTreeMap<String, OnnxAttribute>,
    name: &str,
) -> Option<&'a str> {
    match attributes.get(name) {
        Some(OnnxAttribute::String(value)) => Some(value),
        _ => None,
    }
}
