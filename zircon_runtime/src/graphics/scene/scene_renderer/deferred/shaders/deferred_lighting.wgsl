struct SceneUniform {
    view_proj: mat4x4<f32>,
    view_proj_unjittered: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
    lightmapped_ambient_color: vec4<f32>,
    previous_view_proj_unjittered: mat4x4<f32>,
    motion_params: vec4<f32>,
    jitter_params: vec4<f32>,
    camera_world_position: vec4<f32>,
    camera_view_direction: vec4<f32>,
    sky_horizon_color: vec4<f32>,
    sky_zenith_color: vec4<f32>,
    sky_ground_color: vec4<f32>,
    sky_sun_direction: vec4<f32>,
    sky_sun_color_radius: vec4<f32>,
    sky_sun_params: vec4<f32>,
    environment_params: vec4<f32>,
    environment_sample_params: vec4<f32>,
    environment_rotation_sin_cos: vec4<f32>,
};

const ZR_STANDARD_MATERIAL_MIN_ROUGHNESS: f32 = 0.001;

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var gbuffer_albedo_tex: texture_2d<f32>;
@group(1) @binding(1) var normal_tex: texture_2d<f32>;
@group(1) @binding(2) var ambient_occlusion_tex: texture_2d<f32>;
@group(1) @binding(3) var gbuffer_material_tex: texture_2d<f32>;
@group(1) @binding(4) var scene_depth_tex: texture_depth_2d;
@group(1) @binding(5) var gbuffer_emissive_tex: texture_2d<f32>;

const EPSILON: f32 = 0.000001;
const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;
const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;
const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;
const ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK: u32 = 0x7Fu;
const ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG: u32 = 0x80u;

struct ZrDeferredLightingComponents {
    diffuse: vec3<f32>,
    retained: vec3<f32>,
};

struct ZrDeferredLightingMrtOutput {
    @location(0) scene_color: vec4<f32>,
    @location(1) sss_diffuse: vec4<f32>,
    @location(2) sss_retained: vec4<f32>,
};

var<private> zr_deferred_selected_ambient_color: vec3<f32>;

fn deferred_ambient_color(lightmapped: bool) -> vec3<f32> {
    return select(scene.ambient_color.rgb, scene.lightmapped_ambient_color.rgb, lightmapped);
}

fn zr_deferred_ambient_radiance() -> vec3<f32> {
    return zr_deferred_selected_ambient_color;
}

fn select_deferred_ambient(emissive: vec4<f32>) {
    let lightmapped = emissive.a > 0.5;
    zr_deferred_selected_ambient_color = deferred_ambient_color(lightmapped);
}

fn deferred_material_flags(encoded: f32) -> u32 {
    return u32(round(clamp(encoded, 0.0, 1.0) * 255.0));
}

fn decode_shading_model_id(encoded: f32) -> u32 {
    return deferred_material_flags(encoded) & ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK;
}

fn decode_receive_shadows(encoded: f32) -> bool {
    return (deferred_material_flags(encoded) & ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG) != 0u;
}

fn screen_uv_to_clip(uv: vec2<f32>, depth: f32) -> vec4<f32> {
    return vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
}

fn reconstruct_world_position(coord: vec2<i32>, depth: f32) -> vec3<f32> {
    let viewport_size = max(textureDimensions(scene_depth_tex), vec2<u32>(1u, 1u));
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5, 0.5)) / vec2<f32>(viewport_size);
    let world = scene.inverse_view_proj * screen_uv_to_clip(uv, depth);
    if (abs(world.w) <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    return world.xyz / world.w;
}

fn normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    return zr_pbr_common_normalize_or_zero(value);
}

fn light_radiance(light: ZrGpuLightData) -> vec3<f32> {
    return max(light.color_intensity.rgb, vec3<f32>(0.0, 0.0, 0.0)) * max(light.color_intensity.w, 0.0);
}

fn shade_standard_pbr_light_vector_components_normalized(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, roughness: f32, direct_f0: vec3<f32>, direct_diffuse_brdf: vec3<f32>, world_view: vec3<f32>) -> ZrDeferredLightingComponents {
    let world_light = light_vector;
    let lambert = max(dot(world_normal, world_light), 0.0);
    let specular = zr_pbr_isotropic_ggx(
        world_normal,
        world_view,
        world_light,
        roughness,
        direct_f0,
    );
    return ZrDeferredLightingComponents(
        direct_diffuse_brdf * radiance * lambert,
        radiance * specular * lambert,
    );
}

fn shade_standard_pbr_light_vector_normalized(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, roughness: f32, direct_f0: vec3<f32>, direct_diffuse_brdf: vec3<f32>, world_view: vec3<f32>) -> vec3<f32> {
    let components = shade_standard_pbr_light_vector_components_normalized(light_vector, radiance, world_normal, roughness, direct_f0, direct_diffuse_brdf, world_view);
    return components.diffuse + components.retained;
}

fn shade_blinn_phong_light_vector_normalized(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, roughness: f32, diffuse_color: vec3<f32>, world_view: vec3<f32>) -> vec3<f32> {
    let lambert = max(dot(world_normal, light_vector), 0.0);
    let half_dir = normalize_or_zero(light_vector + world_view);
    let specular_power = mix(96.0, 12.0, roughness);
    let specular = pow(max(dot(world_normal, half_dir), 0.0), specular_power) * (1.0 - roughness) * 0.5;
    return diffuse_color * radiance * lambert + radiance * specular;
}

fn shade_light_vector_normalized(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, roughness: f32, diffuse_color: vec3<f32>, direct_f0: vec3<f32>, direct_diffuse_brdf: vec3<f32>, world_view: vec3<f32>, shading_model_id: u32) -> vec3<f32> {
    if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return shade_blinn_phong_light_vector_normalized(light_vector, radiance, world_normal, roughness, diffuse_color, world_view);
    }
    return shade_standard_pbr_light_vector_normalized(light_vector, radiance, world_normal, roughness, direct_f0, direct_diffuse_brdf, world_view);
}

fn punctual_light_visibility(light: ZrGpuLightData, light_type: u32, light_vector_to_light: vec3<f32>, distance_to_light: f32, range: f32) -> f32 {
    var visibility = pow(clamp(1.0 - distance_to_light / range, 0.0, 1.0), 2.0);
    let light_to_surface = select(
        vec3<f32>(0.0, 0.0, 0.0),
        -light_vector_to_light,
        distance_to_light > EPSILON,
    );
    if (light_type == ZR_GPU_LIGHT_TYPE_SPOT) {
        let cone = dot(normalize_or_zero(light.direction_type.xyz), light_to_surface);
        let inner = light.spot_angles_size.x;
        let outer = light.spot_angles_size.y;
        visibility = visibility * clamp((cone - outer) / max(inner - outer, EPSILON), 0.0, 1.0);
    } else if (light_type == ZR_GPU_LIGHT_TYPE_RECT) {
        visibility = visibility * max(dot(normalize_or_zero(light.direction_type.xyz), light_to_surface), 0.0);
    }
    return visibility;
}

fn shade_gpu_light_index(light_index: u32, world_position: vec3<f32>, world_normal: vec3<f32>, roughness: f32, diffuse_color: vec3<f32>, direct_f0: vec3<f32>, direct_diffuse_brdf: vec3<f32>, world_view: vec3<f32>, view_z: f32, shading_model_id: u32, receive_shadows: bool) -> vec3<f32> {
    if (light_index >= zr_gpu_scene_light_count()) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let light = zr_gpu_light(light_index);
    let light_type = zr_gpu_light_type(light);
    let base_radiance = light_radiance(light) * zr_light_cookie_factor(light, world_position);
    if (length(base_radiance) <= EPSILON) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        let light_vector = normalize_or_zero(-light.direction_type.xyz);
        var direct_visibility = 1.0;
        if (receive_shadows) {
            direct_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
        }
        return shade_light_vector_normalized(
            light_vector,
            base_radiance * direct_visibility,
            world_normal,
            roughness,
            diffuse_color,
            direct_f0,
            direct_diffuse_brdf,
            world_view,
            shading_model_id,
        );
    }

    let to_light = light.position_range.xyz - world_position;
    let distance_to_light = length(to_light);
    let range = max(light.position_range.w, EPSILON);
    if (distance_to_light >= range) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let light_vector = to_light / max(distance_to_light, EPSILON);
    let visibility = punctual_light_visibility(light, light_type, light_vector, distance_to_light, range);
    if (visibility <= 0.0) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    var shadow_visibility = 1.0;
    if (receive_shadows) {
        shadow_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
    }
    return shade_light_vector_normalized(
        light_vector,
        base_radiance * visibility * shadow_visibility,
        world_normal,
        roughness,
        diffuse_color,
        direct_f0,
        direct_diffuse_brdf,
        world_view,
        shading_model_id,
    );
}

fn shade_gpu_light_index_components(light_index: u32, world_position: vec3<f32>, world_normal: vec3<f32>, roughness: f32, direct_f0: vec3<f32>, direct_diffuse_brdf: vec3<f32>, world_view: vec3<f32>, view_z: f32, receive_shadows: bool) -> ZrDeferredLightingComponents {
    if (light_index >= zr_gpu_scene_light_count()) {
        return ZrDeferredLightingComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    let light = zr_gpu_light(light_index);
    let light_type = zr_gpu_light_type(light);
    let base_radiance = light_radiance(light) * zr_light_cookie_factor(light, world_position);
    if (length(base_radiance) <= EPSILON) {
        return ZrDeferredLightingComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    if (light_type == ZR_GPU_LIGHT_TYPE_DIRECTIONAL) {
        let light_vector = normalize_or_zero(-light.direction_type.xyz);
        var direct_visibility = 1.0;
        if (receive_shadows) {
            direct_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
        }
        return shade_standard_pbr_light_vector_components_normalized(
            light_vector,
            base_radiance * direct_visibility,
            world_normal,
            roughness,
            direct_f0,
            direct_diffuse_brdf,
            world_view,
        );
    }

    let to_light = light.position_range.xyz - world_position;
    let distance_to_light = length(to_light);
    let range = max(light.position_range.w, EPSILON);
    if (distance_to_light >= range) {
        return ZrDeferredLightingComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    let light_vector = to_light / max(distance_to_light, EPSILON);
    let visibility = punctual_light_visibility(light, light_type, light_vector, distance_to_light, range);
    if (visibility <= 0.0) {
        return ZrDeferredLightingComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }
    var shadow_visibility = 1.0;
    if (receive_shadows) {
        shadow_visibility = zr_gpu_light_shadow_visibility(light, light_type, world_position, view_z);
    }
    return shade_standard_pbr_light_vector_components_normalized(
        light_vector,
        base_radiance * visibility * shadow_visibility,
        world_normal,
        roughness,
        direct_f0,
        direct_diffuse_brdf,
        world_view,
    );
}

fn gpu_light_lighting(frag_coord: vec2<f32>, world_position: vec3<f32>, normal_normalized: vec3<f32>, roughness: f32, metallic: f32, diffuse_color: vec3<f32>, view_dir_normalized: vec3<f32>, shading_model_id: u32, receive_shadows: bool) -> vec3<f32> {
    if (zr_light_grid_params.light_count == 0u || zr_light_grid_params.bin_count == 0u) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let view_z = zr_light_view_z(world_position, zr_light_grid_params);
    let bin = zr_light_zbin_index(view_z, zr_light_grid_params);
    let header = zr_light_zbin_header(bin, zr_light_grid_params);
    if (header.x == 0xFFFFu || header.x > header.y) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let world_normal = normal_normalized;
    let world_view = view_dir_normalized;
    var direct_f0 = vec3<f32>(0.0);
    var direct_diffuse_brdf = diffuse_color / ZR_PBR_EXTRAS_PI;
    if (shading_model_id != ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        let direct_metallic = clamp(metallic, 0.0, 1.0);
        direct_f0 = zr_pbr_material_f0(
            vec3<f32>(0.04),
            diffuse_color,
            direct_metallic,
        );
        direct_diffuse_brdf = diffuse_color
            * zr_surface_metallic_diffuse_energy_scale(direct_metallic)
            / ZR_PBR_EXTRAS_PI;
    }
    let tile_base = zr_light_tile_base(frag_coord, zr_light_grid_params);
    var accumulated = vec3<f32>(0.0, 0.0, 0.0);
    for (var word = header.x / 32u; word <= header.y / 32u; word = word + 1u) {
        var mask = zr_light_mask_word(tile_base, bin, word, zr_light_grid_params);
        while (mask != 0u) {
            let bit_index = firstTrailingBit(mask);
            let light_index = word * 32u + bit_index;
            accumulated = accumulated + shade_gpu_light_index(
                light_index,
                world_position,
                world_normal,
                roughness,
                diffuse_color,
                direct_f0,
                direct_diffuse_brdf,
                world_view,
                view_z,
                shading_model_id,
                receive_shadows,
            );
            mask = mask & (mask - 1u);
        }
    }

    return accumulated;
}

fn gpu_light_lighting_components(frag_coord: vec2<f32>, world_position: vec3<f32>, normal_normalized: vec3<f32>, roughness: f32, metallic: f32, diffuse_color: vec3<f32>, view_dir_normalized: vec3<f32>, receive_shadows: bool) -> ZrDeferredLightingComponents {
    if (zr_light_grid_params.light_count == 0u || zr_light_grid_params.bin_count == 0u) {
        return ZrDeferredLightingComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    let view_z = zr_light_view_z(world_position, zr_light_grid_params);
    let bin = zr_light_zbin_index(view_z, zr_light_grid_params);
    let header = zr_light_zbin_header(bin, zr_light_grid_params);
    if (header.x == 0xFFFFu || header.x > header.y) {
        return ZrDeferredLightingComponents(vec3<f32>(0.0), vec3<f32>(0.0));
    }

    let world_normal = normal_normalized;
    let world_view = view_dir_normalized;
    let direct_metallic = clamp(metallic, 0.0, 1.0);
    let direct_f0 = zr_pbr_material_f0(
        vec3<f32>(0.04),
        diffuse_color,
        direct_metallic,
    );
    let direct_diffuse_brdf = diffuse_color
        * zr_surface_metallic_diffuse_energy_scale(direct_metallic)
        / ZR_PBR_EXTRAS_PI;
    let tile_base = zr_light_tile_base(frag_coord, zr_light_grid_params);
    var diffuse = vec3<f32>(0.0);
    var retained = vec3<f32>(0.0);
    for (var word = header.x / 32u; word <= header.y / 32u; word = word + 1u) {
        var mask = zr_light_mask_word(tile_base, bin, word, zr_light_grid_params);
        while (mask != 0u) {
            let bit_index = firstTrailingBit(mask);
            let light_index = word * 32u + bit_index;
            let components = shade_gpu_light_index_components(
                light_index,
                world_position,
                world_normal,
                roughness,
                direct_f0,
                direct_diffuse_brdf,
                world_view,
                view_z,
                receive_shadows,
            );
            diffuse += components.diffuse;
            retained += components.retained;
            mask = mask & (mask - 1u);
        }
    }
    return ZrDeferredLightingComponents(diffuse, retained);
}

fn deferred_standard_pbr_diffuse_color(albedo: vec4<f32>) -> vec3<f32> {
    return zr_pbr_base_color(albedo.rgb);
}

fn deferred_diffuse_color(albedo: vec4<f32>, shading_model_id: u32) -> vec3<f32> {
    if (shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID) {
        return deferred_standard_pbr_diffuse_color(albedo);
    }
    return albedo.rgb;
}

fn add_deferred_emissive(shaded: vec4<f32>, emissive: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(shaded.rgb + max(emissive, vec3<f32>(0.0)), shaded.a);
}

fn apply_deferred_volumetric(
    shaded: vec4<f32>,
    position: vec4<f32>,
    depth: f32,
) -> vec4<f32> {
    return vec4<f32>(
        zr_volumetric_apply(shaded.rgb, position.xy, depth),
        shaded.a,
    );
}

fn shade_deferred_lit(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>, shading_model_id: u32) -> vec4<f32> {
    let metallic = clamp(material.r, 0.0, 1.0);
    let roughness = clamp(max(material.g, ZR_STANDARD_MATERIAL_MIN_ROUGHNESS), ZR_STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0);
    let occlusion = clamp(max(material.b, 0.0), 0.0, 1.0);
    let receive_shadows = decode_receive_shadows(material.a);
    let depth = clamp(textureLoad(scene_depth_tex, coord, 0), 0.0, 1.0);
    let world_position = reconstruct_world_position(coord, depth);
    let view_dir = zr_pbr_view_direction_ws(world_position);
    let screen_space_ao = clamp(textureLoad(ambient_occlusion_tex, coord, 0).r, 0.0, 1.0);
    let ambient = zr_deferred_ambient_radiance() * occlusion * screen_space_ao;
    let diffuse_color = deferred_diffuse_color(albedo, shading_model_id);
    let direct_lights = gpu_light_lighting(position.xy, world_position, normal, roughness, metallic, diffuse_color, view_dir, shading_model_id, receive_shadows);
    let environment_lights = zr_environment_pbr_components_normalized(
        world_position,
        normal,
        view_dir,
        roughness,
        metallic,
        diffuse_color,
        diffuse_color,
        occlusion,
        shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
    );
    var diffuse_energy_scale = vec3<f32>(1.0);
    if (shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID) {
        diffuse_energy_scale = vec3<f32>(zr_surface_metallic_diffuse_energy_scale(metallic));
    }
    let color = diffuse_color * diffuse_energy_scale * ambient
        + direct_lights
        + environment_lights.diffuse * screen_space_ao
        + environment_lights.specular;
    return vec4<f32>(color, albedo.a);
}

fn shade_deferred_subsurface_components(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>) -> ZrDeferredLightingComponents {
    let metallic = clamp(material.r, 0.0, 1.0);
    let roughness = clamp(max(material.g, ZR_STANDARD_MATERIAL_MIN_ROUGHNESS), ZR_STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0);
    let occlusion = clamp(max(material.b, 0.0), 0.0, 1.0);
    let receive_shadows = decode_receive_shadows(material.a);
    let depth = clamp(textureLoad(scene_depth_tex, coord, 0), 0.0, 1.0);
    let world_position = reconstruct_world_position(coord, depth);
    let view_dir = zr_pbr_view_direction_ws(world_position);
    let diffuse_color = deferred_standard_pbr_diffuse_color(albedo);
    let direct = gpu_light_lighting_components(position.xy, world_position, normal, roughness, metallic, diffuse_color, view_dir, receive_shadows);
    let environment = zr_environment_pbr_components_normalized(
        world_position,
        normal,
        view_dir,
        roughness,
        metallic,
        diffuse_color,
        diffuse_color,
        occlusion,
        true,
    );
    let ambient_diffuse_energy = vec3<f32>(zr_surface_metallic_diffuse_energy_scale(metallic));
    let screen_space_ao = clamp(textureLoad(ambient_occlusion_tex, coord, 0).r, 0.0, 1.0);
    return ZrDeferredLightingComponents(
        diffuse_color
            * ambient_diffuse_energy
            * zr_deferred_ambient_radiance()
            * occlusion
            * screen_space_ao
            + direct.diffuse
            + environment.diffuse * screen_space_ao,
        direct.retained + environment.specular,
    );
}

fn shade_deferred_pixel(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>, emissive: vec3<f32>, depth: f32, shading_model_id: u32) -> vec4<f32> {
    if (shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        return apply_deferred_volumetric(
            add_deferred_emissive(shade_deferred_unlit(albedo), emissive),
            position,
            depth,
        );
    }
    if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return apply_deferred_volumetric(
            add_deferred_emissive(
                shade_deferred_blinn_phong(position, coord, albedo, material, normal),
                emissive,
            ),
            position,
            depth,
        );
    }
    // zr-deferred-lighting-custom-shading-model-dispatch
    return apply_deferred_volumetric(
        add_deferred_emissive(
            shade_deferred_standard_pbr(position, coord, albedo, material, normal),
            emissive,
        ),
        position,
        depth,
    );
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = vec2<i32>(position.xy);
    let albedo = textureLoad(gbuffer_albedo_tex, coord, 0);
    if (albedo.a <= 0.001) {
        discard;
    }

    let encoded_normal = textureLoad(normal_tex, coord, 0).xyz;
    let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0, 1.0, 1.0));
    let material = textureLoad(gbuffer_material_tex, coord, 0);
    let emissive = textureLoad(gbuffer_emissive_tex, coord, 0);
    select_deferred_ambient(emissive);
    let depth = clamp(textureLoad(scene_depth_tex, coord, 0), 0.0, 1.0);
    let shading_model_id = decode_shading_model_id(material.a);
    return shade_deferred_pixel(position, coord, albedo, material, normal, emissive.rgb, depth, shading_model_id);
}

@fragment
fn fs_main_sss(@builtin(position) position: vec4<f32>) -> ZrDeferredLightingMrtOutput {
    let coord = vec2<i32>(position.xy);
    let albedo = textureLoad(gbuffer_albedo_tex, coord, 0);
    if (albedo.a <= 0.001) {
        discard;
    }
    let encoded_normal = textureLoad(normal_tex, coord, 0).xyz;
    let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0));
    let material = textureLoad(gbuffer_material_tex, coord, 0);
    let emissive_sample = textureLoad(gbuffer_emissive_tex, coord, 0);
    select_deferred_ambient(emissive_sample);
    let emissive = max(emissive_sample.rgb, vec3<f32>(0.0));
    let depth = clamp(textureLoad(scene_depth_tex, coord, 0), 0.0, 1.0);
    let shading_model_id = decode_shading_model_id(material.a);
    if (shading_model_id != 16u) {
        return ZrDeferredLightingMrtOutput(
            shade_deferred_pixel(position, coord, albedo, material, normal, emissive, depth, shading_model_id),
            vec4<f32>(0.0),
            vec4<f32>(0.0),
        );
    }

    let components = shade_deferred_subsurface_components(position, coord, albedo, material, normal);
    let transmittance = zr_volumetric_transmittance(position.xy, depth);
    let diffuse = max(components.diffuse, vec3<f32>(0.0)) * transmittance;
    let retained = max(components.retained + emissive, vec3<f32>(0.0)) * transmittance
        + zr_volumetric_scattering(position.xy, depth);
    return ZrDeferredLightingMrtOutput(
        vec4<f32>(diffuse + retained, albedo.a),
        vec4<f32>(diffuse, 1.0),
        vec4<f32>(retained, 1.0),
    );
}
