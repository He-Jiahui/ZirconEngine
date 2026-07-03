fn vampire_default_normal(input: ZrVertexOutput, uv: vec2<f32>) -> vec3<f32> {
    let sampled = zr_sample_normal(uv).xyz * 2.0 - vec3<f32>(1.0);
    let tangent = zr_normalize_or_zero(input.tangent_ws);
    let bitangent = zr_normalize_or_zero(cross(input.normal_ws, tangent) * input.tangent_handedness);
    return zr_normalize_or_zero(
        input.normal_ws + tangent * sampled.x * 0.18 + bitangent * sampled.y * 0.18,
    );
}

fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let uv = input.uv0;
    let base_sample = zr_sample_base_color(uv);
    let metallic_roughness = zr_sample_metallic_roughness(uv).rgb;
    let occlusion_sample = zr_sample_occlusion(uv).r;
    let emissive_sample = zr_sample_emissive(uv).rgb;

    var surface = zr_surface_from_base_color(
        zr_mat_base_color() * base_sample * input.tint * input.color,
    );
    surface.normal_ws = vampire_default_normal(input, uv);
    surface.metallic = clamp(zr_mat_metallic() * metallic_roughness.b, 0.0, 1.0);
    surface.roughness = clamp(zr_mat_roughness() * metallic_roughness.g, 0.04, 1.0);
    surface.occlusion = clamp(occlusion_sample, 0.0, 1.0);
    surface.emissive = max(zr_mat_emissive(), vec3<f32>(0.0)) * emissive_sample;
    surface.shading_model_id = 2u;
    return surface;
}
