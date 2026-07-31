fn zr_vs_main_impl(v: ZrVertexInput, instance_index: u32) -> ZrVertexOutput {
    return zr_build_vertex_output(
        instance_index,
        fetch_position(v, instance_index),
        fetch_normal(v, instance_index),
        fetch_tangent(v, instance_index),
        fetch_uv0(v),
        fetch_uv1(v),
        fetch_color(v, instance_index),
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
    var output = encode_gbuffer(surface, zr_build_shading_context(input));
    if (surface.unlit < 0.5 && surface.shading_model_id != 0u) {
        let diffuse_energy_scale = select(
            1.0,
            zr_surface_metallic_diffuse_energy_scale(surface.metallic),
            surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
        );
        let baked_indirect = surface.base_color.rgb * diffuse_energy_scale
            * clamp(surface.occlusion, 0.0, 1.0)
            * zr_lightmap_baked_irradiance(
                input.instance_index,
                input.uv1,
                input.position_ws,
                surface.normal_ws,
            );
        output.emissive = vec4<f32>(output.emissive.rgb + baked_indirect, output.emissive.a);
    }
    return output;
}

@fragment
fn zr_fs_main(input: ZrVertexOutput) -> ZrDeferredGBufferOutput {
    return zr_fs_main_impl(input);
}

@fragment
fn fs_main(input: ZrVertexOutput) -> ZrDeferredGBufferOutput {
    return zr_fs_main_impl(input);
}
