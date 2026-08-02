fn zr_scene_view_dir_ws(position_ws: vec3<f32>) -> vec3<f32> {
    let perspective_view_dir = zr_normalize_or_zero(scene.camera_world_position.xyz - position_ws);
    return zr_normalize_or_zero(mix(
        perspective_view_dir,
        scene.camera_view_direction.xyz,
        clamp(scene.camera_view_direction.w, 0.0, 1.0),
    ));
}

fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> {
    if (surface.shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        return surface.base_color.rgb + surface.emissive;
    }

    let diffuse_color = surface.base_color.rgb;
    let view_dir_ws = zr_scene_view_dir_ws(ctx.position_ws);
    let environment_lights = zr_environment_pbr_indirect(
        ctx.position_ws,
        surface.normal_ws,
        view_dir_ws,
        surface.roughness,
        surface.metallic,
        diffuse_color,
        surface.base_color.rgb,
        surface.occlusion,
        surface.shading_model_id == ZR_SHADING_MODEL_STANDARD_PBR_ID,
    );
    let ambient = scene.ambient_color.rgb * surface.occlusion;
    return diffuse_color * zr_surface_metallic_diffuse_energy_scale(surface.metallic) * ambient
        + environment_lights
        + surface.emissive;
}
