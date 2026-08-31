const ZR_SHADING_MODEL_UNLIT_ID: u32 = 0u;
const ZR_SHADING_MODEL_BLINN_PHONG_ID: u32 = 1u;
const ZR_SHADING_MODEL_STANDARD_PBR_ID: u32 = 2u;
const ZR_PBR_NO_ATTENUATION_DISTANCE: f32 = 1.0e30;

struct ZrVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) joints: vec4<u32>,
    @location(4) weights: vec4<f32>,
    @location(5) tangent: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) uv1: vec2<f32>,
    @builtin(vertex_index) vertex_index: u32,
};

struct ZrVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position_ws: vec3<f32>,
    @location(1) normal_ws: vec3<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) uv1: vec2<f32>,
    @location(4) tangent_ws: vec3<f32>,
    @location(5) tangent_handedness: f32,
    @location(6) color: vec4<f32>,
    @location(7) tint: vec4<f32>,
    @location(8) shadow_params: vec4<f32>,
    @location(9) motion_params: vec4<f32>,
    @location(10) @interpolate(flat) instance_index: u32,
};

struct ZrSurfaceOutput {
    base_color: vec4<f32>,
    normal_ws: vec3<f32>,
    tangent_ws: vec3<f32>,
    bitangent_ws: vec3<f32>,
    metallic: f32,
    roughness: f32,
    occlusion: f32,
    emissive: vec3<f32>,
    alpha_cutoff: f32,
    unlit: f32,
    shading_model_id: u32,
    clearcoat_normal_ws: vec3<f32>,
    clearcoat: f32,
    clearcoat_roughness: f32,
    anisotropy_strength: f32,
    anisotropy_rotation: f32,
    specular_transmission: f32,
    diffuse_transmission: f32,
    thickness: f32,
    ior: f32,
    dielectric_f0: vec3<f32>,
    attenuation_color: vec3<f32>,
    attenuation_distance: f32,
    custom0: vec4<f32>,
};

alias ZrSurfaceInput = ZrVertexOutput;

struct ZrDeferredGBufferOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) material: vec4<f32>,
    @location(3) emissive: vec4<f32>,
};

const ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK: u32 = 0x7Fu;
const ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG: u32 = 0x80u;

struct ZrShadingContext {
    frag_coord: vec2<f32>,
    position_ws: vec3<f32>,
    shadow_params: vec4<f32>,
    uv2: vec2<f32>,
    instance_index: u32,
};

fn zr_build_vertex_output(
    instance_index: u32,
    position_os: vec3<f32>,
    normal_os: vec3<f32>,
    tangent_os: vec4<f32>,
    uv0: vec2<f32>,
    uv1: vec2<f32>,
    color: vec4<f32>,
) -> ZrVertexOutput {
    var output: ZrVertexOutput;
    let instance = zr_gpu_scene_instance(instance_index);
    let world_from_local = instance.world_from_local;
    let instance_flags = instance.flags;
    let position_ws = world_from_local * vec4<f32>(position_os, 1.0);
    output.clip_position = scene.view_proj * position_ws;
    output.position_ws = position_ws.xyz;
    output.normal_ws = zr_normalize_or_zero(zr_gpu_scene_normal_to_world_direction(world_from_local, instance_flags, normal_os));
    output.uv0 = uv0;
    output.uv1 = uv1;
    output.tangent_ws = zr_normalize_or_zero(zr_gpu_scene_tangent_to_world_direction(world_from_local, instance_flags, tangent_os.xyz));
    output.tangent_handedness = select(-1.0, 1.0, tangent_os.w >= 0.0) * zr_gpu_scene_tangent_handedness_scale(instance_flags);
    output.color = color;
    output.tint = zr_gpu_scene_tint(instance_index);
    output.shadow_params = zr_gpu_scene_shadow_params(instance_index);
    output.motion_params = zr_gpu_scene_motion_params(instance_index);
    output.instance_index = instance_index;
    return output;
}

fn zr_build_shading_context(input: ZrVertexOutput) -> ZrShadingContext {
    var ctx: ZrShadingContext;
    ctx.frag_coord = input.clip_position.xy;
    ctx.position_ws = input.position_ws;
    ctx.shadow_params = input.shadow_params;
    ctx.uv2 = input.uv1;
    ctx.instance_index = input.instance_index;
    return ctx;
}

fn zr_normalize_or_zero(value: vec3<f32>) -> vec3<f32> {
    return zr_pbr_common_normalize_or_zero(value);
}

fn zr_surface_from_base_color(base_color: vec4<f32>) -> ZrSurfaceOutput {
    var surface: ZrSurfaceOutput;
    surface.base_color = base_color;
    surface.normal_ws = vec3<f32>(0.0, 0.0, 1.0);
    surface.tangent_ws = vec3<f32>(1.0, 0.0, 0.0);
    surface.bitangent_ws = vec3<f32>(0.0, 1.0, 0.0);
    surface.metallic = 0.0;
    surface.roughness = 1.0;
    surface.occlusion = 1.0;
    surface.emissive = vec3<f32>(0.0, 0.0, 0.0);
    surface.alpha_cutoff = 0.0;
    surface.unlit = 0.0;
    surface.shading_model_id = 2u;
    surface.clearcoat_normal_ws = surface.normal_ws;
    surface.clearcoat = 0.0;
    surface.clearcoat_roughness = 0.5;
    surface.anisotropy_strength = 0.0;
    surface.anisotropy_rotation = 0.0;
    surface.specular_transmission = 0.0;
    surface.diffuse_transmission = 0.0;
    surface.thickness = 0.0;
    surface.ior = 1.5;
    surface.dielectric_f0 = vec3<f32>(0.04);
    surface.attenuation_color = vec3<f32>(1.0);
    surface.attenuation_distance = ZR_PBR_NO_ATTENUATION_DISTANCE;
    surface.custom0 = vec4<f32>(0.0);
    return surface;
}

fn zr_surface_default(input: ZrSurfaceInput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(input.color);
}

fn zr_raster_facing_sign(front_facing: bool) -> f32 {
    return select(-1.0, 1.0, !ZR_FEATURE_DOUBLE_SIDED || front_facing);
}

fn zr_raster_facing_normal(normal_ws: vec3<f32>, front_facing: bool) -> vec3<f32> {
    return normal_ws * zr_raster_facing_sign(front_facing);
}

fn zr_surface_apply_raster_facing(
    surface: ZrSurfaceOutput,
    front_facing: bool,
) -> ZrSurfaceOutput {
    let facing_sign = zr_raster_facing_sign(front_facing);
    var oriented = surface;
    oriented.normal_ws = surface.normal_ws * facing_sign;
    oriented.bitangent_ws = surface.bitangent_ws * facing_sign;
    oriented.clearcoat_normal_ws = surface.clearcoat_normal_ws * facing_sign;
    return oriented;
}

fn zr_surface_apply_environment_capture_policy(
    surface: ZrSurfaceOutput,
) -> ZrSurfaceOutput {
    if (scene.sky_sun_params.w > 0.5) {
        var resolved = surface;
        resolved.roughness = 1.0;
        resolved.clearcoat_roughness = 1.0;
        return resolved;
    }
    return surface;
}

fn zr_surface_fails_alpha_clip(surface: ZrSurfaceOutput) -> bool {
    return ZR_FEATURE_ALPHA_TEST && surface.base_color.a < surface.alpha_cutoff;
}

fn zr_deferred_encode_material_flags(shading_model_id: u32, receive_shadows: bool) -> f32 {
    let model = shading_model_id & ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK;
    let receive_shadow_flag = select(
        0u,
        ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG,
        receive_shadows,
    );
    return f32(model | receive_shadow_flag) / 255.0;
}
