use std::collections::BTreeMap;
use std::fmt;

use crate::{
    NnConv2dAttrs, NnDataType, NnModelAsset, NnOpAttrs, NnOpCode, NnTensorDesc, NnTensorKind,
};

#[derive(Clone, Debug, PartialEq)]
pub enum NnCpuError {
    InvalidModel(String),
    MissingInput(u16),
    InvalidInputLength {
        tensor: u16,
        expected: usize,
        actual: usize,
    },
    InvalidWeightLength {
        tensor: u16,
        expected: usize,
        actual: usize,
    },
    MissingTensor(u16),
    UnsupportedOp(NnOpCode),
    InvalidOpArity {
        code: NnOpCode,
        expected: usize,
        actual: usize,
    },
    ShapeMismatch {
        code: NnOpCode,
    },
    UnsupportedDataType(NnDataType),
}

impl fmt::Display for NnCpuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for NnCpuError {}

pub fn run_cpu(
    model: &NnModelAsset,
    inputs: &[(u16, &[f32])],
) -> Result<Vec<Vec<f32>>, NnCpuError> {
    model
        .validate()
        .map_err(|error| NnCpuError::InvalidModel(error.to_string()))?;
    let mut tensors = BTreeMap::<u16, Vec<f32>>::new();
    load_inputs(model, inputs, &mut tensors)?;
    load_weights(model, &mut tensors)?;

    for op in &model.ops {
        let output = match op.code {
            NnOpCode::Gemm => execute_gemm(model, op, &tensors)?,
            NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d => execute_conv2d(model, op, &tensors)?,
            NnOpCode::Relu | NnOpCode::Sigmoid | NnOpCode::Tanh | NnOpCode::Silu => {
                execute_unary(op.code, op, &tensors)?
            }
            NnOpCode::Add | NnOpCode::Mul | NnOpCode::Sub | NnOpCode::Div => {
                execute_binary(op.code, op, &tensors)?
            }
            NnOpCode::MaxPool2d | NnOpCode::AvgPool2d => execute_pool2d(model, op, &tensors)?,
            NnOpCode::Upsample2d => execute_upsample2d(model, op, &tensors)?,
            NnOpCode::BatchNorm => execute_batch_norm(model, op, &tensors)?,
            NnOpCode::LayerNorm => execute_layer_norm(model, op, &tensors)?,
            NnOpCode::Reshape => execute_reshape(op, &tensors)?,
            code => return Err(NnCpuError::UnsupportedOp(code)),
        };
        if op.outputs.len() != 1 {
            return Err(NnCpuError::InvalidOpArity {
                code: op.code,
                expected: 1,
                actual: op.outputs.len(),
            });
        }
        let output_id = op.outputs[0];
        let descriptor = model
            .tensors
            .get(usize::from(output_id))
            .ok_or(NnCpuError::MissingTensor(output_id))?;
        let expected = element_count(descriptor)?;
        if output.len() != expected {
            return Err(NnCpuError::ShapeMismatch { code: op.code });
        }
        tensors.insert(output_id, output);
    }

    model
        .tensors
        .iter()
        .enumerate()
        .filter(|(_, tensor)| tensor.kind == NnTensorKind::Output)
        .map(|(index, _)| {
            let tensor_id = index as u16;
            tensors
                .remove(&tensor_id)
                .ok_or(NnCpuError::MissingTensor(tensor_id))
        })
        .collect()
}

fn load_inputs(
    model: &NnModelAsset,
    inputs: &[(u16, &[f32])],
    tensors: &mut BTreeMap<u16, Vec<f32>>,
) -> Result<(), NnCpuError> {
    for (index, descriptor) in model.tensors.iter().enumerate() {
        if descriptor.kind != NnTensorKind::Input {
            continue;
        }
        let tensor_id = index as u16;
        let (_, values) = inputs
            .iter()
            .find(|(provided_id, _)| *provided_id == tensor_id)
            .ok_or(NnCpuError::MissingInput(tensor_id))?;
        let expected = element_count(descriptor)?;
        if values.len() != expected {
            return Err(NnCpuError::InvalidInputLength {
                tensor: tensor_id,
                expected,
                actual: values.len(),
            });
        }
        tensors.insert(tensor_id, values.to_vec());
    }
    Ok(())
}

fn load_weights(
    model: &NnModelAsset,
    tensors: &mut BTreeMap<u16, Vec<f32>>,
) -> Result<(), NnCpuError> {
    for (index, descriptor) in model.tensors.iter().enumerate() {
        if descriptor.kind != NnTensorKind::Weight {
            continue;
        }
        if descriptor.dtype != NnDataType::F32 {
            return Err(NnCpuError::UnsupportedDataType(descriptor.dtype));
        }
        let element_count = element_count(descriptor)?;
        let byte_count = element_count
            .checked_mul(4)
            .ok_or(NnCpuError::ShapeMismatch {
                code: NnOpCode::Gemm,
            })?;
        let offset = usize::try_from(descriptor.weight_offset).map_err(|_| {
            NnCpuError::InvalidWeightLength {
                tensor: index as u16,
                expected: byte_count,
                actual: 0,
            }
        })?;
        let end = offset
            .checked_add(byte_count)
            .ok_or(NnCpuError::InvalidWeightLength {
                tensor: index as u16,
                expected: byte_count,
                actual: model.weights.len().saturating_sub(offset),
            })?;
        let bytes = model
            .weights
            .get(offset..end)
            .ok_or(NnCpuError::InvalidWeightLength {
                tensor: index as u16,
                expected: byte_count,
                actual: model.weights.len().saturating_sub(offset),
            })?;
        let values = bytes
            .chunks_exact(4)
            .map(|value| f32::from_le_bytes(value.try_into().unwrap()))
            .collect();
        tensors.insert(index as u16, values);
    }
    Ok(())
}

fn execute_gemm(
    model: &NnModelAsset,
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if !(2..=3).contains(&op.inputs.len()) {
        return Err(NnCpuError::InvalidOpArity {
            code: op.code,
            expected: 2,
            actual: op.inputs.len(),
        });
    }
    let attrs = match &op.attrs {
        NnOpAttrs::Gemm(attrs) => attrs,
        _ => return Err(NnCpuError::ShapeMismatch { code: op.code }),
    };
    let a = tensor(tensors, op.inputs[0])?;
    let b = tensor(tensors, op.inputs[1])?;
    let a_shape = matrix_shape(model, op.inputs[0])?;
    let b_shape = matrix_shape(model, op.inputs[1])?;
    let (a_rows, a_columns) = if attrs.transpose_a {
        (a_shape.1, a_shape.0)
    } else {
        a_shape
    };
    let (b_rows, b_columns) = if attrs.transpose_b {
        (b_shape.1, b_shape.0)
    } else {
        b_shape
    };
    if a_columns != b_rows {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    let bias = if op.inputs.len() == 3 {
        Some(tensor(tensors, op.inputs[2])?)
    } else {
        None
    };
    let mut result = vec![0.0; a_rows * b_columns];
    for row in 0..a_rows {
        for column in 0..b_columns {
            let mut value = 0.0;
            for index in 0..a_columns {
                let a_index = if attrs.transpose_a {
                    index * a_shape.1 + row
                } else {
                    row * a_shape.1 + index
                };
                let b_index = if attrs.transpose_b {
                    column * b_shape.1 + index
                } else {
                    index * b_shape.1 + column
                };
                value += a[a_index] * b[b_index];
            }
            let bias_value = bias.map_or(0.0, |bias| bias[column]);
            result[row * b_columns + column] = attrs.alpha * value + attrs.beta * bias_value;
        }
    }
    Ok(result)
}

fn execute_conv2d(
    model: &NnModelAsset,
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if !(2..=3).contains(&op.inputs.len()) {
        return Err(NnCpuError::InvalidOpArity {
            code: op.code,
            expected: 2,
            actual: op.inputs.len(),
        });
    }
    let attrs = match &op.attrs {
        NnOpAttrs::Conv2d(attrs) => attrs,
        _ => return Err(NnCpuError::ShapeMismatch { code: op.code }),
    };
    let input_desc = descriptor(model, op.inputs[0])?;
    let weight_desc = descriptor(model, op.inputs[1])?;
    let output_desc = descriptor(model, only_output_id(op)?)?;
    if input_desc.rank != 4 || weight_desc.rank != 4 || output_desc.rank != 4 {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    validate_conv_shape(input_desc, weight_desc, output_desc, attrs, op.code)?;
    let input = tensor(tensors, op.inputs[0])?;
    let weights = tensor(tensors, op.inputs[1])?;
    let bias = if op.inputs.len() == 3 {
        Some(tensor(tensors, op.inputs[2])?)
    } else {
        None
    };
    let [batch, channels, input_height, input_width] = input_desc.shape.map(|value| value as usize);
    let [output_channels, channels_per_group, kernel_height, kernel_width] =
        weight_desc.shape.map(|value| value as usize);
    let [_, _, output_height, output_width] = output_desc.shape.map(|value| value as usize);
    if bias.is_some_and(|values| values.len() != output_channels) {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    let groups = attrs.groups as usize;
    let output_channels_per_group = output_channels / groups;
    let mut output = vec![0.0; element_count(output_desc)?];
    for batch_index in 0..batch {
        for output_channel in 0..output_channels {
            let group = output_channel / output_channels_per_group;
            let input_channel_start = group * channels_per_group;
            for output_y in 0..output_height {
                for output_x in 0..output_width {
                    let mut value = bias.map_or(0.0, |bias| bias[output_channel]);
                    for input_channel_in_group in 0..channels_per_group {
                        for kernel_y in 0..kernel_height {
                            for kernel_x in 0..kernel_width {
                                let input_y = output_y * attrs.stride[0] as usize
                                    + kernel_y * attrs.dilation[0] as usize;
                                let input_x = output_x * attrs.stride[1] as usize
                                    + kernel_x * attrs.dilation[1] as usize;
                                let input_y = input_y as isize - attrs.padding[0] as isize;
                                let input_x = input_x as isize - attrs.padding[1] as isize;
                                if input_y < 0
                                    || input_x < 0
                                    || input_y >= input_height as isize
                                    || input_x >= input_width as isize
                                {
                                    continue;
                                }
                                let input_index = nchw_index(
                                    batch_index,
                                    input_channel_start + input_channel_in_group,
                                    input_y as usize,
                                    input_x as usize,
                                    channels,
                                    input_height,
                                    input_width,
                                );
                                let weight_index = ((output_channel * channels_per_group
                                    + input_channel_in_group)
                                    * kernel_height
                                    + kernel_y)
                                    * kernel_width
                                    + kernel_x;
                                value += input[input_index] * weights[weight_index];
                            }
                        }
                    }
                    let output_index = nchw_index(
                        batch_index,
                        output_channel,
                        output_y,
                        output_x,
                        output_channels,
                        output_height,
                        output_width,
                    );
                    output[output_index] = value;
                }
            }
        }
    }
    Ok(output)
}

fn validate_conv_shape(
    input: &NnTensorDesc,
    weight: &NnTensorDesc,
    output: &NnTensorDesc,
    attrs: &NnConv2dAttrs,
    code: NnOpCode,
) -> Result<(), NnCpuError> {
    let groups = attrs.groups as usize;
    let input_channels = input.shape[1] as usize;
    let output_channels = weight.shape[0] as usize;
    if groups == 0
        || input_channels % groups != 0
        || output_channels % groups != 0
        || weight.shape[1] as usize != input_channels / groups
    {
        return Err(NnCpuError::ShapeMismatch { code });
    }
    if code == NnOpCode::DepthwiseConv2d && groups != input_channels {
        return Err(NnCpuError::ShapeMismatch { code });
    }
    let output_height = convolution_output_size(
        input.shape[2] as usize,
        weight.shape[2] as usize,
        attrs.padding[0] as usize,
        attrs.padding[2] as usize,
        attrs.stride[0] as usize,
        attrs.dilation[0] as usize,
    )?;
    let output_width = convolution_output_size(
        input.shape[3] as usize,
        weight.shape[3] as usize,
        attrs.padding[1] as usize,
        attrs.padding[3] as usize,
        attrs.stride[1] as usize,
        attrs.dilation[1] as usize,
    )?;
    if output.shape
        != [
            input.shape[0],
            weight.shape[0],
            output_height as u32,
            output_width as u32,
        ]
    {
        return Err(NnCpuError::ShapeMismatch { code });
    }
    Ok(())
}

fn convolution_output_size(
    input: usize,
    kernel: usize,
    padding_before: usize,
    padding_after: usize,
    stride: usize,
    dilation: usize,
) -> Result<usize, NnCpuError> {
    if stride == 0 || dilation == 0 || kernel == 0 {
        return Err(NnCpuError::ShapeMismatch {
            code: NnOpCode::Conv2d,
        });
    }
    let effective_kernel = (kernel - 1)
        .checked_mul(dilation)
        .and_then(|value| value.checked_add(1))
        .ok_or(NnCpuError::ShapeMismatch {
            code: NnOpCode::Conv2d,
        })?;
    input
        .checked_add(padding_before)
        .and_then(|value| value.checked_add(padding_after))
        .and_then(|value| value.checked_sub(effective_kernel))
        .map(|value| value / stride + 1)
        .ok_or(NnCpuError::ShapeMismatch {
            code: NnOpCode::Conv2d,
        })
}

fn nchw_index(
    batch: usize,
    channel: usize,
    y: usize,
    x: usize,
    channels: usize,
    height: usize,
    width: usize,
) -> usize {
    ((batch * channels + channel) * height + y) * width + x
}

fn execute_unary(
    code: NnOpCode,
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if op.inputs.len() != 1 {
        return Err(NnCpuError::InvalidOpArity {
            code,
            expected: 1,
            actual: op.inputs.len(),
        });
    }
    let source = tensor(tensors, op.inputs[0])?;
    Ok(source
        .iter()
        .map(|value| match code {
            NnOpCode::Relu => value.max(0.0),
            NnOpCode::Sigmoid => 1.0 / (1.0 + (-value).exp()),
            NnOpCode::Tanh => value.tanh(),
            NnOpCode::Silu => value / (1.0 + (-value).exp()),
            _ => unreachable!("only unary elementwise ops are dispatched here"),
        })
        .collect())
}

fn execute_binary(
    code: NnOpCode,
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if op.inputs.len() != 2 {
        return Err(NnCpuError::InvalidOpArity {
            code,
            expected: 2,
            actual: op.inputs.len(),
        });
    }
    let lhs = tensor(tensors, op.inputs[0])?;
    let rhs = tensor(tensors, op.inputs[1])?;
    if lhs.len() != rhs.len() {
        return Err(NnCpuError::ShapeMismatch { code });
    }
    Ok(lhs
        .iter()
        .zip(rhs)
        .map(|(lhs, rhs)| match code {
            NnOpCode::Add => lhs + rhs,
            NnOpCode::Mul => lhs * rhs,
            NnOpCode::Sub => lhs - rhs,
            NnOpCode::Div => lhs / rhs,
            _ => unreachable!("only binary elementwise ops are dispatched here"),
        })
        .collect())
}

fn execute_pool2d(
    model: &NnModelAsset,
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if op.inputs.len() != 1 {
        return Err(NnCpuError::InvalidOpArity {
            code: op.code,
            expected: 1,
            actual: op.inputs.len(),
        });
    }
    let attrs = match &op.attrs {
        NnOpAttrs::Pool2d(attrs) => attrs,
        _ => return Err(NnCpuError::ShapeMismatch { code: op.code }),
    };
    let input_desc = descriptor(model, op.inputs[0])?;
    let output_desc = descriptor(model, only_output_id(op)?)?;
    if input_desc.rank != 4 || output_desc.rank != 4 {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    validate_pool_shape(input_desc, output_desc, attrs, op.code)?;
    let input = tensor(tensors, op.inputs[0])?;
    let [batch, channels, input_height, input_width] = input_desc.shape.map(|value| value as usize);
    let [_, _, output_height, output_width] = output_desc.shape.map(|value| value as usize);
    let kernel_height = attrs.kernel[0] as usize;
    let kernel_width = attrs.kernel[1] as usize;
    let mut output = vec![0.0; element_count(output_desc)?];

    for batch_index in 0..batch {
        for channel in 0..channels {
            for output_y in 0..output_height {
                for output_x in 0..output_width {
                    let mut value = if op.code == NnOpCode::MaxPool2d {
                        f32::NEG_INFINITY
                    } else {
                        0.0
                    };
                    let mut valid_samples = 0_usize;
                    for kernel_y in 0..kernel_height {
                        for kernel_x in 0..kernel_width {
                            let input_y = output_y * attrs.stride[0] as usize + kernel_y;
                            let input_x = output_x * attrs.stride[1] as usize + kernel_x;
                            let input_y = input_y as isize - attrs.padding[0] as isize;
                            let input_x = input_x as isize - attrs.padding[1] as isize;
                            if input_y < 0
                                || input_x < 0
                                || input_y >= input_height as isize
                                || input_x >= input_width as isize
                            {
                                continue;
                            }
                            let sample = input[nchw_index(
                                batch_index,
                                channel,
                                input_y as usize,
                                input_x as usize,
                                channels,
                                input_height,
                                input_width,
                            )];
                            valid_samples += 1;
                            if op.code == NnOpCode::MaxPool2d {
                                value = value.max(sample);
                            } else {
                                value += sample;
                            }
                        }
                    }
                    if valid_samples == 0 {
                        return Err(NnCpuError::ShapeMismatch { code: op.code });
                    }
                    if op.code == NnOpCode::AvgPool2d {
                        value /= valid_samples as f32;
                    }
                    output[nchw_index(
                        batch_index,
                        channel,
                        output_y,
                        output_x,
                        channels,
                        output_height,
                        output_width,
                    )] = value;
                }
            }
        }
    }
    Ok(output)
}

fn validate_pool_shape(
    input: &NnTensorDesc,
    output: &NnTensorDesc,
    attrs: &crate::NnPool2dAttrs,
    code: NnOpCode,
) -> Result<(), NnCpuError> {
    let output_height = convolution_output_size(
        input.shape[2] as usize,
        attrs.kernel[0] as usize,
        attrs.padding[0] as usize,
        attrs.padding[2] as usize,
        attrs.stride[0] as usize,
        1,
    )?;
    let output_width = convolution_output_size(
        input.shape[3] as usize,
        attrs.kernel[1] as usize,
        attrs.padding[1] as usize,
        attrs.padding[3] as usize,
        attrs.stride[1] as usize,
        1,
    )?;
    if output.shape
        != [
            input.shape[0],
            input.shape[1],
            output_height as u32,
            output_width as u32,
        ]
    {
        return Err(NnCpuError::ShapeMismatch { code });
    }
    Ok(())
}

fn execute_upsample2d(
    model: &NnModelAsset,
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if op.inputs.len() != 1 {
        return Err(NnCpuError::InvalidOpArity {
            code: op.code,
            expected: 1,
            actual: op.inputs.len(),
        });
    }
    let attrs = match &op.attrs {
        NnOpAttrs::Upsample2d { scale } if scale[0] > 0 && scale[1] > 0 => scale,
        _ => return Err(NnCpuError::ShapeMismatch { code: op.code }),
    };
    let input_desc = descriptor(model, op.inputs[0])?;
    let output_desc = descriptor(model, only_output_id(op)?)?;
    if input_desc.rank != 4 || output_desc.rank != 4 {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    let output_height = input_desc.shape[2]
        .checked_mul(attrs[0])
        .ok_or(NnCpuError::ShapeMismatch { code: op.code })?;
    let output_width = input_desc.shape[3]
        .checked_mul(attrs[1])
        .ok_or(NnCpuError::ShapeMismatch { code: op.code })?;
    if output_desc.shape
        != [
            input_desc.shape[0],
            input_desc.shape[1],
            output_height,
            output_width,
        ]
    {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    let input = tensor(tensors, op.inputs[0])?;
    let [batch, channels, input_height, input_width] = input_desc.shape.map(|value| value as usize);
    let output_height = output_height as usize;
    let output_width = output_width as usize;
    let mut output = vec![0.0; element_count(output_desc)?];
    for batch_index in 0..batch {
        for channel in 0..channels {
            for output_y in 0..output_height {
                for output_x in 0..output_width {
                    let input_y = output_y / attrs[0] as usize;
                    let input_x = output_x / attrs[1] as usize;
                    output[nchw_index(
                        batch_index,
                        channel,
                        output_y,
                        output_x,
                        channels,
                        output_height,
                        output_width,
                    )] = input[nchw_index(
                        batch_index,
                        channel,
                        input_y,
                        input_x,
                        channels,
                        input_height,
                        input_width,
                    )];
                }
            }
        }
    }
    Ok(output)
}

fn execute_batch_norm(
    model: &NnModelAsset,
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if op.inputs.len() != 5 {
        return Err(NnCpuError::InvalidOpArity {
            code: op.code,
            expected: 5,
            actual: op.inputs.len(),
        });
    }
    let epsilon = match &op.attrs {
        NnOpAttrs::BatchNorm { epsilon } if epsilon.is_finite() && *epsilon >= 0.0 => *epsilon,
        _ => return Err(NnCpuError::ShapeMismatch { code: op.code }),
    };
    let input_desc = descriptor(model, op.inputs[0])?;
    let output_desc = descriptor(model, only_output_id(op)?)?;
    if input_desc.rank != 4 || output_desc.shape != input_desc.shape {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    let channels = input_desc.shape[1] as usize;
    let spatial_elements = (input_desc.shape[2] as usize)
        .checked_mul(input_desc.shape[3] as usize)
        .ok_or(NnCpuError::ShapeMismatch { code: op.code })?;
    let input = tensor(tensors, op.inputs[0])?;
    let scale = tensor(tensors, op.inputs[1])?;
    let bias = tensor(tensors, op.inputs[2])?;
    let mean = tensor(tensors, op.inputs[3])?;
    let variance = tensor(tensors, op.inputs[4])?;
    if [scale, bias, mean, variance]
        .iter()
        .any(|values| values.len() != channels)
    {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }

    input
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let channel = (index / spatial_elements) % channels;
            let normalization = variance[channel] + epsilon;
            if normalization <= 0.0 || !normalization.is_finite() {
                return Err(NnCpuError::ShapeMismatch { code: op.code });
            }
            Ok(scale[channel] * (*value - mean[channel]) / normalization.sqrt() + bias[channel])
        })
        .collect()
}

fn execute_layer_norm(
    model: &NnModelAsset,
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if op.inputs.len() != 3 {
        return Err(NnCpuError::InvalidOpArity {
            code: op.code,
            expected: 3,
            actual: op.inputs.len(),
        });
    }
    let epsilon = match &op.attrs {
        NnOpAttrs::LayerNorm { epsilon } if epsilon.is_finite() && *epsilon >= 0.0 => *epsilon,
        _ => return Err(NnCpuError::ShapeMismatch { code: op.code }),
    };
    let input_desc = descriptor(model, op.inputs[0])?;
    let output_desc = descriptor(model, only_output_id(op)?)?;
    if output_desc.shape != input_desc.shape {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    let axis_size = input_desc.shape[3] as usize;
    let input = tensor(tensors, op.inputs[0])?;
    let scale = tensor(tensors, op.inputs[1])?;
    let bias = tensor(tensors, op.inputs[2])?;
    if axis_size == 0 || scale.len() != axis_size || bias.len() != axis_size {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    let mut output = Vec::with_capacity(input.len());
    for row in input.chunks_exact(axis_size) {
        let mean = row.iter().sum::<f32>() / axis_size as f32;
        let variance = row
            .iter()
            .map(|value| {
                let delta = *value - mean;
                delta * delta
            })
            .sum::<f32>()
            / axis_size as f32;
        let normalization = variance + epsilon;
        if normalization <= 0.0 || !normalization.is_finite() {
            return Err(NnCpuError::ShapeMismatch { code: op.code });
        }
        output.extend(row.iter().enumerate().map(|(index, value)| {
            scale[index] * (*value - mean) / normalization.sqrt() + bias[index]
        }));
    }
    if output.len() != input.len() {
        return Err(NnCpuError::ShapeMismatch { code: op.code });
    }
    Ok(output)
}

fn execute_reshape(
    op: &crate::NnOp,
    tensors: &BTreeMap<u16, Vec<f32>>,
) -> Result<Vec<f32>, NnCpuError> {
    if op.inputs.len() != 1 {
        return Err(NnCpuError::InvalidOpArity {
            code: op.code,
            expected: 1,
            actual: op.inputs.len(),
        });
    }
    Ok(tensor(tensors, op.inputs[0])?.clone())
}

fn tensor<'a>(tensors: &'a BTreeMap<u16, Vec<f32>>, id: u16) -> Result<&'a Vec<f32>, NnCpuError> {
    tensors.get(&id).ok_or(NnCpuError::MissingTensor(id))
}

fn descriptor<'a>(model: &'a NnModelAsset, id: u16) -> Result<&'a NnTensorDesc, NnCpuError> {
    model
        .tensors
        .get(usize::from(id))
        .ok_or(NnCpuError::MissingTensor(id))
}

fn only_output_id(op: &crate::NnOp) -> Result<u16, NnCpuError> {
    if op.outputs.len() == 1 {
        Ok(op.outputs[0])
    } else {
        Err(NnCpuError::InvalidOpArity {
            code: op.code,
            expected: 1,
            actual: op.outputs.len(),
        })
    }
}

fn matrix_shape(model: &NnModelAsset, id: u16) -> Result<(usize, usize), NnCpuError> {
    let descriptor = model
        .tensors
        .get(usize::from(id))
        .ok_or(NnCpuError::MissingTensor(id))?;
    if descriptor.rank != 2 {
        return Err(NnCpuError::ShapeMismatch {
            code: NnOpCode::Gemm,
        });
    }
    Ok((descriptor.shape[2] as usize, descriptor.shape[3] as usize))
}

fn element_count(descriptor: &NnTensorDesc) -> Result<usize, NnCpuError> {
    descriptor
        .element_count()
        .and_then(|count| usize::try_from(count).ok())
        .ok_or(NnCpuError::ShapeMismatch {
            code: NnOpCode::Gemm,
        })
}
