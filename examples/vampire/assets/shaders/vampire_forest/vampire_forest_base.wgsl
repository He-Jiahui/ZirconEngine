fn vampire_forest_noise(value: vec2<f32>) -> f32 {
    return fract(sin(dot(value, vec2<f32>(91.7, 13.3))) * 43758.5453);
}

fn vampire_forest_detail_color(base_color: vec4<f32>, input: ZrVertexOutput) -> vec4<f32> {
    let canopy = vampire_forest_noise(floor(input.position_ws.xz * 0.33));
    let root_band = 1.0 - smoothstep(0.10, 0.22, abs(fract(input.position_ws.x * 0.16 + input.position_ws.z * 0.09) - 0.5));
    let moss = smoothstep(0.15, 0.72, fract(input.uv0.x * 1.7 + input.uv0.y * 2.3));
    var detail = mix(base_color.rgb, vec3<f32>(0.24, 0.44, 0.16), 0.42);
    detail *= mix(0.86, 1.20, canopy);
    detail = mix(detail, vec3<f32>(0.16, 0.25, 0.095), root_band * 0.18);
    detail += vec3<f32>(0.035, 0.11, 0.030) * moss;
    return vec4<f32>(clamp(detail, vec3<f32>(0.0), vec3<f32>(1.25)), base_color.a);
}

fn vampire_forest_normal(input: ZrVertexOutput, uv: vec2<f32>) -> vec3<f32> {
    let sampled = zr_sample_normal(uv).xyz * 2.0 - vec3<f32>(1.0);
    let tangent = zr_normalize_or_zero(input.tangent_ws);
    let bitangent = zr_normalize_or_zero(cross(input.normal_ws, tangent) * input.tangent_handedness);
    return zr_normalize_or_zero(input.normal_ws + tangent * sampled.x * 0.09 + bitangent * sampled.y * 0.09);
}

fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let uv = input.uv0;
    let base_sample = zr_sample_base_color(uv);
    let metallic_roughness = zr_sample_metallic_roughness(uv).rgb;
    let occlusion_sample = zr_sample_occlusion(uv).r;
    let emissive_sample = zr_sample_emissive(uv).rgb;

    var base_color = zr_mat_base_color() * base_sample * input.tint * input.color;
    base_color = vampire_forest_detail_color(base_color, input);

    var surface = zr_surface_from_base_color(base_color);
    surface.normal_ws = vampire_forest_normal(input, uv);
    surface.metallic = clamp(zr_mat_metallic() * metallic_roughness.b, 0.0, 1.0);
    surface.roughness = clamp(zr_mat_roughness() * metallic_roughness.g, 0.12, 1.0);
    surface.occlusion = clamp(occlusion_sample, 0.0, 1.0);
    surface.emissive = max(zr_mat_emissive(), vec3<f32>(0.0)) * emissive_sample;
    surface.shading_model_id = 2u;
    surface.custom0 = vec4<f32>(f32(VAMPIRE_FOREST_SHADER_VARIANT_MARKER), 0.0, 0.0, 0.0);
    return surface;
}
