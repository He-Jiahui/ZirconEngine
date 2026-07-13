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
    let sky_t = clamp(direction.y * 0.5 + 0.5, 0.0, 1.0);
    let ground_t = clamp(direction.y + 1.0, 0.0, 1.0);
    let sky = mix(params.horizon_color.rgb, params.zenith_color.rgb, sky_t);
    let ground = mix(params.ground_color.rgb, params.horizon_color.rgb, ground_t);
    var color = select(ground, sky, direction.y >= 0.0);
    let sun_direction_length = length(params.sun_direction.xyz);
    if (params.sun_params.x > 0.0 && sun_direction_length > 0.000001) {
        let sun_direction = params.sun_direction.xyz / sun_direction_length;
        let angular_radius = clamp(params.sun_params.y, 0.0001, 1.5707963);
        let inner_cosine = cos(angular_radius * 0.72);
        let outer_cosine = cos(angular_radius);
        let sun_mask = smoothstep(outer_cosine, inner_cosine, dot(direction, sun_direction));
        color += params.sun_color.rgb * params.sun_params.x * sun_mask;
    }
    textureStore(output_cube, vec2<i32>(global_id.xy), i32(face), vec4<f32>(color, 1.0));
}
