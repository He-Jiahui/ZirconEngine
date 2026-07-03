fn vampire_effect_noise(value: vec2<f32>) -> f32 {
    return fract(sin(dot(value, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn vampire_effect_color(base_color: vec4<f32>, input: ZrVertexOutput) -> vec4<f32> {
    let swirl = sin(input.position_ws.x * 2.7 + input.position_ws.y * 4.1 + input.uv0.y * 8.0);
    let cell = vampire_effect_noise(floor(input.position_ws.xz * 1.8 + input.uv0 * 4.0));
    let glow = smoothstep(-0.2, 1.0, swirl * 0.5 + cell);
    let ember = mix(vec3<f32>(0.28, 0.04, 0.02), vec3<f32>(1.35, 0.42, 0.06), glow);
    return vec4<f32>(base_color.rgb * ember, base_color.a);
}

fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let uv = input.uv0;
    let base_sample = zr_sample_base_color(uv);
    let metallic_roughness = zr_sample_metallic_roughness(uv).rgb;
    let occlusion_sample = zr_sample_occlusion(uv).r;
    let emissive_sample = zr_sample_emissive(uv).rgb;

    var base_color = zr_mat_base_color() * base_sample * input.tint * input.color;
    base_color = vampire_effect_color(base_color, input);

    var surface = zr_surface_from_base_color(base_color);
    surface.normal_ws = zr_normalize_or_zero(input.normal_ws);
    surface.metallic = clamp(zr_mat_metallic() * metallic_roughness.b, 0.0, 1.0);
    surface.roughness = clamp(zr_mat_roughness() * metallic_roughness.g, 0.08, 1.0);
    surface.occlusion = clamp(occlusion_sample, 0.0, 1.0);
    surface.emissive = max(zr_mat_emissive(), vec3<f32>(0.0)) * emissive_sample + base_color.rgb * 0.18;
    surface.shading_model_id = 2u;
    surface.custom0 = vec4<f32>(f32(VAMPIRE_EFFECT_SHADER_VARIANT_MARKER), 0.0, 0.0, 0.0);
    return surface;
}
