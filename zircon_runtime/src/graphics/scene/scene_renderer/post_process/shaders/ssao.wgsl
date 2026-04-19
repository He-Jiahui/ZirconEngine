struct SsaoParams {
    viewport_and_flags: vec4<u32>,
    tuning: vec4<f32>,
};

@group(0) @binding(0) var depth_tex: texture_depth_2d;
@group(0) @binding(1) var normal_tex: texture_2d<f32>;
@group(0) @binding(2) var previous_ao_tex: texture_2d<f32>;
@group(0) @binding(3) var<uniform> params: SsaoParams;
@group(0) @binding(4) var ao_out: texture_storage_2d<rgba8unorm, write>;

fn load_depth(coords: vec2<i32>, size: vec2<i32>) -> f32 {
    let clamped = clamp(coords, vec2<i32>(0, 0), size - vec2<i32>(1, 1));
    return textureLoad(depth_tex, clamped, 0);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let viewport_size = params.viewport_and_flags.xy;
    if (invocation_id.x >= viewport_size.x || invocation_id.y >= viewport_size.y) {
        return;
    }

    let coord = vec2<i32>(invocation_id.xy);
    let size_i32 = vec2<i32>(viewport_size);
    let center_depth = load_depth(coord, size_i32);
    let encoded_normal = textureLoad(normal_tex, coord, 0).xyz;
    let normal = normalize(encoded_normal * 2.0 - vec3<f32>(1.0, 1.0, 1.0));

    var occlusion = 0.0;
    let offsets = array<vec2<i32>, 8>(
        vec2<i32>(1, 0),
        vec2<i32>(-1, 0),
        vec2<i32>(0, 1),
        vec2<i32>(0, -1),
        vec2<i32>(1, 1),
        vec2<i32>(-1, 1),
        vec2<i32>(1, -1),
        vec2<i32>(-1, -1),
    );

    for (var i = 0u; i < 8u; i = i + 1u) {
        let sample_depth = load_depth(coord + offsets[i], size_i32);
        let depth_delta = abs(sample_depth - center_depth);
        occlusion = occlusion + smoothstep(params.tuning.y, params.tuning.y * 12.0, depth_delta);
    }

    let curvature = clamp(1.0 - max(normal.z, 0.0), 0.0, 1.0);
    var ao = clamp(1.0 - (occlusion / 8.0) * params.tuning.x - curvature * 0.24, 0.1, 1.0);
    ao = clamp(ao * params.tuning.w, 0.1, 1.0);
    if (params.viewport_and_flags.z != 0u) {
        let previous = textureLoad(previous_ao_tex, coord, 0).r;
        ao = mix(ao, previous, params.tuning.z);
    }

    textureStore(ao_out, coord, vec4<f32>(ao, ao, ao, 1.0));
}
