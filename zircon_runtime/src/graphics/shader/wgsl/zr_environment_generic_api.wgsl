fn zr_environment_procedural_sky_color(direction: vec3<f32>) -> vec3<f32> {
    return zr_environment_procedural_sky_color_normalized(
        zr_environment_normalize_or_zero(direction),
    );
}

fn zr_environment_diffuse_color(normal_ws: vec3<f32>) -> vec3<f32> {
    return zr_environment_diffuse_color_normalized(
        zr_environment_normalize_or_zero(normal_ws),
    );
}

fn zr_environment_fix_source_cube_lookup(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    return zr_environment_fix_cube_lookup_for_face_size(
        direction,
        lod,
        scene.environment_sample_params.y,
    );
}

fn zr_environment_source_cube_color_at_lod(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    let mip_count = floor(log2(max(scene.environment_sample_params.y, 1.0))) + 1.0;
    let max_mip = mip_count - 1.0;
    let clamped_lod = clamp(lod, 0.0, max_mip);
    let rotated = zr_environment_rotated_direction(zr_environment_normalize_or_zero(direction));
    return textureSampleLevel(
        zr_environment_source_cube,
        zr_environment_sampler,
        zr_environment_fix_source_cube_lookup(rotated, clamped_lod),
        clamped_lod,
    ).rgb * max(scene.environment_params.y, 0.0);
}

fn zr_environment_specular_pmrem_color_at_lod(direction: vec3<f32>, lod: f32) -> vec3<f32> {
    let mip_count = max(scene.environment_sample_params.w, 1.0);
    let max_mip = mip_count - 1.0;
    let clamped_lod = clamp(lod, 0.0, max_mip);
    return zr_environment_specular_pmrem_color_at_clamped_lod_normalized(
        zr_environment_normalize_or_zero(direction),
        clamped_lod,
    );
}

fn zr_environment_env_brdf_approx(f0: vec3<f32>, roughness: f32, no_v: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * no_v)) * r.x + r.y;
    let ab = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    let f90 = clamp(50.0 * f0.g, 0.0, 1.0);
    // Keep the HDR split-sum result; only the F90 gate is saturated.
    return f0 * ab.x + vec3<f32>(f90) * ab.y;
}

fn zr_environment_sh9_eval(normal_ws: vec3<f32>) -> vec3<f32> {
    return zr_environment_sh9_eval_normalized(zr_environment_normalize_or_zero(normal_ws));
}

fn zr_environment_irradiance_cube_color(normal_ws: vec3<f32>) -> vec3<f32> {
    return zr_environment_irradiance_cube_color_normalized(
        zr_environment_normalize_or_zero(normal_ws),
    );
}

fn zr_environment_sky_color(direction: vec3<f32>) -> vec3<f32> {
    if (zr_environment_is_source_cubemap() || zr_environment_is_realtime_ibl()) {
        return zr_environment_source_cube_color_at_lod(direction, 0.0);
    }
    return zr_environment_procedural_sky_color(direction);
}
