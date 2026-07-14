const ZR_STANDARD_PBR_EPSILON: f32 = 0.000001;
const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;
const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;
const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;

fn zr_standard_pbr_light_radiance(light: ZrGpuLightData) -> vec3<f32> {
    return max(light.color_intensity.rgb, vec3<f32>(0.0)) * max(light.color_intensity.w, 0.0);
}

fn zr_standard_pbr_diffuse_color(surface: ZrSurfaceOutput) -> vec3<f32> {
    if (surface.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return surface.base_color.rgb;
    }
    return surface.base_color.rgb * (1.0 - surface.metallic * 0.45);
}

fn zr_scene_view_dir_ws(position_ws: vec3<f32>) -> vec3<f32> {
    let perspective_view_dir = zr_normalize_or_zero(scene.camera_world_position.xyz - position_ws);
    return zr_normalize_or_zero(mix(
        perspective_view_dir,
        scene.camera_view_direction.xyz,
        clamp(scene.camera_view_direction.w, 0.0, 1.0),
    ));
}

fn zr_standard_pbr_shade_standard_light_vector(
    light_vector: vec3<f32>,
    radiance: vec3<f32>,
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    view_dir_ws: vec3<f32>,
) -> vec3<f32> {
    let world_normal = zr_normalize_or_zero(surface.normal_ws);
    let world_view = zr_normalize_or_zero(view_dir_ws);
    let no_l = max(dot(world_normal, light_vector), 0.0);
    let f0 = mix(vec3<f32>(0.04), max(surface.base_color.rgb, vec3<f32>(0.0)), surface.metallic);
    let specular = select(
        zr_pbr_isotropic_ggx(
            world_normal,
            world_view,
            light_vector,
            surface.roughness,
            f0,
        ),
        zr_aniso_ggx(
            world_normal,
            surface.tangent_ws,
            surface.bitangent_ws,
            world_view,
            light_vector,
            surface.roughness,
            surface.anisotropy_strength,
            surface.anisotropy_rotation,
            f0,
        ),
        ZR_FEATURE_PBR_ANISOTROPY,
    );
    let base_lighting = (
        diffuse_color * (1.0 - surface.metallic) / ZR_PBR_EXTRAS_PI + specular
    ) * radiance * no_l * zr_pbr_clearcoat_base_energy_scale(surface, world_view);
    let clearcoat = zr_clearcoat_lobe(
        zr_normalize_or_zero(surface.clearcoat_normal_ws),
        world_view,
        light_vector,
        surface.clearcoat_roughness,
    ) * radiance * no_l * surface.clearcoat;
    let transmission = zr_transmission_btdf(
        world_normal,
        world_view,
        light_vector,
        surface.base_color.rgb,
        surface.diffuse_transmission,
        surface.ior,
    ) * radiance;
    return base_lighting + select(vec3<f32>(0.0), clearcoat, ZR_FEATURE_PBR_CLEARCOAT)
        + select(vec3<f32>(0.0), transmission, ZR_FEATURE_PBR_TRANSMISSION);
}

fn zr_standard_pbr_shade_blinn_phong_light_vector(
    light_vector: vec3<f32>,
    radiance: vec3<f32>,
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    view_dir_ws: vec3<f32>,
) -> vec3<f32> {
    let world_normal = zr_normalize_or_zero(surface.normal_ws);
    let lambert = max(dot(world_normal, light_vector), 0.0);
    let half_dir = zr_normalize_or_zero(light_vector + view_dir_ws);
    let specular_power = mix(96.0, 12.0, surface.roughness);
    let specular_intensity =
        pow(max(dot(world_normal, half_dir), 0.0), specular_power) * (1.0 - surface.roughness) * 0.5;
    return diffuse_color * radiance * lambert + radiance * specular_intensity;
}

fn zr_standard_pbr_shade_light_vector(
    light_vector: vec3<f32>,
    radiance: vec3<f32>,
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    view_dir_ws: vec3<f32>,
) -> vec3<f32> {
    if (surface.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return zr_standard_pbr_shade_blinn_phong_light_vector(
            light_vector,
            radiance,
            surface,
            diffuse_color,
            view_dir_ws,
        );
    }
    return zr_standard_pbr_shade_standard_light_vector(
        light_vector,
        radiance,
        surface,
        diffuse_color,
        view_dir_ws,
    );
}

fn zr_standard_pbr_punctual_light_visibility(
    light: ZrGpuLightData,
    light_type: u32,
    world_position: vec3<f32>,
    distance_to_light: f32,
) -> f32 {
    let range = max(light.position_range.w, ZR_STANDARD_PBR_EPSILON);
    if (distance_to_light >= range) {
        return 0.0;
    }

    var visibility = pow(clamp(1.0 - distance_to_light / range, 0.0, 1.0), 2.0);
    if (light_type == ZR_GPU_LIGHT_TYPE_SPOT) {
        let light_to_surface = zr_normalize_or_zero(world_position - light.position_range.xyz);
        let cone = dot(zr_normalize_or_zero(light.direction_type.xyz), light_to_surface);
        let inner = light.spot_angles_size.x;
        let outer = light.spot_angles_size.y;
        visibility = visibility * clamp((cone - outer) / max(inner - outer, ZR_STANDARD_PBR_EPSILON), 0.0, 1.0);
    } else if (light_type == ZR_GPU_LIGHT_TYPE_RECT) {
        let light_to_surface = zr_normalize_or_zero(world_position - light.position_range.xyz);
        visibility = visibility * max(dot(zr_normalize_or_zero(light.direction_type.xyz), light_to_surface), 0.0);
    }
    return visibility;
}

fn zr_standard_pbr_shade_gpu_light_index(
    light_index: u32,
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    ctx: ZrShadingContext,
    view_z: f32,
    view_dir_ws: vec3<f32>,
) -> vec3<f32> {
    if (light_index >= zr_gpu_scene_light_count()) {
        return vec3<f32>(0.0);
    }

    let light = zr_gpu_light(light_index);
    let light_type = zr_gpu_light_type(light);
    let base_radiance = zr_standard_pbr_light_radiance(light);
    if (length(base_radiance) <= ZR_STANDARD_PBR_EPSILON) {
        return vec3<f32>(0.0);
    }

    var shadow_visibility = 1.0;
    if (ZR_FEATURE_RECEIVE_SHADOWS && ctx.shadow_params.z > 0.5) {
        shadow_visibility = zr_gpu_light_shadow_visibility(light, light_type, ctx.position_ws, view_z);
    }

    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        let light_vector = zr_normalize_or_zero(-light.direction_type.xyz);
        let radiance = base_radiance * shadow_visibility * surface.occlusion;
        return zr_standard_pbr_shade_light_vector(
            light_vector,
            radiance,
            surface,
            diffuse_color,
            view_dir_ws,
        );
    }

    let to_light = light.position_range.xyz - ctx.position_ws;
    let distance_to_light = length(to_light);
    let light_vector = to_light / max(distance_to_light, ZR_STANDARD_PBR_EPSILON);
    let visibility = zr_standard_pbr_punctual_light_visibility(
        light,
        light_type,
        ctx.position_ws,
        distance_to_light,
    );
    if (visibility <= 0.0) {
        return vec3<f32>(0.0);
    }

    return zr_standard_pbr_shade_light_vector(
        light_vector,
        base_radiance * visibility * shadow_visibility * surface.occlusion,
        surface,
        diffuse_color,
        view_dir_ws,
    );
}

fn zr_standard_pbr_gpu_light_lighting(
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    ctx: ZrShadingContext,
    view_dir_ws: vec3<f32>,
) -> vec3<f32> {
    if (zr_light_grid_params.light_count == 0u || zr_light_grid_params.bin_count == 0u) {
        return vec3<f32>(0.0);
    }

    let view_z = zr_light_view_z(ctx.position_ws, zr_light_grid_params);
    let bin = zr_light_zbin_index(view_z, zr_light_grid_params);
    let header = zr_light_zbin_header(bin, zr_light_grid_params);
    if (header.x == 0xFFFFu || header.x > header.y) {
        return vec3<f32>(0.0);
    }

    let tile_base = zr_light_tile_base(ctx.frag_coord, zr_light_grid_params);
    var accumulated = vec3<f32>(0.0);
    for (var word = header.x / 32u; word <= header.y / 32u; word = word + 1u) {
        var mask = zr_light_mask_word(tile_base, bin, word, zr_light_grid_params);
        while (mask != 0u) {
            let bit_index = firstTrailingBit(mask);
            let light_index = word * 32u + bit_index;
            accumulated = accumulated + zr_standard_pbr_shade_gpu_light_index(
                light_index,
                surface,
                diffuse_color,
                ctx,
                view_z,
                view_dir_ws,
            );
            mask = mask & (mask - 1u);
        }
    }
    return accumulated;
}

fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> {
    if (surface.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        return surface.base_color.rgb + surface.emissive;
    }
    let ambient = scene.ambient_color.rgb * surface.occlusion;
    let diffuse_color = zr_standard_pbr_diffuse_color(surface);
    let view_dir_ws = zr_scene_view_dir_ws(ctx.position_ws);
    let direct_lights = zr_standard_pbr_gpu_light_lighting(
        surface,
        diffuse_color,
        ctx,
        view_dir_ws,
    );
    let environment_lights = zr_environment_pbr_indirect(
        ctx.position_ws,
        surface.normal_ws,
        view_dir_ws,
        surface.roughness,
        surface.metallic,
        diffuse_color,
        surface.base_color.rgb,
        surface.occlusion,
        surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
    );
    let clearcoat_environment = zr_pbr_advanced_environment(surface, ctx.position_ws, view_dir_ws);
    let opaque_lighting = diffuse_color * ambient
        + direct_lights
        + environment_lights * zr_pbr_clearcoat_base_energy_scale(surface, view_dir_ws)
        + clearcoat_environment;
    let specular_transmission = select(
        0.0,
        clamp(surface.specular_transmission, 0.0, 1.0),
        ZR_FEATURE_PBR_TRANSMISSION,
    );
    let transmitted_scene = zr_pbr_screen_space_transmission(
        surface,
        ctx.frag_coord.xy,
        environment_lights,
    );
    return opaque_lighting * (1.0 - specular_transmission)
        + transmitted_scene
        + surface.emissive;
}
