struct DownsampleParams {
    source_face_size: u32,
    destination_face_size: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> params: DownsampleParams;
@group(0) @binding(1) var source_cube: texture_cube<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(0) @binding(3) var output_cube: texture_storage_2d_array<rgba16float, write>;

fn cube_face_direction(face: u32, uv: vec2<f32>) -> vec3<f32> {
    switch face {
        case 0u: { return normalize(vec3<f32>(1.0, -uv.y, -uv.x)); }
        case 1u: { return normalize(vec3<f32>(-1.0, -uv.y, uv.x)); }
        case 2u: { return normalize(vec3<f32>(uv.x, 1.0, uv.y)); }
        case 3u: { return normalize(vec3<f32>(uv.x, -1.0, -uv.y)); }
        case 4u: { return normalize(vec3<f32>(uv.x, -uv.y, 1.0)); }
        default: { return normalize(vec3<f32>(-uv.x, -uv.y, -1.0)); }
    }
}

fn source_texel_direction(face: u32, pixel: vec2<f32>) -> vec3<f32> {
    let uv = ((pixel + vec2<f32>(0.5)) / f32(params.source_face_size)) * 2.0
        - vec2<f32>(1.0);
    return cube_face_direction(face, uv);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (
        global_id.x >= params.destination_face_size
        || global_id.y >= params.destination_face_size
        || global_id.z >= 6u
    ) {
        return;
    }
    let source_origin = vec2<f32>(global_id.xy * 2u);
    var color = vec3<f32>(0.0);
    color += textureSampleLevel(
        source_cube, source_sampler, source_texel_direction(global_id.z, source_origin), 0.0
    ).rgb;
    color += textureSampleLevel(
        source_cube, source_sampler, source_texel_direction(global_id.z, source_origin + vec2<f32>(1.0, 0.0)), 0.0
    ).rgb;
    color += textureSampleLevel(
        source_cube, source_sampler, source_texel_direction(global_id.z, source_origin + vec2<f32>(0.0, 1.0)), 0.0
    ).rgb;
    color += textureSampleLevel(
        source_cube, source_sampler, source_texel_direction(global_id.z, source_origin + vec2<f32>(1.0, 1.0)), 0.0
    ).rgb;
    textureStore(
        output_cube,
        vec2<i32>(global_id.xy),
        i32(global_id.z),
        vec4<f32>(color * 0.25, 1.0),
    );
}
