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
    metallic: f32,
    roughness: f32,
    occlusion: f32,
    emissive: vec3<f32>,
    alpha_cutoff: f32,
    unlit: f32,
    shading_model_id: u32,
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
    let world_from_local = zr_world_from_local(instance_index);
    let position_ws = world_from_local * vec4<f32>(position_os, 1.0);
    output.clip_position = scene.view_proj * position_ws;
    output.position_ws = position_ws.xyz;
    output.normal_ws = zr_normalize_or_zero((world_from_local * vec4<f32>(normal_os, 0.0)).xyz);
    output.uv0 = uv0;
    output.uv1 = uv1;
    output.tangent_ws = zr_normalize_or_zero((world_from_local * vec4<f32>(tangent_os.xyz, 0.0)).xyz);
    output.tangent_handedness = select(-1.0, 1.0, tangent_os.w >= 0.0);
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
    let value_length = length(value);
    if (value_length <= 0.000001) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    return value / value_length;
}

fn zr_surface_from_base_color(base_color: vec4<f32>) -> ZrSurfaceOutput {
    var surface: ZrSurfaceOutput;
    surface.base_color = base_color;
    surface.normal_ws = vec3<f32>(0.0, 0.0, 1.0);
    surface.metallic = 0.0;
    surface.roughness = 1.0;
    surface.occlusion = 1.0;
    surface.emissive = vec3<f32>(0.0, 0.0, 0.0);
    surface.alpha_cutoff = 0.0;
    surface.unlit = 0.0;
    surface.shading_model_id = 2u;
    surface.custom0 = vec4<f32>(0.0);
    return surface;
}

fn zr_surface_default(input: ZrSurfaceInput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(input.color);
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
