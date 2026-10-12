const ZR_STANDARD_MATERIAL_MIN_ROUGHNESS: f32 = 0.001;

/// 按 deferred 读取端约定输出四个固定附件；material.a 编码 shading-model 与接收阴影位。
/// 标准 PBR 将透明度保留在 normal.a，通道分配须与 ZrDeferredGBufferOutput 一致。
fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrDeferredGBufferOutput {
    let receive_shadows = ctx.shadow_params.z > 0.5;
    return ZrDeferredGBufferOutput(
        surface.base_color,
        vec4<f32>(surface.normal_ws * 0.5 + vec3<f32>(0.5), surface.base_color.a),
        vec4<f32>(
            surface.metallic,
            clamp(surface.roughness, ZR_STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0),
            clamp(surface.occlusion, 0.0, 1.0),
            zr_deferred_encode_material_flags(surface.shading_model_id, receive_shadows),
        ),
        vec4<f32>(max(surface.emissive, vec3<f32>(0.0)), 1.0),
    );
}
