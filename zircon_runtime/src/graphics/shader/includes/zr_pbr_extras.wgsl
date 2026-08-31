@group(1) @binding(31) var zr_transmission_scene_color: texture_2d<f32>;
@group(1) @binding(32) var zr_transmission_scene_color_sampler: sampler;

struct ZrTransmissionSceneColorParams {
    available: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
};

@group(1) @binding(38) var<uniform> zr_transmission_scene_color_params:
    ZrTransmissionSceneColorParams;

fn zr_pbr_smith_joint_visibility_anisotropic(
    no_v: f32,
    no_l: f32,
    to_v: f32,
    bo_v: f32,
    to_l: f32,
    bo_l: f32,
    alpha_t: f32,
    alpha_b: f32,
) -> f32 {
    let visibility_v = no_l * length(vec3<f32>(
        alpha_t * to_v,
        alpha_b * bo_v,
        no_v,
    ));
    let visibility_l = no_v * length(vec3<f32>(
        alpha_t * to_l,
        alpha_b * bo_l,
        no_l,
    ));
    return 0.5 / max(visibility_v + visibility_l, ZR_PBR_EXTRAS_EPSILON);
}

fn zr_pbr_rotated_tangent(
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
    rotation: f32,
) -> vec3<f32> {
    return zr_normalize_or_zero(tangent * cos(rotation) + bitangent * sin(rotation));
}

fn zr_pbr_anisotropic_environment_normal_normalized(
    normal: vec3<f32>,
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
    view_dir: vec3<f32>,
    perceptual_roughness: f32,
    anisotropy_strength: f32,
    anisotropy_rotation: f32,
) -> vec3<f32> {
    let anisotropic_tangent = zr_pbr_rotated_tangent(
        tangent,
        bitangent,
        anisotropy_rotation,
    );
    let anisotropic_bitangent = zr_pbr_normalize_or_zero(cross(
        normal,
        anisotropic_tangent,
    ));
    if (all(anisotropic_bitangent == vec3<f32>(0.0))) {
        return normal;
    }
    let anisotropic_normal = zr_pbr_normalize_or_zero(cross(
        cross(anisotropic_bitangent, view_dir),
        anisotropic_bitangent,
    ));
    if (all(anisotropic_normal == vec3<f32>(0.0))) {
        return normal;
    }
    let roughness = clamp(perceptual_roughness, 0.0, 1.0);
    let strength = clamp(anisotropy_strength, 0.0, 1.0);
    let bend_factor = 1.0 - strength * (1.0 - roughness);
    let bend_factor_sq = bend_factor * bend_factor;
    let bend_factor_pow4 = bend_factor_sq * bend_factor_sq;
    let bent_normal = zr_pbr_normalize_or_zero(mix(
        anisotropic_normal,
        normal,
        bend_factor_pow4,
    ));
    return bent_normal;
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
    let strength = clamp(anisotropy_strength, 0.0, 1.0);
    if (strength <= ZR_PBR_EXTRAS_EPSILON) {
        return zr_pbr_isotropic_ggx(
            normal,
            view_dir,
            light_dir,
            perceptual_roughness,
            f0,
        );
    }
    let half_dir = zr_normalize_or_zero(view_dir + light_dir);
    let rotated_tangent = zr_pbr_rotated_tangent(tangent, bitangent, anisotropy_rotation);
    let rotated_bitangent = zr_normalize_or_zero(cross(normal, rotated_tangent));
    if (all(rotated_tangent == vec3<f32>(0.0))
        || all(rotated_bitangent == vec3<f32>(0.0)))
    {
        return zr_pbr_isotropic_ggx(
            normal,
            view_dir,
            light_dir,
            perceptual_roughness,
            f0,
        );
    }
    let base_alpha = max(perceptual_roughness * perceptual_roughness, 0.001);
    let alpha_t = mix(base_alpha, 1.0, strength * strength);
    let alpha_b = base_alpha;
    let to_v = dot(rotated_tangent, view_dir);
    let bo_v = dot(rotated_bitangent, view_dir);
    let to_l = dot(rotated_tangent, light_dir);
    let bo_l = dot(rotated_bitangent, light_dir);
    let to_h = dot(rotated_tangent, half_dir);
    let bo_h = dot(rotated_bitangent, half_dir);
    let no_h = clamp(dot(normal, half_dir), 0.0, 1.0);
    let no_v = clamp(dot(normal, view_dir), ZR_PBR_EXTRAS_EPSILON, 1.0);
    let no_l = clamp(dot(normal, light_dir), ZR_PBR_EXTRAS_EPSILON, 1.0);
    let vo_h = clamp(dot(view_dir, half_dir), 0.0, 1.0);
    let alpha_product = alpha_t * alpha_b;
    let distribution_vector = vec3<f32>(
        alpha_b * to_h,
        alpha_t * bo_h,
        alpha_product * no_h,
    );
    let distribution_length_squared = dot(distribution_vector, distribution_vector);
    var distribution = 0.0;
    if (distribution_length_squared > 0.0) {
        let distribution_scale = alpha_product / distribution_length_squared;
        distribution = alpha_product
            * distribution_scale
            * distribution_scale
            / ZR_PBR_EXTRAS_PI;
    }
    let visibility = zr_pbr_smith_joint_visibility_anisotropic(
        no_v,
        no_l,
        to_v,
        bo_v,
        to_l,
        bo_l,
        alpha_t,
        alpha_b,
    );
    let fresnel = zr_pbr_fresnel_schlick(vo_h, f0);
    return fresnel * distribution * visibility;
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
    dielectric_f0: f32,
    metallic: f32,
) -> vec3<f32> {
    let back_lambert = max(dot(-normal, light_dir), 0.0);
    let fresnel = zr_pbr_fresnel_schlick(
        max(dot(normal, view_dir), 0.0),
        vec3<f32>(clamp(dielectric_f0, 0.0, 1.0)),
    );
    return base_color
        * (vec3<f32>(1.0) - fresnel)
        * back_lambert
        * clamp(diffuse_transmission, 0.0, 1.0)
        * zr_surface_metallic_diffuse_energy_scale(metallic)
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
    let clamped_roughness = clamp(surface.clearcoat_roughness, 0.0, 1.0);
    let clamped_occlusion = clamp(surface.occlusion, 0.0, 1.0);
    if (clamped_occlusion <= 0.0) {
        return vec3<f32>(0.0);
    }
    let planar = zr_environment_planar_reflection(world_position, clamped_roughness);
    var reflected = vec3<f32>(0.0);
    if (planar.a > 0.0) {
        reflected = planar.rgb;
    } else {
        let has_global_environment = zr_environment_is_enabled()
            && scene.environment_params.y > 0.0;
        reflected = zr_environment_reflection_color_after_planar(
            world_position,
            coat_normal,
            normalized_view_dir,
            clamped_roughness,
            has_global_environment,
        );
    }
    if (all(reflected == vec3<f32>(0.0))) {
        return vec3<f32>(0.0);
    }
    let no_v = max(dot(coat_normal, normalized_view_dir), 0.0);
    let specular_occlusion =
        zr_environment_specular_occlusion(no_v, clamped_roughness, clamped_occlusion);
    return reflected
        * zr_environment_env_brdf_lut(vec3<f32>(0.04), clamped_roughness, no_v)
        * clamp(surface.clearcoat, 0.0, 1.0)
        * specular_occlusion;
}

struct ZrPbrViewportProjection {
    uv: vec2<f32>,
    valid: bool,
};

fn zr_pbr_viewport_projection(world_position: vec3<f32>) -> ZrPbrViewportProjection {
    let clip_position = scene.view_proj * vec4<f32>(world_position, 1.0);
    if (clip_position.w <= ZR_PBR_EXTRAS_EPSILON) {
        return ZrPbrViewportProjection(vec2<f32>(0.5), false);
    }
    let ndc = clip_position.xyz / clip_position.w;
    let valid = all(abs(ndc.xy) <= vec2<f32>(1.0))
        && ndc.z >= -ZR_PBR_EXTRAS_EPSILON
        && ndc.z <= 1.0;
    return ZrPbrViewportProjection(
        vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5),
        valid,
    );
}

struct ZrPbrTransmissionFrame {
    exit_position: vec3<f32>,
    environment_direction: vec3<f32>,
    transmission_distance: f32,
    fresnel_cosine: f32,
};

fn zr_pbr_transmission_frame_normalized(
    surface: ZrSurfaceOutput,
    world_position: vec3<f32>,
    instance_index: u32,
    normalized_normal: vec3<f32>,
    normalized_view_dir: vec3<f32>,
) -> ZrPbrTransmissionFrame {
    let normal = zr_pbr_normalize_or_zero(normalized_normal);
    let view_dir = zr_pbr_normalize_or_zero(normalized_view_dir);
    let thickness = max(surface.thickness, 0.0);
    let fresnel_cosine = clamp(dot(normal, view_dir), 0.0, 1.0);
    if (thickness <= 0.0) {
        return ZrPbrTransmissionFrame(
            world_position,
            -view_dir,
            0.0,
            fresnel_cosine,
        );
    }
    let world_from_local = zr_world_from_local(instance_index);
    let inverse_ior = 1.0 / max(surface.ior, 1.0);
    let refracted_direction = refract(-view_dir, normal, inverse_ior);
    let model_scale = vec3<f32>(
        length(world_from_local[0].xyz),
        length(world_from_local[1].xyz),
        length(world_from_local[2].xyz),
    );
    let transmission_ray = zr_pbr_normalize_or_zero(refracted_direction)
        * thickness
        * model_scale;
    let exit_position = world_position + transmission_ray;
    let environment_direction = -zr_pbr_normalize_or_zero(
        view_dir - transmission_ray,
    );
    let transmission_distance = length(transmission_ray);
    return ZrPbrTransmissionFrame(
        exit_position,
        environment_direction,
        transmission_distance,
        fresnel_cosine,
    );
}

fn zr_pbr_volume_attenuation(
    surface: ZrSurfaceOutput,
    transmission_distance: f32,
) -> vec3<f32> {
    if (transmission_distance <= 0.0
        || surface.attenuation_distance >= ZR_PBR_NO_ATTENUATION_DISTANCE)
    {
        return vec3<f32>(1.0);
    }
    let attenuation_distance = max(surface.attenuation_distance, ZR_PBR_EXTRAS_EPSILON);
    let attenuation_power = transmission_distance / attenuation_distance;
    return pow(
        clamp(surface.attenuation_color, vec3<f32>(0.0), vec3<f32>(1.0)),
        vec3<f32>(attenuation_power),
    );
}

fn zr_pbr_transmission_source(
    surface: ZrSurfaceOutput,
    transmission_frame: ZrPbrTransmissionFrame,
) -> vec3<f32> {
    if (zr_transmission_scene_color_params.available != 0u) {
        let refracted_projection = zr_pbr_viewport_projection(
            transmission_frame.exit_position,
        );
        if (refracted_projection.valid) {
            return textureSampleLevel(
                zr_transmission_scene_color,
                zr_transmission_scene_color_sampler,
                refracted_projection.uv,
                0.0,
            ).rgb;
        }
    }
    return zr_environment_transmission_radiance_normalized(
        transmission_frame.exit_position,
        transmission_frame.environment_direction,
        surface.roughness,
    );
}

fn zr_pbr_screen_space_transmission(
    surface: ZrSurfaceOutput,
    transmission_frame: ZrPbrTransmissionFrame,
    volume_attenuation: vec3<f32>,
) -> vec3<f32> {
    if (!ZR_FEATURE_PBR_TRANSMISSION || surface.specular_transmission <= 0.0) {
        return vec3<f32>(0.0);
    }
    let transmission_source = zr_pbr_transmission_source(surface, transmission_frame);
    let transmission_f0 = zr_pbr_material_f0(
        surface.dielectric_f0,
        surface.base_color.rgb,
        surface.metallic,
    );
    let transmission_energy = zr_pbr_transmission_energy_scale(
        transmission_frame.fresnel_cosine,
        transmission_f0,
        surface.metallic,
    );
    let transmission_tint = zr_pbr_base_color(surface.base_color.rgb)
        * transmission_energy;
    return transmission_source
        * transmission_tint
        * volume_attenuation
        * clamp(surface.specular_transmission, 0.0, 1.0);
}
