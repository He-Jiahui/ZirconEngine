struct CaptureParams {
    horizon_color: vec4<f32>,
    zenith_color: vec4<f32>,
    ground_color: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    sun_params: vec4<f32>,
    face_params: vec4<u32>,
};

@group(0) @binding(0) var<uniform> params: CaptureParams;
@group(0) @binding(1) var output_cube: texture_storage_2d_array<rgba16float, write>;

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

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.face_params.x || global_id.y >= params.face_params.x) {
        return;
    }
    let face = params.face_params.y + global_id.z;
    if (face >= 6u) {
        return;
    }
    let uv = ((vec2<f32>(global_id.xy) + vec2<f32>(0.5)) / f32(params.face_params.x)) * 2.0
        - vec2<f32>(1.0);
    let direction = cube_face_direction(face, uv);
    let color = zr_procedural_sky_radiance(
        direction,
        params.horizon_color.rgb,
        params.zenith_color.rgb,
        params.ground_color.rgb,
        params.sun_direction,
        params.sun_color.rgb,
        params.sun_params,
    );
    textureStore(output_cube, vec2<i32>(global_id.xy), i32(face), vec4<f32>(color, 1.0));
}
