use crate::NnOpCode;

const GEMM_SHADER: &str = r#"
struct GemmParams {
    m: u32,
    n: u32,
    k: u32,
    _reserved: u32,
    alpha: f32,
    beta: f32,
};

var<workgroup> tile_a: array<f32, 256>;
var<workgroup> tile_b: array<f32, 256>;

@group(1) @binding(0) var<uniform> params: GemmParams;
@group(1) @binding(1) var<storage, read> input_a: array<f32>;
@group(1) @binding(2) var<storage, read> input_b: array<f32>;
@group(1) @binding(3) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(16, 16, 1)
fn cs_main(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let row = workgroup_id.y * 16u + local_id.y;
    let column = workgroup_id.x * 16u + local_id.x;
    var accumulator = 0.0;
    let steps = (params.k + 15u) / 16u;
    for (var step = 0u; step < steps; step = step + 1u) {
        let a_column = step * 16u + local_id.x;
        let b_row = step * 16u + local_id.y;
        let local_index = local_id.y * 16u + local_id.x;
        tile_a[local_index] = select(0.0, input_a[row * params.k + a_column], row < params.m && a_column < params.k);
        tile_b[local_index] = select(0.0, input_b[b_row * params.n + column], b_row < params.k && column < params.n);
        workgroupBarrier();
        for (var index = 0u; index < 16u; index = index + 1u) {
            accumulator = accumulator + tile_a[local_id.y * 16u + index] * tile_b[index * 16u + local_id.x];
        }
        workgroupBarrier();
    }
    if (row < params.m && column < params.n) {
        output[row * params.n + column] = params.alpha * accumulator;
    }
}
"#;

const UNARY_ELEMENTWISE_SHADER: &str = r#"
struct ElementwiseParams { count: u32 };
@group(1) @binding(0) var<uniform> params: ElementwiseParams;
@group(1) @binding(1) var<storage, read> input: array<f32>;
@group(1) @binding(2) var<storage, read_write> output: array<f32>;

fn apply(value: f32) -> f32 {
    //ZR_NN_OP_BODY
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = invocation_id.x;
    if (index >= params.count) { return; }
    output[index] = apply(input[index]);
}
"#;

const BINARY_ELEMENTWISE_SHADER: &str = r#"
struct ElementwiseParams { count: u32 };
@group(1) @binding(0) var<uniform> params: ElementwiseParams;
@group(1) @binding(1) var<storage, read> lhs: array<f32>;
@group(1) @binding(2) var<storage, read> rhs: array<f32>;
@group(1) @binding(3) var<storage, read_write> output: array<f32>;

fn apply(lhs_value: f32, rhs_value: f32) -> f32 {
    //ZR_NN_OP_BODY
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = invocation_id.x;
    if (index >= params.count) { return; }
    output[index] = apply(lhs[index], rhs[index]);
}
"#;

const CONV2D_SHADER: &str = r#"
struct ConvParams {
    input_shape: vec4<u32>,
    output_shape: vec4<u32>,
    kernel_and_stride: vec4<u32>,
    padding_and_dilation: vec4<u32>,
    groups: u32,
};

@group(1) @binding(0) var<uniform> params: ConvParams;
@group(1) @binding(1) var<storage, read> input: array<f32>;
@group(1) @binding(2) var<storage, read> weights: array<f32>;
@group(1) @binding(3) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let output_x = invocation_id.x;
    let output_y = invocation_id.y;
    let output_channel_count = params.output_shape.y;
    let batch = invocation_id.z / output_channel_count;
    let output_channel = invocation_id.z % output_channel_count;
    if (output_x >= params.output_shape.w || output_y >= params.output_shape.z || batch >= params.output_shape.x) {
        return;
    }
    let input_channels_per_group = params.input_shape.y / params.groups;
    let output_channels_per_group = output_channel_count / params.groups;
    let group = output_channel / output_channels_per_group;
    var value = 0.0;
    for (var input_channel = 0u; input_channel < input_channels_per_group; input_channel = input_channel + 1u) {
        for (var kernel_y = 0u; kernel_y < params.kernel_and_stride.x; kernel_y = kernel_y + 1u) {
            for (var kernel_x = 0u; kernel_x < params.kernel_and_stride.y; kernel_x = kernel_x + 1u) {
                let input_y = i32(output_y * params.kernel_and_stride.z + kernel_y * params.padding_and_dilation.z) - i32(params.padding_and_dilation.x);
                let input_x = i32(output_x * params.kernel_and_stride.w + kernel_x * params.padding_and_dilation.w) - i32(params.padding_and_dilation.y);
                if (input_y < 0 || input_x < 0 || input_y >= i32(params.input_shape.z) || input_x >= i32(params.input_shape.w)) {
                    continue;
                }
                let source_channel = group * input_channels_per_group + input_channel;
                let input_index = ((batch * params.input_shape.y + source_channel) * params.input_shape.z + u32(input_y)) * params.input_shape.w + u32(input_x);
                let weight_index = ((output_channel * input_channels_per_group + input_channel) * params.kernel_and_stride.x + kernel_y) * params.kernel_and_stride.y + kernel_x;
                value = value + input[input_index] * weights[weight_index];
            }
        }
    }
    let output_index = ((batch * output_channel_count + output_channel) * params.output_shape.z + output_y) * params.output_shape.w + output_x;
    output[output_index] = value;
}
"#;

const POOL2D_SHADER: &str = r#"
struct PoolParams {
    input_shape: vec4<u32>,
    output_shape: vec4<u32>,
    kernel_and_stride: vec4<u32>,
    padding: vec4<u32>,
};

@group(1) @binding(0) var<uniform> params: PoolParams;
@group(1) @binding(1) var<storage, read> input: array<f32>;
@group(1) @binding(2) var<storage, read_write> output: array<f32>;

fn pool_initial_value() -> f32 {
    //ZR_NN_POOL_INITIAL
}

fn pool_combine(value: f32, sample: f32) -> f32 {
    //ZR_NN_POOL_COMBINE
}

fn pool_finish(value: f32, count: u32) -> f32 {
    //ZR_NN_POOL_FINISH
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let output_x = invocation_id.x;
    let output_y = invocation_id.y;
    let channels = params.output_shape.y;
    let batch = invocation_id.z / channels;
    let channel = invocation_id.z % channels;
    if (output_x >= params.output_shape.w || output_y >= params.output_shape.z || batch >= params.output_shape.x) {
        return;
    }

    var value = pool_initial_value();
    var count = 0u;
    for (var kernel_y = 0u; kernel_y < params.kernel_and_stride.x; kernel_y = kernel_y + 1u) {
        for (var kernel_x = 0u; kernel_x < params.kernel_and_stride.y; kernel_x = kernel_x + 1u) {
            let input_y = i32(output_y * params.kernel_and_stride.z + kernel_y) - i32(params.padding.x);
            let input_x = i32(output_x * params.kernel_and_stride.w + kernel_x) - i32(params.padding.y);
            if (input_y < 0 || input_x < 0 || input_y >= i32(params.input_shape.z) || input_x >= i32(params.input_shape.w)) {
                continue;
            }
            let input_index = ((batch * params.input_shape.y + channel) * params.input_shape.z + u32(input_y)) * params.input_shape.w + u32(input_x);
            value = pool_combine(value, input[input_index]);
            count = count + 1u;
        }
    }
    if (count == 0u) {
        return;
    }
    let output_index = ((batch * channels + channel) * params.output_shape.z + output_y) * params.output_shape.w + output_x;
    output[output_index] = pool_finish(value, count);
}
"#;

const UPSAMPLE2D_SHADER: &str = r#"
struct UpsampleParams {
    input_shape: vec4<u32>,
    output_shape: vec4<u32>,
    scale: vec4<u32>,
};

@group(1) @binding(0) var<uniform> params: UpsampleParams;
@group(1) @binding(1) var<storage, read> input: array<f32>;
@group(1) @binding(2) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let output_x = invocation_id.x;
    let output_y = invocation_id.y;
    let channels = params.output_shape.y;
    let batch = invocation_id.z / channels;
    let channel = invocation_id.z % channels;
    if (output_x >= params.output_shape.w || output_y >= params.output_shape.z || batch >= params.output_shape.x) {
        return;
    }
    let input_x = output_x / params.scale.y;
    let input_y = output_y / params.scale.x;
    let input_index = ((batch * params.input_shape.y + channel) * params.input_shape.z + input_y) * params.input_shape.w + input_x;
    let output_index = ((batch * channels + channel) * params.output_shape.z + output_y) * params.output_shape.w + output_x;
    output[output_index] = input[input_index];
}
"#;

const BATCH_NORM_SHADER: &str = r#"
struct BatchNormParams {
    input_shape: vec4<u32>,
    epsilon: f32,
};

@group(1) @binding(0) var<uniform> params: BatchNormParams;
@group(1) @binding(1) var<storage, read> input: array<f32>;
@group(1) @binding(2) var<storage, read> scale: array<f32>;
@group(1) @binding(3) var<storage, read> bias: array<f32>;
@group(1) @binding(4) var<storage, read> mean: array<f32>;
@group(1) @binding(5) var<storage, read> variance: array<f32>;
@group(1) @binding(6) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = invocation_id.x;
    let element_count = params.input_shape.x * params.input_shape.y * params.input_shape.z * params.input_shape.w;
    if (index >= element_count) {
        return;
    }
    let spatial_elements = params.input_shape.z * params.input_shape.w;
    let channel = (index / spatial_elements) % params.input_shape.y;
    output[index] = scale[channel] * (input[index] - mean[channel])
        / sqrt(variance[channel] + params.epsilon) + bias[channel];
}
"#;

const LAYER_NORM_SHADER: &str = r#"
struct LayerNormParams {
    input_shape: vec4<u32>,
    epsilon: f32,
};

@group(1) @binding(0) var<uniform> params: LayerNormParams;
@group(1) @binding(1) var<storage, read> input: array<f32>;
@group(1) @binding(2) var<storage, read> scale: array<f32>;
@group(1) @binding(3) var<storage, read> bias: array<f32>;
@group(1) @binding(4) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let axis_size = params.input_shape.w;
    let element_count = params.input_shape.x * params.input_shape.y * params.input_shape.z * axis_size;
    let row = invocation_id.x;
    let row_count = element_count / axis_size;
    if (row >= row_count) {
        return;
    }
    let offset = row * axis_size;
    var mean = 0.0;
    for (var index = 0u; index < axis_size; index = index + 1u) {
        mean = mean + input[offset + index];
    }
    mean = mean / f32(axis_size);
    var variance = 0.0;
    for (var index = 0u; index < axis_size; index = index + 1u) {
        let delta = input[offset + index] - mean;
        variance = variance + delta * delta;
    }
    let inverse_stddev = inverseSqrt(variance / f32(axis_size) + params.epsilon);
    for (var index = 0u; index < axis_size; index = index + 1u) {
        output[offset + index] = (input[offset + index] - mean) * inverse_stddev * scale[index] + bias[index];
    }
}
"#;

pub(super) fn shader_for(code: NnOpCode) -> Option<String> {
    match code {
        NnOpCode::Gemm => Some(GEMM_SHADER.to_owned()),
        NnOpCode::Conv2d | NnOpCode::DepthwiseConv2d => Some(CONV2D_SHADER.to_owned()),
        NnOpCode::Relu => Some(unary_shader("return max(value, 0.0);")),
        NnOpCode::Sigmoid => Some(unary_shader("return 1.0 / (1.0 + exp(-value));")),
        NnOpCode::Tanh => Some(unary_shader("return tanh(value);")),
        NnOpCode::Silu => Some(unary_shader("return value / (1.0 + exp(-value));")),
        NnOpCode::Add => Some(binary_shader("return lhs_value + rhs_value;")),
        NnOpCode::Mul => Some(binary_shader("return lhs_value * rhs_value;")),
        NnOpCode::Sub => Some(binary_shader("return lhs_value - rhs_value;")),
        NnOpCode::Div => Some(binary_shader("return lhs_value / rhs_value;")),
        NnOpCode::MaxPool2d => Some(pool_shader(
            "return -3.402823e38;",
            "return max(value, sample);",
            "return value;",
        )),
        NnOpCode::AvgPool2d => Some(pool_shader(
            "return 0.0;",
            "return value + sample;",
            "return value / f32(count);",
        )),
        NnOpCode::Upsample2d => Some(UPSAMPLE2D_SHADER.to_owned()),
        NnOpCode::BatchNorm => Some(BATCH_NORM_SHADER.to_owned()),
        NnOpCode::LayerNorm => Some(LAYER_NORM_SHADER.to_owned()),
        _ => None,
    }
}

fn unary_shader(operation_body: &str) -> String {
    UNARY_ELEMENTWISE_SHADER.replace("//ZR_NN_OP_BODY", operation_body)
}

fn binary_shader(operation_body: &str) -> String {
    BINARY_ELEMENTWISE_SHADER.replace("//ZR_NN_OP_BODY", operation_body)
}

fn pool_shader(initial: &str, combine: &str, finish: &str) -> String {
    POOL2D_SHADER
        .replace("//ZR_NN_POOL_INITIAL", initial)
        .replace("//ZR_NN_POOL_COMBINE", combine)
        .replace("//ZR_NN_POOL_FINISH", finish)
}
