const ZR_SHADING_MODEL_SUBSURFACE_ID: u32 = 16u;

fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrDeferredGBufferOutput {
    let receive_shadows = ctx.shadow_params.z > 0.5;
    let profile_index = clamp(surface.custom0.w, 0.0, 15.0) / 255.0;
    return ZrDeferredGBufferOutput(
        surface.base_color,
        vec4<f32>(surface.normal_ws * 0.5 + vec3<f32>(0.5), profile_index),
        vec4<f32>(
            surface.metallic,
            clamp(surface.roughness, 0.001, 1.0),
            clamp(surface.occlusion, 0.0, 1.0),
            zr_deferred_encode_material_flags(ZR_SHADING_MODEL_SUBSURFACE_ID, receive_shadows),
        ),
        vec4<f32>(max(surface.emissive, vec3<f32>(0.0)), 1.0),
    );
}
