@group(1) @binding(31) var zr_transmission_scene_color: texture_2d<f32>;
@group(1) @binding(32) var zr_transmission_scene_color_sampler: sampler;

fn zr_pbr_rotated_tangent(
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
    rotation: f32,
) -> vec3<f32> {
    return zr_normalize_or_zero(tangent * cos(rotation) + bitangent * sin(rotation));
}

fn zr_aniso_ggx(
    normal: vec3<f32>,
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    perceptual_roughness: f32,
    anisotropy_strength: f32,
    anisotropy_rotation: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    let half_dir = zr_normalize_or_zero(view_dir + light_dir);
    let rotated_tangent = zr_pbr_rotated_tangent(tangent, bitangent, anisotropy_rotation);
    let rotated_bitangent = zr_normalize_or_zero(cross(normal, rotated_tangent));
    let strength = clamp(anisotropy_strength, 0.0, 0.99);
    let base_alpha = max(perceptual_roughness * perceptual_roughness, 0.002);
    let alpha_t = max(base_alpha * (1.0 + strength), 0.002);
    let alpha_b = max(base_alpha * (1.0 - strength), 0.002);
    let to_h = dot(rotated_tangent, half_dir);
    let bo_h = dot(rotated_bitangent, half_dir);
    let no_h = max(dot(normal, half_dir), 0.0);
    let no_v = max(dot(normal, view_dir), ZR_PBR_EXTRAS_EPSILON);
    let no_l = max(dot(normal, light_dir), ZR_PBR_EXTRAS_EPSILON);
    let vo_h = max(dot(view_dir, half_dir), 0.0);
    let denominator = to_h * to_h / (alpha_t * alpha_t)
        + bo_h * bo_h / (alpha_b * alpha_b)
        + no_h * no_h;
    let distribution = 1.0 / max(
        ZR_PBR_EXTRAS_PI * alpha_t * alpha_b * denominator * denominator,
        ZR_PBR_EXTRAS_EPSILON,
    );
    let visibility = zr_pbr_smith_visibility(no_v, no_l, sqrt(alpha_t * alpha_b));
    return zr_pbr_fresnel_schlick(vo_h, f0) * distribution * visibility;
}

fn zr_clearcoat_lobe(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    perceptual_roughness: f32,
) -> vec3<f32> {
    return zr_pbr_isotropic_ggx(
        normal,
        view_dir,
        light_dir,
        perceptual_roughness,
        vec3<f32>(0.04),
    );
}

fn zr_transmission_btdf(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    base_color: vec3<f32>,
    diffuse_transmission: f32,
    ior: f32,
) -> vec3<f32> {
    let back_lambert = max(dot(-normal, light_dir), 0.0);
    let dielectric_f0 = pow((max(ior, 1.0) - 1.0) / (max(ior, 1.0) + 1.0), 2.0);
    let fresnel = zr_pbr_fresnel_schlick(
        max(dot(normal, view_dir), 0.0),
        vec3<f32>(dielectric_f0),
    );
    return base_color
        * (vec3<f32>(1.0) - fresnel)
        * back_lambert
        * clamp(diffuse_transmission, 0.0, 1.0)
        / ZR_PBR_EXTRAS_PI;
}

fn zr_pbr_clearcoat_base_energy_scale(surface: ZrSurfaceOutput, view_dir: vec3<f32>) -> vec3<f32> {
    if (!ZR_FEATURE_PBR_CLEARCOAT || surface.clearcoat <= 0.0) {
        return vec3<f32>(1.0);
    }
    let coat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);
    let normalized_view_dir = zr_normalize_or_zero(view_dir);
    return zr_pbr_clearcoat_base_energy_scale_normalized(
        surface,
        coat_normal,
        normalized_view_dir,
    );
}

fn zr_pbr_clearcoat_base_energy_scale_normalized(
    surface: ZrSurfaceOutput,
    coat_normal: vec3<f32>,
    normalized_view_dir: vec3<f32>,
) -> vec3<f32> {
    if (!ZR_FEATURE_PBR_CLEARCOAT || surface.clearcoat <= 0.0) {
        return vec3<f32>(1.0);
    }
    if (all(coat_normal == vec3<f32>(0.0)) || all(normalized_view_dir == vec3<f32>(0.0))) {
        return vec3<f32>(1.0);
    }
    let no_v = max(dot(coat_normal, normalized_view_dir), 0.0);
    let coat_fresnel = zr_pbr_fresnel_schlick(no_v, vec3<f32>(0.04));
    return vec3<f32>(1.0) - coat_fresnel * clamp(surface.clearcoat, 0.0, 1.0);
}

fn zr_pbr_advanced_environment(
    surface: ZrSurfaceOutput,
    world_position: vec3<f32>,
    view_dir: vec3<f32>,
) -> vec3<f32> {
    if (!ZR_FEATURE_PBR_CLEARCOAT || surface.clearcoat <= 0.0) {
        return vec3<f32>(0.0);
    }
    let coat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);
    let normalized_view_dir = zr_normalize_or_zero(view_dir);
    return zr_pbr_advanced_environment_normalized(
        surface,
        world_position,
        coat_normal,
        normalized_view_dir,
    );
}

fn zr_pbr_advanced_environment_normalized(
    surface: ZrSurfaceOutput,
    world_position: vec3<f32>,
    coat_normal: vec3<f32>,
    normalized_view_dir: vec3<f32>,
) -> vec3<f32> {
    if (!ZR_FEATURE_PBR_CLEARCOAT || surface.clearcoat <= 0.0) {
        return vec3<f32>(0.0);
    }
    if (all(coat_normal == vec3<f32>(0.0)) || all(normalized_view_dir == vec3<f32>(0.0))) {
        return vec3<f32>(0.0);
    }
    let reflected = zr_environment_reflection_color(
        world_position,
        coat_normal,
        normalized_view_dir,
        surface.clearcoat_roughness,
    );
    let no_v = max(dot(coat_normal, normalized_view_dir), 0.0);
    return reflected
        * zr_environment_env_brdf_lut(vec3<f32>(0.04), surface.clearcoat_roughness, no_v)
        * clamp(surface.clearcoat, 0.0, 1.0)
        * clamp(surface.occlusion, 0.0, 1.0);
}

fn zr_pbr_viewport_uv(world_position: vec3<f32>) -> vec2<f32> {
    let clip_position = scene.view_proj * vec4<f32>(world_position, 1.0);
    let w_sign = select(-1.0, 1.0, clip_position.w >= 0.0);
    let safe_w = w_sign * max(abs(clip_position.w), ZR_PBR_EXTRAS_EPSILON);
    let ndc = clip_position.xy / safe_w;
    return clamp(
        vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5),
        vec2<f32>(0.0),
        vec2<f32>(1.0),
    );
}

fn zr_pbr_screen_space_transmission(
    surface: ZrSurfaceOutput,
    world_position: vec3<f32>,
    environment_lighting: vec3<f32>,
) -> vec3<f32> {
    if (!ZR_FEATURE_PBR_TRANSMISSION || surface.specular_transmission <= 0.0) {
        return vec3<f32>(0.0);
    }
    let base_uv = zr_pbr_viewport_uv(world_position);
    let refraction_scale = max(surface.ior - 1.0, 0.0)
        * max(surface.thickness, 0.0)
        * 0.02;
    let refracted_uv = clamp(
        base_uv + zr_normalize_or_zero(surface.normal_ws).xy * refraction_scale,
        vec2<f32>(0.0),
        vec2<f32>(1.0),
    );
    let scene_color_sample = textureSampleLevel(
        zr_transmission_scene_color,
        zr_transmission_scene_color_sampler,
        refracted_uv,
        0.0,
    );
    let transmission_source = select(
        environment_lighting,
        scene_color_sample.rgb,
        scene_color_sample.a > 0.0,
    );
    let attenuation_distance = max(surface.attenuation_distance, ZR_PBR_EXTRAS_EPSILON);
    let attenuation_power = max(surface.thickness, 0.0) / attenuation_distance;
    let attenuation = pow(
        max(surface.attenuation_color, vec3<f32>(ZR_PBR_EXTRAS_EPSILON)),
        vec3<f32>(attenuation_power),
    );
    return transmission_source
        * attenuation
        * clamp(surface.specular_transmission, 0.0, 1.0);
}
