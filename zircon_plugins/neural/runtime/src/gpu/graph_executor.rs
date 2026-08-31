use std::collections::BTreeMap;
use std::fmt;

mod parameters;
mod resource_aliases;

use parameters::{elementwise_parameters, gemm_parameters};
use resource_aliases::ResourceAliases;

use zircon_runtime::graphics::{ComputePassDescriptor, ComputeShaderSource, RenderPassStage};
use zircon_runtime::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphComputeDispatchExtent,
};

use crate::gpu::shader_templates::shader_for;
use crate::{
    NnConv2dAttrs, NnDataType, NnModelAsset, NnOp, NnOpAttrs, NnOpCode, NnTensorDesc, NnTensorKind,
};

const GEMM_WORKGROUP_EDGE: u32 = 16;
const ELEMENTWISE_WORKGROUP_SIZE: u32 = 64;
const CONV_WORKGROUP_SIZE: u32 = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NnGraphIo {
    weight_buffer: String,
    inputs: BTreeMap<u16, String>,
    outputs: BTreeMap<u16, String>,
}

impl NnGraphIo {
    pub fn new(weight_buffer: impl Into<String>) -> Self {
        Self {
            weight_buffer: weight_buffer.into(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }
    }

    pub fn with_input(mut self, tensor: u16, resource: impl Into<String>) -> Self {
        self.inputs.insert(tensor, resource.into());
        self
    }

    pub fn with_output(mut self, tensor: u16, resource: impl Into<String>) -> Self {
        self.outputs.insert(tensor, resource.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnGraphPassPlan {
    pub descriptor: ComputePassDescriptor,
    pub parameter_resource: String,
    pub parameter_bytes: Vec<u8>,
    pub transient_outputs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NnGraphBuildError {
    InvalidModel(String),
    MissingInputResource(u16),
    MissingOutputResource(u16),
    MissingTensor(u16),
    UnsupportedDataType(NnDataType),
    UnsupportedOp(NnOpCode),
    InvalidOpArity {
        code: NnOpCode,
        expected: usize,
        actual: usize,
    },
    InvalidOpAttrs(NnOpCode),
    InvalidShape(NnOpCode),
    UnsupportedViewOp(NnOpCode),
}

impl fmt::Display for NnGraphBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for NnGraphBuildError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnGraphExecutor {
    stage: RenderPassStage,
    queue: QueueLane,
}

impl Default for NnGraphExecutor {
    fn default() -> Self {
        Self {
            stage: RenderPassStage::PostProcess,
            queue: QueueLane::AsyncCompute,
        }
    }
}

impl NnGraphExecutor {
    pub fn build_passes(
        &self,
        model: &NnModelAsset,
        io: &NnGraphIo,
    ) -> Result<Vec<ComputePassDescriptor>, NnGraphBuildError> {
        self.build_plan(model, io)
            .map(|passes| passes.into_iter().map(|pass| pass.descriptor).collect())
    }

    pub fn build_plan(
        &self,
        model: &NnModelAsset,
        io: &NnGraphIo,
    ) -> Result<Vec<NnGraphPassPlan>, NnGraphBuildError> {
        model
            .validate()
            .map_err(|error| NnGraphBuildError::InvalidModel(error.to_string()))?;
        if model
            .tensors
            .iter()
            .any(|tensor| tensor.dtype != NnDataType::F32)
        {
            return Err(NnGraphBuildError::UnsupportedDataType(NnDataType::F16));
        }

        let mut resource_aliases = ResourceAliases::new(model.tensors.len());
        let mut plans = Vec::with_capacity(model.ops.len());
        for (op_index, op) in model.ops.iter().enumerate() {
            if op.code == NnOpCode::Reshape {
                fold_reshape(op, &mut resource_aliases, model, io)?;
                continue;
            }
            if op.code.is_view() {
                return Err(NnGraphBuildError::UnsupportedViewOp(op.code));
            }
            let output = only_output(op)?;
            let parameter_resource = format!("nn.params.{op_index}");
            let mut bindings = Vec::with_capacity(op.inputs.len() + 2);
            bindings.push(BindingSchemaEntry::new(
                0,
                parameter_resource.clone(),
                ComputeBindingKind::UniformBuffer,
            ));
            for (input_index, tensor) in op.inputs.iter().enumerate() {
                let descriptor = tensor_descriptor(model, *tensor)?;
                let mut binding = BindingSchemaEntry::new(
                    input_index as u32 + 1,
                    resource_for(*tensor, model, &resource_aliases, io)?,
                    ComputeBindingKind::StorageBufferRead,
                );
                if descriptor.kind == NnTensorKind::Weight {
                    binding = binding.with_buffer_offset(descriptor.weight_offset);
                }
                bindings.push(binding);
            }
            let output_descriptor = tensor_descriptor(model, output)?;
            let output_resource = resource_for(output, model, &resource_aliases, io)?;
            bindings.push(BindingSchemaEntry::new(
                bindings.len() as u32,
                output_resource.clone(),
                ComputeBindingKind::StorageBufferReadWrite,
            ));

            let (workgroup_size, dispatch, parameter_bytes) = dispatch_and_parameters(model, op)?;
            let shader = shader_for(op.code).ok_or(NnGraphBuildError::UnsupportedOp(op.code))?;
            let pass_name = format!("nn.{}.{}", op_name(op.code), op_index);
            let transient_outputs = if output_descriptor.kind == NnTensorKind::Intermediate {
                vec![output_resource]
            } else {
                Vec::new()
            };
            plans.push(NnGraphPassPlan {
                descriptor: ComputePassDescriptor::new(
                    pass_name.clone(),
                    self.stage,
                    self.queue,
                    ComputeShaderSource::inline_wgsl(pass_name, shader),
                    "cs_main",
                    workgroup_size,
                    bindings,
                    dispatch,
                    PassFlags::default(),
                ),
                parameter_resource,
                parameter_bytes,
                transient_outputs,
            });
        }
        Ok(plans)
    }
}

fn fold_reshape(
    op: &NnOp,
    aliases: &mut ResourceAliases,
    model: &NnModelAsset,
    io: &NnGraphIo,
) -> Result<(), NnGraphBuildError> {
    if op.inputs.len() != 1 || op.outputs.len() != 1 {
        return Err(NnGraphBuildError::InvalidOpArity {
            code: op.code,
            expected: 1,
            actual: op.inputs.len().max(op.outputs.len()),
        });
    }
    let source = op.inputs[0];
    resource_for(source, model, aliases, io)?;
    let output = op.outputs[0];
    if !aliases.alias(output, source) {
        return Err(NnGraphBuildError::MissingTensor(output));
    }
    Ok(())
}

fn only_output(op: &NnOp) -> Result<u16, NnGraphBuildError> {
    if op.outputs.len() == 1 {
        Ok(op.outputs[0])
    } else {
        Err(NnGraphBuildError::InvalidOpArity {
            code: op.code,
            expected: 1,
            actual: op.outputs.len(),
        })
    }
}

fn tensor_descriptor(
    model: &NnModelAsset,
    tensor: u16,
) -> Result<&crate::NnTensorDesc, NnGraphBuildError> {
    model
        .tensors
        .get(usize::from(tensor))
        .ok_or(NnGraphBuildError::MissingTensor(tensor))
}

fn resource_for(
    tensor: u16,
    model: &NnModelAsset,
    aliases: &ResourceAliases,
    io: &NnGraphIo,
) -> Result<String, NnGraphBuildError> {
    let tensor = aliases.resolve(tensor);
    match tensor_descriptor(model, tensor)?.kind {
        NnTensorKind::Input => io
            .inputs
            .get(&tensor)
            .cloned()
            .ok_or(NnGraphBuildError::MissingInputResource(tensor)),
        NnTensorKind::Output => io
            .outputs
            .get(&tensor)
            .cloned()
            .ok_or(NnGraphBuildError::MissingOutputResource(tensor)),
        NnTensorKind::Intermediate => Ok(format!("nn.tensor.{tensor}")),
        NnTensorKind::Weight => Ok(io.weight_buffer.clone()),
    }
}

fn dispatch_and_parameters(
    model: &NnModelAsset,
    op: &NnOp,
) -> Result<([u32; 3], RenderGraphComputeDispatchExtent, Vec<u8>), NnGraphBuildError> {
    let output = only_output(op)?;
    match op.code {
        NnOpCode::Gemm => gemm_dispatch(model, op, output),
        NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d => conv_dispatch(model, op, output),
        NnOpCode::Relu | NnOpCode::Sigmoid | NnOpCode::Tanh | NnOpCode::Silu => {
            if op.inputs.len() != 1 {
                return Err(NnGraphBuildError::InvalidOpArity {
                    code: op.code,
                    expected: 1,
                    actual: op.inputs.len(),
                });
            }
            elementwise_dispatch(model, output)
        }
        NnOpCode::Add | NnOpCode::Mul | NnOpCode::Sub | NnOpCode::Div => {
            if op.inputs.len() != 2 {
                return Err(NnGraphBuildError::InvalidOpArity {
                    code: op.code,
                    expected: 2,
                    actual: op.inputs.len(),
                });
            }
            elementwise_dispatch(model, output)
        }
        NnOpCode::MaxPool2d | NnOpCode::AvgPool2d => pool_dispatch(model, op, output),
        NnOpCode::Upsample2d => upsample_dispatch(model, op, output),
        NnOpCode::BatchNorm => batch_norm_dispatch(model, op, output),
        NnOpCode::LayerNorm => layer_norm_dispatch(model, op, output),
        code => Err(NnGraphBuildError::UnsupportedOp(code)),
    }
}

fn layer_norm_dispatch(
    model: &NnModelAsset,
    op: &NnOp,
    output: u16,
) -> Result<([u32; 3], RenderGraphComputeDispatchExtent, Vec<u8>), NnGraphBuildError> {
    if op.inputs.len() != 3 {
        return Err(NnGraphBuildError::InvalidOpArity {
            code: op.code,
            expected: 3,
            actual: op.inputs.len(),
        });
    }
    let epsilon = match &op.attrs {
        NnOpAttrs::LayerNorm { epsilon } if epsilon.is_finite() && *epsilon >= 0.0 => *epsilon,
        _ => return Err(NnGraphBuildError::InvalidOpAttrs(op.code)),
    };
    let input = tensor_descriptor(model, op.inputs[0])?;
    let output = tensor_descriptor(model, output)?;
    if output.rank != input.rank || output.shape != input.shape {
        return Err(NnGraphBuildError::InvalidShape(op.code));
    }
    let axis_size = u64::from(input.shape[3]);
    if axis_size == 0 {
        return Err(NnGraphBuildError::InvalidShape(op.code));
    }
    for tensor in &op.inputs[1..] {
        let descriptor = tensor_descriptor(model, *tensor)?;
        if descriptor.element_count() != Some(axis_size) {
            return Err(NnGraphBuildError::InvalidShape(op.code));
        }
    }
    let elements = input
        .element_count()
        .and_then(|count| u32::try_from(count).ok())
        .ok_or(NnGraphBuildError::InvalidShape(op.code))?;
    let rows = elements / input.shape[3];
    let mut parameters = Vec::with_capacity(32);
    for value in input.shape {
        parameters.extend_from_slice(&value.to_le_bytes());
    }
    parameters.extend_from_slice(&epsilon.to_le_bytes());
    parameters.resize(32, 0);
    Ok((
        [ELEMENTWISE_WORKGROUP_SIZE, 1, 1],
        RenderGraphComputeDispatchExtent::Fixed([rows.div_ceil(ELEMENTWISE_WORKGROUP_SIZE), 1, 1]),
        parameters,
    ))
}

fn batch_norm_dispatch(
    model: &NnModelAsset,
    op: &NnOp,
    output: u16,
) -> Result<([u32; 3], RenderGraphComputeDispatchExtent, Vec<u8>), NnGraphBuildError> {
    if op.inputs.len() != 5 {
        return Err(NnGraphBuildError::InvalidOpArity {
            code: op.code,
            expected: 5,
            actual: op.inputs.len(),
        });
    }
    let epsilon = match &op.attrs {
        NnOpAttrs::BatchNorm { epsilon } if epsilon.is_finite() && *epsilon >= 0.0 => *epsilon,
        _ => return Err(NnGraphBuildError::InvalidOpAttrs(op.code)),
    };
    let input = tensor_descriptor(model, op.inputs[0])?;
    let output = tensor_descriptor(model, output)?;
    if input.rank != 4 || output.shape != input.shape {
        return Err(NnGraphBuildError::InvalidShape(op.code));
    }
    let channels = u64::from(input.shape[1]);
    for tensor in &op.inputs[1..] {
        let descriptor = tensor_descriptor(model, *tensor)?;
        if descriptor.element_count() != Some(channels) {
            return Err(NnGraphBuildError::InvalidShape(op.code));
        }
    }
    let elements = input
        .element_count()
        .and_then(|count| u32::try_from(count).ok())
        .ok_or(NnGraphBuildError::InvalidShape(op.code))?;
    let mut parameters = Vec::with_capacity(32);
    for value in input.shape {
        parameters.extend_from_slice(&value.to_le_bytes());
    }
    parameters.extend_from_slice(&epsilon.to_le_bytes());
    parameters.resize(32, 0);
    Ok((
        [ELEMENTWISE_WORKGROUP_SIZE, 1, 1],
        RenderGraphComputeDispatchExtent::Fixed([
            elements.div_ceil(ELEMENTWISE_WORKGROUP_SIZE),
            1,
            1,
        ]),
        parameters,
    ))
}

fn pool_dispatch(
    model: &NnModelAsset,
    op: &NnOp,
    output: u16,
) -> Result<([u32; 3], RenderGraphComputeDispatchExtent, Vec<u8>), NnGraphBuildError> {
    if op.inputs.len() != 1 {
        return Err(NnGraphBuildError::InvalidOpArity {
            code: op.code,
            expected: 1,
            actual: op.inputs.len(),
        });
    }
    let attrs = match &op.attrs {
        NnOpAttrs::Pool2d(attrs) => attrs,
        _ => return Err(NnGraphBuildError::InvalidOpAttrs(op.code)),
    };
    let input = tensor_descriptor(model, op.inputs[0])?;
    let output = tensor_descriptor(model, output)?;
    validate_pool_shapes(input, output, attrs, op.code)?;
    let mut parameters = Vec::with_capacity(64);
    for values in [
        input.shape,
        output.shape,
        [
            attrs.kernel[0],
            attrs.kernel[1],
            attrs.stride[0],
            attrs.stride[1],
        ],
        attrs.padding,
    ] {
        for value in values {
            parameters.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok((
        [CONV_WORKGROUP_SIZE, CONV_WORKGROUP_SIZE, 1],
        RenderGraphComputeDispatchExtent::Fixed([
            output.shape[3].div_ceil(CONV_WORKGROUP_SIZE),
            output.shape[2].div_ceil(CONV_WORKGROUP_SIZE),
            output.shape[0].saturating_mul(output.shape[1]),
        ]),
        parameters,
    ))
}

fn validate_pool_shapes(
    input: &NnTensorDesc,
    output: &NnTensorDesc,
    attrs: &crate::NnPool2dAttrs,
    code: NnOpCode,
) -> Result<(), NnGraphBuildError> {
    if input.rank != 4 || output.rank != 4 {
        return Err(NnGraphBuildError::InvalidShape(code));
    }
    let output_height = conv_output_dimension(
        input.shape[2],
        attrs.kernel[0],
        attrs.padding[0],
        attrs.padding[2],
        attrs.stride[0],
        1,
    )?;
    let output_width = conv_output_dimension(
        input.shape[3],
        attrs.kernel[1],
        attrs.padding[1],
        attrs.padding[3],
        attrs.stride[1],
        1,
    )?;
    if output.shape != [input.shape[0], input.shape[1], output_height, output_width] {
        return Err(NnGraphBuildError::InvalidShape(code));
    }
    Ok(())
}

fn upsample_dispatch(
    model: &NnModelAsset,
    op: &NnOp,
    output: u16,
) -> Result<([u32; 3], RenderGraphComputeDispatchExtent, Vec<u8>), NnGraphBuildError> {
    if op.inputs.len() != 1 {
        return Err(NnGraphBuildError::InvalidOpArity {
            code: op.code,
            expected: 1,
            actual: op.inputs.len(),
        });
    }
    let scale = match &op.attrs {
        NnOpAttrs::Upsample2d { scale } if scale[0] > 0 && scale[1] > 0 => scale,
        _ => return Err(NnGraphBuildError::InvalidOpAttrs(op.code)),
    };
    let input = tensor_descriptor(model, op.inputs[0])?;
    let output = tensor_descriptor(model, output)?;
    if input.rank != 4 || output.rank != 4 {
        return Err(NnGraphBuildError::InvalidShape(op.code));
    }
    let output_height = input.shape[2]
        .checked_mul(scale[0])
        .ok_or(NnGraphBuildError::InvalidShape(op.code))?;
    let output_width = input.shape[3]
        .checked_mul(scale[1])
        .ok_or(NnGraphBuildError::InvalidShape(op.code))?;
    if output.shape != [input.shape[0], input.shape[1], output_height, output_width] {
        return Err(NnGraphBuildError::InvalidShape(op.code));
    }
    let mut parameters = Vec::with_capacity(48);
    for values in [input.shape, output.shape, [scale[0], scale[1], 0, 0]] {
        for value in values {
            parameters.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok((
        [CONV_WORKGROUP_SIZE, CONV_WORKGROUP_SIZE, 1],
        RenderGraphComputeDispatchExtent::Fixed([
            output.shape[3].div_ceil(CONV_WORKGROUP_SIZE),
            output.shape[2].div_ceil(CONV_WORKGROUP_SIZE),
            output.shape[0].saturating_mul(output.shape[1]),
        ]),
        parameters,
    ))
}

fn conv_dispatch(
    model: &NnModelAsset,
    op: &NnOp,
    output: u16,
) -> Result<([u32; 3], RenderGraphComputeDispatchExtent, Vec<u8>), NnGraphBuildError> {
    if op.inputs.len() != 2 {
        return Err(NnGraphBuildError::InvalidOpArity {
            code: op.code,
            expected: 2,
            actual: op.inputs.len(),
        });
    }
    let attrs = match &op.attrs {
        NnOpAttrs::Conv2d(attrs) => attrs,
        _ => return Err(NnGraphBuildError::InvalidOpAttrs(op.code)),
    };
    let input = tensor_descriptor(model, op.inputs[0])?;
    let weights = tensor_descriptor(model, op.inputs[1])?;
    let output = tensor_descriptor(model, output)?;
    validate_conv_shapes(input, weights, output, attrs, op.code)?;
    let mut parameters = Vec::with_capacity(80);
    for values in [
        input.shape,
        output.shape,
        [
            weights.shape[2],
            weights.shape[3],
            attrs.stride[0],
            attrs.stride[1],
        ],
        [
            attrs.padding[0],
            attrs.padding[1],
            attrs.dilation[0],
            attrs.dilation[1],
        ],
    ] {
        for value in values {
            parameters.extend_from_slice(&value.to_le_bytes());
        }
    }
    parameters.extend_from_slice(&attrs.groups.to_le_bytes());
    parameters.resize(80, 0);
    let groups = [
        output.shape[3].div_ceil(CONV_WORKGROUP_SIZE),
        output.shape[2].div_ceil(CONV_WORKGROUP_SIZE),
        output.shape[0].saturating_mul(output.shape[1]),
    ];
    Ok((
        [CONV_WORKGROUP_SIZE, CONV_WORKGROUP_SIZE, 1],
        RenderGraphComputeDispatchExtent::Fixed(groups),
        parameters,
    ))
}

fn validate_conv_shapes(
    input: &NnTensorDesc,
    weights: &NnTensorDesc,
    output: &NnTensorDesc,
    attrs: &NnConv2dAttrs,
    code: NnOpCode,
) -> Result<(), NnGraphBuildError> {
    if input.rank != 4 || weights.rank != 4 || output.rank != 4 || attrs.groups == 0 {
        return Err(NnGraphBuildError::InvalidShape(code));
    }
    if input.shape[1] % attrs.groups != 0
        || weights.shape[0] % attrs.groups != 0
        || weights.shape[1] != input.shape[1] / attrs.groups
        || (code == NnOpCode::DepthwiseConv2d && attrs.groups != input.shape[1])
    {
        return Err(NnGraphBuildError::InvalidShape(code));
    }
    let output_height = conv_output_dimension(
        input.shape[2],
        weights.shape[2],
        attrs.padding[0],
        attrs.padding[2],
        attrs.stride[0],
        attrs.dilation[0],
    )?;
    let output_width = conv_output_dimension(
        input.shape[3],
        weights.shape[3],
        attrs.padding[1],
        attrs.padding[3],
        attrs.stride[1],
        attrs.dilation[1],
    )?;
    if output.shape
        != [
            input.shape[0],
            weights.shape[0],
            output_height,
            output_width,
        ]
    {
        return Err(NnGraphBuildError::InvalidShape(code));
    }
    Ok(())
}

fn conv_output_dimension(
    input: u32,
    kernel: u32,
    padding_before: u32,
    padding_after: u32,
    stride: u32,
    dilation: u32,
) -> Result<u32, NnGraphBuildError> {
    if kernel == 0 || stride == 0 || dilation == 0 {
        return Err(NnGraphBuildError::InvalidShape(NnOpCode::Conv2d));
    }
    let effective_kernel = kernel
        .checked_sub(1)
        .and_then(|value| value.checked_mul(dilation))
        .and_then(|value| value.checked_add(1))
        .ok_or(NnGraphBuildError::InvalidShape(NnOpCode::Conv2d))?;
    input
        .checked_add(padding_before)
        .and_then(|value| value.checked_add(padding_after))
        .and_then(|value| value.checked_sub(effective_kernel))
        .map(|value| value / stride + 1)
        .ok_or(NnGraphBuildError::InvalidShape(NnOpCode::Conv2d))
}

fn gemm_dispatch(
    model: &NnModelAsset,
    op: &NnOp,
    output: u16,
) -> Result<([u32; 3], RenderGraphComputeDispatchExtent, Vec<u8>), NnGraphBuildError> {
    if op.inputs.len() != 2 {
        return Err(NnGraphBuildError::InvalidOpArity {
            code: op.code,
            expected: 2,
            actual: op.inputs.len(),
        });
    }
    let attrs = match &op.attrs {
        NnOpAttrs::Gemm(attrs) if !attrs.transpose_a && !attrs.transpose_b && attrs.beta == 0.0 => {
            attrs
        }
        _ => return Err(NnGraphBuildError::InvalidOpAttrs(op.code)),
    };
    let input = tensor_descriptor(model, op.inputs[0])?;
    let weight = tensor_descriptor(model, op.inputs[1])?;
    let output = tensor_descriptor(model, output)?;
    if input.rank != 2 || weight.rank != 2 || output.rank != 2 {
        return Err(NnGraphBuildError::InvalidShape(op.code));
    }
    let m = input.shape[2];
    let k = input.shape[3];
    let n = weight.shape[3];
    if weight.shape[2] != k || output.shape[2] != m || output.shape[3] != n {
        return Err(NnGraphBuildError::InvalidShape(op.code));
    }
    let parameters = gemm_parameters(m, n, k, attrs.alpha, attrs.beta);
    let groups = [
        n.div_ceil(GEMM_WORKGROUP_EDGE),
        m.div_ceil(GEMM_WORKGROUP_EDGE),
        1,
    ];
    Ok((
        [GEMM_WORKGROUP_EDGE, GEMM_WORKGROUP_EDGE, 1],
        RenderGraphComputeDispatchExtent::Fixed(groups),
        parameters,
    ))
}

fn elementwise_dispatch(
    model: &NnModelAsset,
    output: u16,
) -> Result<([u32; 3], RenderGraphComputeDispatchExtent, Vec<u8>), NnGraphBuildError> {
    let elements = tensor_descriptor(model, output)?
        .element_count()
        .ok_or(NnGraphBuildError::InvalidShape(NnOpCode::Relu))?;
    let elements =
        u32::try_from(elements).map_err(|_| NnGraphBuildError::InvalidShape(NnOpCode::Relu))?;
    Ok((
        [ELEMENTWISE_WORKGROUP_SIZE, 1, 1],
        RenderGraphComputeDispatchExtent::Fixed([
            elements.div_ceil(ELEMENTWISE_WORKGROUP_SIZE),
            1,
            1,
        ]),
        elementwise_parameters(elements),
    ))
}

fn op_name(code: NnOpCode) -> &'static str {
    match code {
        NnOpCode::Gemm => "gemm",
        NnOpCode::Conv2d => "conv2d",
        NnOpCode::DepthwiseConv2d => "depthwise-conv2d",
        NnOpCode::Add => "add",
        NnOpCode::Mul => "mul",
        NnOpCode::Sub => "sub",
        NnOpCode::Div => "div",
        NnOpCode::Relu => "relu",
        NnOpCode::Sigmoid => "sigmoid",
        NnOpCode::Tanh => "tanh",
        NnOpCode::Silu => "silu",
        NnOpCode::MaxPool2d => "max-pool2d",
        NnOpCode::AvgPool2d => "avg-pool2d",
        NnOpCode::Upsample2d => "upsample2d",
        NnOpCode::BatchNorm => "batch-norm",
        NnOpCode::LayerNorm => "layer-norm",
        _ => "unsupported",
    }
}
