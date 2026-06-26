struct ZrDeferredGBufferOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) material: vec4<f32>,
};

const ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK: u32 = 0x7Fu;
const ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG: u32 = 0x80u;

fn zr_deferred_encode_material_flags(shading_model_id: u32, receive_shadows: bool) -> f32 {
    let model = shading_model_id & ZR_DEFERRED_MATERIAL_SHADING_MODEL_MASK;
    let receive_shadow_flag = select(
        0u,
        ZR_DEFERRED_MATERIAL_RECEIVE_SHADOWS_FLAG,
        receive_shadows,
    );
    return f32(model | receive_shadow_flag) / 255.0;
}

fn zr_vs_main_impl(v: ZrVertexInput, instance_index: u32) -> ZrVertexOutput {
    return zr_build_vertex_output(
        instance_index,
        fetch_position(v, instance_index),
        fetch_normal(v, instance_index),
        fetch_tangent(v, instance_index),
        fetch_uv0(v),
        fetch_uv1(v),
        fetch_color(v),
    );
}

@vertex
fn zr_vs_main(v: ZrVertexInput, @builtin(instance_index) instance_index: u32) -> ZrVertexOutput {
    return zr_vs_main_impl(v, instance_index);
}

@vertex
fn vs_main(v: ZrVertexInput, @builtin(instance_index) instance_index: u32) -> ZrVertexOutput {
    return zr_vs_main_impl(v, instance_index);
}

fn zr_deferred_apply_alpha_clip(surface: ZrSurfaceOutput) {
    if (zr_surface_fails_alpha_clip(surface)) {
        discard;
    }
}

fn zr_fs_main_impl(input: ZrVertexOutput) -> ZrDeferredGBufferOutput {
    let surface = zr_material_surface(input);
    zr_deferred_apply_alpha_clip(surface);
    let receive_shadows = input.shadow_params.z > 0.5;
    return ZrDeferredGBufferOutput(
        surface.base_color,
        vec4<f32>(
            surface.metallic,
            clamp(surface.roughness, 0.04, 1.0),
            clamp(surface.occlusion, 0.0, 1.0),
            zr_deferred_encode_material_flags(surface.shading_model_id, receive_shadows),
        ),
    );
}

@fragment
fn zr_fs_main(input: ZrVertexOutput) -> ZrDeferredGBufferOutput {
    return zr_fs_main_impl(input);
}

@fragment
fn fs_main(input: ZrVertexOutput) -> ZrDeferredGBufferOutput {
    return zr_fs_main_impl(input);
}
