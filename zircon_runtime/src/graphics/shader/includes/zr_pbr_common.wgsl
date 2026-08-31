const ZR_PBR_COMMON_MAX_NORMALIZABLE_LENGTH: f32 = 3.4e38;

fn zr_pbr_base_color(base_color: vec3<f32>) -> vec3<f32> {
    return clamp(base_color, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn zr_surface_metallic_diffuse_energy_scale(metallic: f32) -> f32 {
    return 1.0 - clamp(metallic, 0.0, 1.0);
}

fn zr_pbr_fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    let grazing = pow(1.0 - clamp(cos_theta, 0.0, 1.0), 5.0);
    return f0 + (vec3<f32>(1.0) - f0) * grazing;
}

// Transmission retains the Fresnel complement because it models the
// view-dependent fraction that exits through the opposite interface.
fn zr_pbr_transmission_energy_scale(
    cos_theta: f32,
    f0: vec3<f32>,
    metallic: f32,
) -> vec3<f32> {
    let fresnel = zr_pbr_fresnel_schlick(cos_theta, f0);
    return (vec3<f32>(1.0) - fresnel)
        * zr_surface_metallic_diffuse_energy_scale(metallic);
}

fn zr_pbr_material_f0(
    dielectric_f0: vec3<f32>,
    base_color: vec3<f32>,
    metallic: f32,
) -> vec3<f32> {
    return mix(
        clamp(dielectric_f0, vec3<f32>(0.0), vec3<f32>(1.0)),
        zr_pbr_base_color(base_color),
        clamp(metallic, 0.0, 1.0),
    );
}

fn zr_pbr_common_normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    let value_length = length(value);
    let is_finite_length = value_length > 0.000001
        && value_length < ZR_PBR_COMMON_MAX_NORMALIZABLE_LENGTH;
    return select(
        vec3<f32>(0.0),
        value / max(value_length, 0.000001),
        is_finite_length,
    );
}

fn zr_pbr_view_direction_ws(world_position: vec3<f32>) -> vec3<f32> {
    let camera_direction_weight = clamp(scene.camera_view_direction.w, 0.0, 1.0);
    if (camera_direction_weight <= 0.0) {
        return zr_pbr_common_normalize_or_zero(
            scene.camera_world_position.xyz - world_position,
        );
    }
    if (camera_direction_weight >= 1.0) {
        return zr_pbr_common_normalize_or_zero(scene.camera_view_direction.xyz);
    }
    let perspective_view_dir = zr_pbr_common_normalize_or_zero(
        scene.camera_world_position.xyz - world_position,
    );
    return zr_pbr_common_normalize_or_zero(mix(
        perspective_view_dir,
        scene.camera_view_direction.xyz,
        camera_direction_weight,
    ));
}
