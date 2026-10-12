struct ComputeBindingProbeParams {
    element_count: u32,
    scale: f32,
    _padding: vec2<u32>,
}

@group(0) @binding(0)
var<uniform> params: ComputeBindingProbeParams;

@group(0) @binding(1)
var<storage, read> input_values: array<f32>;

@group(0) @binding(2)
var<storage, read_write> output_values: array<f32>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = invocation_id.x;
    let available = min(arrayLength(&input_values), arrayLength(&output_values));
    if index >= params.element_count || index >= available {
        return;
    }
    output_values[index] = input_values[index] * params.scale;
}
