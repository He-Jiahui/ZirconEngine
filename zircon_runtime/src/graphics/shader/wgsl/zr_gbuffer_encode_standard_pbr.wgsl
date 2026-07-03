fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrDeferredGBufferOutput {
    let receive_shadows = ctx.shadow_params.z > 0.5;
    return ZrDeferredGBufferOutput(
        surface.base_color,
        vec4<f32>(surface.normal_ws * 0.5 + vec3<f32>(0.5), surface.base_color.a),
        vec4<f32>(
            surface.metallic,
            clamp(surface.roughness, 0.04, 1.0),
            clamp(surface.occlusion, 0.0, 1.0),
            zr_deferred_encode_material_flags(surface.shading_model_id, receive_shadows),
        ),
    );
}
