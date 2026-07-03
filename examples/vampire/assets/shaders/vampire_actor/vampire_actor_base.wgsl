const VAMPIRE_ACTOR_EPSILON: f32 = 0.000001;

fn vampire_actor_noise(value: vec2<f32>) -> f32 {
    return fract(sin(dot(value, vec2<f32>(43.17, 91.73))) * 43758.5453);
}

fn vampire_actor_detail_color(base_color: vec4<f32>, input: ZrVertexOutput) -> vec4<f32> {
    let fabric = 0.5 + 0.5 * sin(dot(input.uv0, vec2<f32>(43.0, 71.0)) + input.position_ws.y * 11.0);
    let vertical_band = 1.0 - smoothstep(0.055, 0.095, abs(fract(input.uv0.x * 5.0 + input.position_ws.y * 0.65) - 0.5));
    let edge = pow(1.0 - abs(input.normal_ws.y), 2.2);
    var detail = base_color.rgb * mix(0.76, 1.18, fabric);
    detail = mix(detail, detail * vec3<f32>(0.52, 0.18, 0.18), vertical_band * 0.24);
    detail += vec3<f32>(0.05, 0.075, 0.13) * edge;
    return vec4<f32>(clamp(detail, vec3<f32>(0.0), vec3<f32>(1.45)), base_color.a);
}

fn vampire_actor_normal(input: ZrVertexOutput, uv: vec2<f32>) -> vec3<f32> {
    let sampled = zr_sample_normal(uv).xyz * 2.0 - vec3<f32>(1.0);
    let tangent = zr_normalize_or_zero(input.tangent_ws);
    let bitangent = zr_normalize_or_zero(cross(input.normal_ws, tangent) * input.tangent_handedness);
    let weave = vampire_actor_noise(floor(input.uv0 * 24.0)) * 0.04;
    return zr_normalize_or_zero(
        input.normal_ws + tangent * sampled.x * (0.12 + weave) + bitangent * sampled.y * 0.12,
    );
}

fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let uv = input.uv0;
    let base_sample = zr_sample_base_color(uv);
    let metallic_roughness = zr_sample_metallic_roughness(uv).rgb;
    let occlusion_sample = zr_sample_occlusion(uv).r;
    let emissive_sample = zr_sample_emissive(uv).rgb;

    var base_color = zr_mat_base_color() * base_sample * input.tint * input.color;
    base_color = vampire_actor_detail_color(base_color, input);

    var surface = zr_surface_from_base_color(base_color);
    surface.normal_ws = vampire_actor_normal(input, uv);
    surface.metallic = clamp(zr_mat_metallic() * metallic_roughness.b, 0.0, 1.0);
    surface.roughness = clamp(zr_mat_roughness() * metallic_roughness.g, 0.05, 1.0);
    surface.occlusion = clamp(occlusion_sample, 0.0, 1.0);
    surface.emissive = max(zr_mat_emissive(), vec3<f32>(0.0)) * emissive_sample;
    surface.shading_model_id = 2u;
    surface.custom0 = vec4<f32>(VAMPIRE_ACTOR_EPSILON, f32(VAMPIRE_ACTOR_SHADER_VARIANT_MARKER), 0.0, 0.0);
    return surface;
}
