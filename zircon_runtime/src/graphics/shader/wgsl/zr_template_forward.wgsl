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

fn zr_apply_alpha_clip(surface: ZrSurfaceOutput) {
    if (zr_surface_fails_alpha_clip(surface)) {
        discard;
    }
}

fn zr_fs_main_impl(input: ZrVertexOutput, front_facing: bool) -> vec4<f32> {
    var surface = zr_surface_apply_raster_facing(zr_material_surface(input), front_facing);
    zr_apply_alpha_clip(surface);
    surface = zr_surface_apply_environment_capture_policy(surface);
    let shaded = shade_forward(surface, zr_build_shading_context(input));
    var baked_indirect = vec3<f32>(0.0);
    if (surface.unlit < 0.5 && surface.shading_model_id != 0u) {
        var baked_diffuse_color = surface.base_color.rgb;
        if (surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID) {
            baked_diffuse_color = zr_pbr_base_color(surface.base_color.rgb);
        }
        var diffuse_energy_scale = vec3<f32>(1.0);
        if (surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID) {
            diffuse_energy_scale = vec3<f32>(
                zr_surface_metallic_diffuse_energy_scale(surface.metallic),
            );
        }
        baked_indirect = baked_diffuse_color * diffuse_energy_scale
            * clamp(surface.occlusion, 0.0, 1.0)
            * zr_lightmap_baked_irradiance(
                input.instance_index,
                input.uv1,
                input.position_ws,
                surface.normal_ws,
            );
    }
    return vec4<f32>(
        zr_volumetric_apply(shaded + baked_indirect, input.clip_position.xy, input.clip_position.z),
        surface.base_color.a,
    );
}

@fragment
fn zr_fs_main(
    input: ZrVertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> @location(0) vec4<f32> {
    return zr_fs_main_impl(input, front_facing);
}

@fragment
fn fs_main(
    input: ZrVertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> @location(0) vec4<f32> {
    return zr_fs_main_impl(input, front_facing);
}
