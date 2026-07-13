struct PlanarFilterParams {
    input_dimensions: vec2<u32>,
    output_dimensions: vec2<u32>,
    kernel: vec4<u32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var<uniform> params: PlanarFilterParams;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    if (any(invocation_id.xy >= params.output_dimensions)) {
        return;
    }

    let scale = max(params.input_dimensions / params.output_dimensions, vec2<u32>(1u));
    let center = min(
        invocation_id.xy * scale + scale / vec2<u32>(2u),
        params.input_dimensions - vec2<u32>(1u),
    );
    let radius = i32(params.kernel.x);
    var sum = vec4<f32>(0.0);
    var weight_sum = 0.0;
    for (var y = -2; y <= 2; y = y + 1) {
        for (var x = -2; x <= 2; x = x + 1) {
            if (abs(x) > radius || abs(y) > radius) {
                continue;
            }
            let coordinate = clamp(
                vec2<i32>(center) + vec2<i32>(x, y),
                vec2<i32>(0),
                vec2<i32>(params.input_dimensions) - vec2<i32>(1),
            );
            let weight = f32((radius + 1 - abs(x)) * (radius + 1 - abs(y)));
            sum += textureLoad(source_texture, coordinate, 0) * weight;
            weight_sum += weight;
        }
    }
    textureStore(output_texture, vec2<i32>(invocation_id.xy), sum / max(weight_sum, 1.0));
}
