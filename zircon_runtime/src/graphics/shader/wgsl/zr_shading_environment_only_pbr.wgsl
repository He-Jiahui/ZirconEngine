fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> {
    if (surface.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        return surface.base_color.rgb + surface.emissive;
    }

    let diffuse_color = zr_pbr_base_color(surface.base_color.rgb);
    let view_dir_ws = zr_pbr_view_direction_ws(ctx.position_ws);
    let world_normal = zr_normalize_or_zero(surface.normal_ws);
    let environment_lights = zr_environment_pbr_indirect(
        ctx.position_ws,
        world_normal,
        view_dir_ws,
        surface.roughness,
        surface.metallic,
        diffuse_color,
        diffuse_color,
        surface.dielectric_f0,
        surface.occlusion,
        surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
    );
    let ambient = zr_scene_ambient_color(
        zr_gpu_scene_has_lightmap(ctx.instance_index),
    ) * surface.occlusion;
    let diffuse_energy = vec3<f32>(
        zr_surface_metallic_diffuse_energy_scale(surface.metallic),
    );
    return diffuse_color * diffuse_energy * ambient
        + environment_lights
        + surface.emissive;
}
