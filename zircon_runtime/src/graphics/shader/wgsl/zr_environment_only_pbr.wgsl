fn zr_environment_pbr_components(
    _world_position: vec3<f32>,
    normal_ws: vec3<f32>,
    view_dir_ws: vec3<f32>,
    roughness: f32,
    metallic: f32,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    occlusion: f32,
    is_standard_pbr: bool,
) -> ZrEnvironmentPbrComponents {
    if (!is_standard_pbr) {
        return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    let environment_intensity = max(scene.environment_params.y, 0.0);
    let has_global_environment = zr_environment_is_enabled()
        && environment_intensity > 0.0;
    if (!has_global_environment) {
        return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    let clamped_occlusion = clamp(occlusion, 0.0, 1.0);
    if (clamped_occlusion <= 0.0) {
        return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    let normal = zr_environment_normalize_or_zero(normal_ws);
    let view_dir = zr_environment_normalize_or_zero(view_dir_ws);
    if (all(normal == vec3<f32>(0.0)) || all(view_dir == vec3<f32>(0.0))) {
        return ZrEnvironmentPbrComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    let clamped_metallic = clamp(metallic, 0.0, 1.0);
    let clamped_roughness = clamp(roughness, 0.0, 1.0);
    let reflection = zr_environment_sky_reflection_color(
        reflect(-view_dir, normal),
        clamped_roughness,
    );
    return zr_environment_pbr_components_from_reflection(
        normal,
        view_dir,
        clamped_roughness,
        clamped_metallic,
        clamped_occlusion,
        diffuse_color,
        base_color,
        has_global_environment,
        reflection,
    );
}

fn zr_environment_pbr_indirect(
    world_position: vec3<f32>,
    normal_ws: vec3<f32>,
    view_dir_ws: vec3<f32>,
    roughness: f32,
    metallic: f32,
    diffuse_color: vec3<f32>,
    base_color: vec3<f32>,
    occlusion: f32,
    is_standard_pbr: bool,
) -> vec3<f32> {
    let components = zr_environment_pbr_components(
        world_position,
        normal_ws,
        view_dir_ws,
        roughness,
        metallic,
        diffuse_color,
        base_color,
        occlusion,
        is_standard_pbr,
    );
    return components.diffuse + components.specular;
}
