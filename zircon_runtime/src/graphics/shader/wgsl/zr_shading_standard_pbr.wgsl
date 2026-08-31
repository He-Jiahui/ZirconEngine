const ZR_STANDARD_PBR_EPSILON: f32 = 0.000001;

struct ZrStandardPbrLayerLighting {
    base_diffuse: vec3<f32>,
    retained_reflection: vec3<f32>,
};

fn zr_standard_pbr_light_radiance(light: ZrGpuLightData) -> vec3<f32> {
    return max(light.color_intensity.rgb, vec3<f32>(0.0)) * max(light.color_intensity.w, 0.0);
}

fn zr_standard_pbr_diffuse_color(surface: ZrSurfaceOutput) -> vec3<f32> {
    if (surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID) {
        return zr_pbr_base_color(surface.base_color.rgb);
    }
    return surface.base_color.rgb;
}

fn zr_standard_pbr_ambient_diffuse_energy_scale(
    surface: ZrSurfaceOutput,
) -> vec3<f32> {
    if (surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID) {
        return vec3<f32>(
            zr_surface_metallic_diffuse_energy_scale(surface.metallic),
        );
    }
    return vec3<f32>(1.0);
}

fn zr_standard_pbr_shade_standard_light_vector_normalized(
    light_vector: vec3<f32>,
    radiance: vec3<f32>,
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    direct_diffuse_brdf: vec3<f32>,
    world_normal: vec3<f32>,
    world_view: vec3<f32>,
    direct_f0: vec3<f32>,
    direct_base_energy: vec3<f32>,
    direct_clearcoat_normal: vec3<f32>,
    reflected_diffuse_weight: f32,
    diffuse_transmission: f32,
    volume_attenuation: vec3<f32>,
) -> ZrStandardPbrLayerLighting {
    let no_l = max(dot(world_normal, light_vector), 0.0);
    var specular = zr_pbr_isotropic_ggx(
        world_normal,
        world_view,
        light_vector,
        surface.roughness,
        direct_f0,
    );
    if (ZR_FEATURE_PBR_ANISOTROPY) {
        specular = zr_aniso_ggx(
            world_normal,
            surface.tangent_ws,
            surface.bitangent_ws,
            world_view,
            light_vector,
            surface.roughness,
            surface.anisotropy_strength,
            surface.anisotropy_rotation,
            direct_f0,
        );
    }
    let reflected_diffuse =
        direct_diffuse_brdf
        * reflected_diffuse_weight
        * radiance
        * no_l;
    let retained_base_specular = specular
        * radiance
        * no_l
        * direct_base_energy;
    var clearcoat = vec3<f32>(0.0);
    if (ZR_FEATURE_PBR_CLEARCOAT && surface.clearcoat > 0.0) {
        let clearcoat_no_l = max(dot(direct_clearcoat_normal, light_vector), 0.0);
        if (clearcoat_no_l > 0.0) {
            clearcoat = zr_clearcoat_lobe(
                direct_clearcoat_normal,
                world_view,
                light_vector,
                surface.clearcoat_roughness,
            ) * radiance * clearcoat_no_l * clamp(surface.clearcoat, 0.0, 1.0);
        }
    }
    var transmitted_diffuse = vec3<f32>(0.0);
    if (ZR_FEATURE_PBR_TRANSMISSION && diffuse_transmission > 0.0) {
        transmitted_diffuse = zr_transmission_btdf(
            world_normal,
            world_view,
            light_vector,
            diffuse_color,
            diffuse_transmission,
            surface.dielectric_f0.x,
            surface.metallic,
        ) * radiance * volume_attenuation;
    }
    return ZrStandardPbrLayerLighting(
        reflected_diffuse + transmitted_diffuse,
        retained_base_specular + clearcoat,
    );
}

fn zr_standard_pbr_shade_blinn_phong_light_vector_normalized(
    light_vector: vec3<f32>,
    radiance: vec3<f32>,
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    world_normal: vec3<f32>,
    world_view: vec3<f32>,
) -> ZrStandardPbrLayerLighting {
    let lambert = max(dot(world_normal, light_vector), 0.0);
    let half_dir = zr_normalize_or_zero(light_vector + world_view);
    let specular_power = mix(96.0, 12.0, surface.roughness);
    let specular_intensity =
        pow(max(dot(world_normal, half_dir), 0.0), specular_power) * (1.0 - surface.roughness) * 0.5;
    return ZrStandardPbrLayerLighting(
        diffuse_color * radiance * lambert + radiance * specular_intensity,
        vec3<f32>(0.0),
    );
}

fn zr_standard_pbr_shade_light_vector_normalized(
    light_vector: vec3<f32>,
    radiance: vec3<f32>,
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    direct_diffuse_brdf: vec3<f32>,
    world_normal: vec3<f32>,
    world_view: vec3<f32>,
    direct_f0: vec3<f32>,
    direct_base_energy: vec3<f32>,
    direct_clearcoat_normal: vec3<f32>,
    reflected_diffuse_weight: f32,
    diffuse_transmission: f32,
    volume_attenuation: vec3<f32>,
) -> ZrStandardPbrLayerLighting {
    if (surface.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return zr_standard_pbr_shade_blinn_phong_light_vector_normalized(
            light_vector,
            radiance,
            surface,
            diffuse_color,
            world_normal,
            world_view,
        );
    }
    return zr_standard_pbr_shade_standard_light_vector_normalized(
        light_vector,
        radiance,
        surface,
        diffuse_color,
        direct_diffuse_brdf,
        world_normal,
        world_view,
        direct_f0,
        direct_base_energy,
        direct_clearcoat_normal,
        reflected_diffuse_weight,
        diffuse_transmission,
        volume_attenuation,
    );
}

fn zr_standard_pbr_punctual_light_visibility(
    light: ZrGpuLightData,
    light_type: u32,
    light_vector_to_light: vec3<f32>,
    distance_to_light: f32,
    range: f32,
) -> f32 {
    var visibility = pow(clamp(1.0 - distance_to_light / range, 0.0, 1.0), 2.0);
    let light_to_surface = select(
        vec3<f32>(0.0),
        -light_vector_to_light,
        distance_to_light > ZR_STANDARD_PBR_EPSILON,
    );
    if (light_type == ZR_GPU_LIGHT_TYPE_SPOT) {
        let cone = dot(zr_normalize_or_zero(light.direction_type.xyz), light_to_surface);
        let inner = light.spot_angles_size.x;
        let outer = light.spot_angles_size.y;
        visibility = visibility * clamp((cone - outer) / max(inner - outer, ZR_STANDARD_PBR_EPSILON), 0.0, 1.0);
    } else if (light_type == ZR_GPU_LIGHT_TYPE_RECT) {
        visibility = visibility * max(dot(zr_normalize_or_zero(light.direction_type.xyz), light_to_surface), 0.0);
    }
    return visibility;
}

fn zr_standard_pbr_shade_gpu_light_index(
    light_index: u32,
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    direct_diffuse_brdf: vec3<f32>,
    ctx: ZrShadingContext,
    view_z: f32,
    world_normal: vec3<f32>,
    world_view: vec3<f32>,
    direct_f0: vec3<f32>,
    direct_base_energy: vec3<f32>,
    direct_clearcoat_normal: vec3<f32>,
    reflected_diffuse_weight: f32,
    diffuse_transmission: f32,
    volume_attenuation: vec3<f32>,
) -> ZrStandardPbrLayerLighting {
    if (light_index >= zr_gpu_scene_light_count()) {
        return ZrStandardPbrLayerLighting(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    let light = zr_gpu_light(light_index);
    let light_type = zr_gpu_light_type(light);
    let base_radiance = zr_standard_pbr_light_radiance(light)
        * zr_light_cookie_factor(light, ctx.position_ws);
    if (length(base_radiance) <= ZR_STANDARD_PBR_EPSILON) {
        return ZrStandardPbrLayerLighting(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    var shadow_visibility = 1.0;
    if (ZR_FEATURE_RECEIVE_SHADOWS && ctx.shadow_params.z > 0.5) {
        shadow_visibility = zr_gpu_light_shadow_visibility(light, light_type, ctx.position_ws, view_z);
    }

    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        let light_vector = zr_normalize_or_zero(-light.direction_type.xyz);
        let radiance = base_radiance * shadow_visibility;
        return zr_standard_pbr_shade_light_vector_normalized(
            light_vector,
            radiance,
            surface,
            diffuse_color,
            direct_diffuse_brdf,
            world_normal,
            world_view,
            direct_f0,
            direct_base_energy,
            direct_clearcoat_normal,
            reflected_diffuse_weight,
            diffuse_transmission,
            volume_attenuation,
        );
    }

    let to_light = light.position_range.xyz - ctx.position_ws;
    let distance_to_light = length(to_light);
    let range = max(light.position_range.w, ZR_STANDARD_PBR_EPSILON);
    if (distance_to_light >= range) {
        return ZrStandardPbrLayerLighting(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    let light_vector = to_light / max(distance_to_light, ZR_STANDARD_PBR_EPSILON);
    let visibility = zr_standard_pbr_punctual_light_visibility(
        light,
        light_type,
        light_vector,
        distance_to_light,
        range,
    );
    if (visibility <= 0.0) {
        return ZrStandardPbrLayerLighting(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    return zr_standard_pbr_shade_light_vector_normalized(
        light_vector,
        base_radiance * visibility * shadow_visibility,
        surface,
        diffuse_color,
        direct_diffuse_brdf,
        world_normal,
        world_view,
        direct_f0,
        direct_base_energy,
        direct_clearcoat_normal,
        reflected_diffuse_weight,
        diffuse_transmission,
        volume_attenuation,
    );
}

fn zr_standard_pbr_gpu_light_lighting(
    surface: ZrSurfaceOutput,
    diffuse_color: vec3<f32>,
    ctx: ZrShadingContext,
    world_view: vec3<f32>,
    world_normal: vec3<f32>,
    direct_clearcoat_normal: vec3<f32>,
    direct_base_energy: vec3<f32>,
    reflected_diffuse_weight: f32,
    diffuse_transmission: f32,
    volume_attenuation: vec3<f32>,
) -> ZrStandardPbrLayerLighting {
    if (zr_light_grid_params.light_count == 0u || zr_light_grid_params.bin_count == 0u) {
        return ZrStandardPbrLayerLighting(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    let view_z = zr_light_view_z(ctx.position_ws, zr_light_grid_params);
    let bin = zr_light_zbin_index(view_z, zr_light_grid_params);
    let header = zr_light_zbin_header(bin, zr_light_grid_params);
    if (header.x == 0xFFFFu || header.x > header.y) {
        return ZrStandardPbrLayerLighting(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    var direct_f0 = vec3<f32>(0.0);
    var direct_diffuse_brdf = diffuse_color / ZR_PBR_EXTRAS_PI;
    if (surface.shading_model_id != ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        let direct_metallic = clamp(surface.metallic, 0.0, 1.0);
        direct_f0 = zr_pbr_material_f0(
            surface.dielectric_f0,
            diffuse_color,
            direct_metallic,
        );
        direct_diffuse_brdf = diffuse_color
            * zr_surface_metallic_diffuse_energy_scale(direct_metallic)
            / ZR_PBR_EXTRAS_PI;
    }
    let tile_base = zr_light_tile_base(ctx.frag_coord, zr_light_grid_params);
    var base_diffuse = vec3<f32>(0.0);
    var retained_reflection = vec3<f32>(0.0);
    for (var word = header.x / 32u; word <= header.y / 32u; word = word + 1u) {
        var mask = zr_light_mask_word(tile_base, bin, word, zr_light_grid_params);
        while (mask != 0u) {
            let bit_index = firstTrailingBit(mask);
            let light_index = word * 32u + bit_index;
            let light_layers = zr_standard_pbr_shade_gpu_light_index(
                light_index,
                surface,
                diffuse_color,
                direct_diffuse_brdf,
                ctx,
                view_z,
                world_normal,
                world_view,
                direct_f0,
                direct_base_energy,
                direct_clearcoat_normal,
                reflected_diffuse_weight,
                diffuse_transmission,
                volume_attenuation,
            );
            base_diffuse += light_layers.base_diffuse;
            retained_reflection += light_layers.retained_reflection;
            mask = mask & (mask - 1u);
        }
    }
    return ZrStandardPbrLayerLighting(base_diffuse, retained_reflection);
}

fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> {
    if (surface.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        return surface.base_color.rgb + surface.emissive;
    }
    let ambient_radiance = zr_scene_ambient_color(
        zr_gpu_scene_has_lightmap(ctx.instance_index),
    );
    let ambient = ambient_radiance * surface.occlusion;
    let diffuse_color = zr_standard_pbr_diffuse_color(surface);
    let view_dir_ws = zr_pbr_view_direction_ws(ctx.position_ws);
    let world_normal = zr_normalize_or_zero(surface.normal_ws);
    let specular_transmission = select(
        0.0,
        clamp(surface.specular_transmission, 0.0, 1.0),
        ZR_FEATURE_PBR_TRANSMISSION
            && surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
    );
    let diffuse_transmission = select(
        0.0,
        clamp(surface.diffuse_transmission, 0.0, 1.0),
        ZR_FEATURE_PBR_TRANSMISSION
            && surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID
            && specular_transmission <= 0.0
            && surface.diffuse_transmission > 0.0,
    );
    let reflected_diffuse_weight = (1.0 - specular_transmission)
        * (1.0 - diffuse_transmission);
    let ambient_diffuse_energy = zr_standard_pbr_ambient_diffuse_energy_scale(surface);
    var clearcoat_normal = vec3<f32>(0.0);
    var direct_clearcoat_normal = vec3<f32>(0.0);
    var clearcoat_base_energy = vec3<f32>(1.0);
    if (ZR_FEATURE_PBR_CLEARCOAT && surface.clearcoat > 0.0) {
        clearcoat_normal = zr_normalize_or_zero(surface.clearcoat_normal_ws);
        if (any(clearcoat_normal != vec3<f32>(0.0)) && any(view_dir_ws != vec3<f32>(0.0))) {
            direct_clearcoat_normal = clearcoat_normal;
            clearcoat_base_energy = zr_pbr_clearcoat_base_energy_scale_normalized(
                surface,
                clearcoat_normal,
                view_dir_ws,
            );
        }
    }
    var diffuse_transmission_attenuation = vec3<f32>(1.0);
    if (diffuse_transmission > 0.0) {
        let diffuse_transmission_frame = zr_pbr_transmission_frame_normalized(
            surface,
            ctx.position_ws,
            ctx.instance_index,
            world_normal,
            view_dir_ws,
        );
        diffuse_transmission_attenuation = zr_pbr_volume_attenuation(
            surface,
            diffuse_transmission_frame.transmission_distance,
        );
    }
    let direct_lights = zr_standard_pbr_gpu_light_lighting(
        surface,
        diffuse_color,
        ctx,
        view_dir_ws,
        world_normal,
        direct_clearcoat_normal,
        clearcoat_base_energy,
        reflected_diffuse_weight,
        diffuse_transmission,
        diffuse_transmission_attenuation,
    );
    var environment_specular_normal = world_normal;
    if (ZR_FEATURE_PBR_ANISOTROPY && surface.anisotropy_strength > 0.0) {
        environment_specular_normal =
            zr_pbr_anisotropic_environment_normal_normalized(
                world_normal,
                surface.tangent_ws,
                surface.bitangent_ws,
                view_dir_ws,
                surface.roughness,
                surface.anisotropy_strength,
                surface.anisotropy_rotation,
            );
    }
    let environment_components =
        zr_environment_pbr_components_with_dielectric_f0_and_specular_normal_normalized(
            ctx.position_ws,
            world_normal,
            view_dir_ws,
            environment_specular_normal,
            surface.roughness,
            surface.metallic,
            diffuse_color,
            diffuse_color,
            surface.dielectric_f0,
            surface.occlusion,
            surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
        );
    var transmitted_indirect_diffuse = vec3<f32>(0.0);
    if (diffuse_transmission > 0.0) {
        var transmitted_environment_irradiance = vec3<f32>(0.0);
        if (zr_environment_is_enabled() && scene.environment_params.y > 0.0) {
            transmitted_environment_irradiance =
                zr_environment_diffuse_color_normalized(-world_normal);
        }
        transmitted_indirect_diffuse =
            (ambient_radiance + transmitted_environment_irradiance)
            * diffuse_color
            * ambient_diffuse_energy
            * diffuse_transmission
            * diffuse_transmission_attenuation;
    }
    var clearcoat_environment = vec3<f32>(0.0);
    if (ZR_FEATURE_PBR_CLEARCOAT && surface.clearcoat > 0.0) {
        if (any(clearcoat_normal != vec3<f32>(0.0)) && any(view_dir_ws != vec3<f32>(0.0))) {
            clearcoat_environment = zr_pbr_advanced_environment_normalized(
                surface,
                ctx.position_ws,
                clearcoat_normal,
                view_dir_ws,
            );
        }
    }
    let base_diffuse_lighting =
        (diffuse_color * ambient_diffuse_energy * ambient
            + environment_components.diffuse)
        * reflected_diffuse_weight
        + direct_lights.base_diffuse
        + transmitted_indirect_diffuse;
    let retained_reflection_lighting = direct_lights.retained_reflection
        + environment_components.specular * clearcoat_base_energy
        + clearcoat_environment;
    var transmitted_scene = vec3<f32>(0.0);
    if (specular_transmission > 0.0) {
        let transmission_frame = zr_pbr_transmission_frame_normalized(
            surface,
            ctx.position_ws,
            ctx.instance_index,
            world_normal,
            view_dir_ws,
        );
        let specular_transmission_attenuation = zr_pbr_volume_attenuation(
            surface,
            transmission_frame.transmission_distance,
        );
        transmitted_scene = zr_pbr_screen_space_transmission(
            surface,
            transmission_frame,
            specular_transmission_attenuation,
        );
    }
    return base_diffuse_lighting * clearcoat_base_energy
        + retained_reflection_lighting
        + transmitted_scene * clearcoat_base_energy
        + surface.emissive * clearcoat_base_energy;
}
